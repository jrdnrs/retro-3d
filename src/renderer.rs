use std::collections::VecDeque;

use maths::{
    geometry::{Polygon, Segment},
    linear::{Mat2f, Vec2f},
};

use crate::{
    app::{FAR, MAP_DEPTH_RANGE, NEAR},
    camera::Camera,
    colour::BGRA8,
    player::Player,
    surface::{Sector, Wall},
    textures::{Texture, Textures, MIP_FACTOR, MIP_LEVELS, MIP_SCALES},
};

pub struct Framebuffer {
    width: usize,
    height: usize,
    half_width: f32,
    half_height: f32,
    pixels: Vec<BGRA8>,
}

impl Framebuffer {
    pub fn new(width: usize, height: usize) -> Self {
        let len = width * height;
        let pixels = vec![BGRA8::default(); len];

        Self {
            width,
            height,
            half_width: width as f32 * 0.5,
            half_height: height as f32 * 0.5,
            pixels,
        }
    }

    pub fn pixels(&self) -> &[BGRA8] {
        &self.pixels
    }

    pub fn pixels_as_u32(&self) -> &[u32] {
        unsafe { core::mem::transmute(&self.pixels as &[BGRA8]) }
    }

    pub fn pixels_as_u8(&self) -> &[u8] {
        unsafe {
            core::slice::from_raw_parts(
                self.pixels.as_ptr() as *const u8,
                self.pixels.len() * core::mem::size_of::<BGRA8>(),
            )
        }
    }

    pub fn fill(&mut self, colour: BGRA8) {
        self.pixels.fill(colour);
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, colour: BGRA8) {
        debug_assert!(x < self.width && y < self.height);
        let index = (y * self.width) + x;

        self.pixels[index] = colour;
    }

    pub fn get_pixel(&self, x: usize, y: usize) -> BGRA8 {
        debug_assert!(x < self.width && y < self.height);
        let index = (y * self.width) + x;

        self.pixels[index]
    }

    pub fn blend_pixel(&mut self, x: usize, y: usize, colour: BGRA8, alpha: u8) {
        debug_assert!(x < self.width && y < self.height);
        let index = (y * self.width) + x;

        let blended = self.pixels[index].blend(colour, alpha);
        self.pixels[index] = blended;
    }

    pub fn draw_line(&mut self, mut start: Vec2f, mut end: Vec2f, colour: BGRA8) {
        start.x = start.x.clamp(0.0, (self.width - 1) as f32);
        start.y = start.y.clamp(0.0, (self.height - 1) as f32);
        end.x = end.x.clamp(0.0, (self.width - 1) as f32);
        end.y = end.y.clamp(0.0, (self.height - 1) as f32);

        let delta = end - start;
        let steps = delta.x.abs().max(delta.y.abs());
        let increment = delta / steps;

        let mut position = start;

        for _ in 0..steps as usize {
            let x = position.x.round() as usize;
            let y = position.y.round() as usize;

            self.set_pixel(x, y, colour);
            position += increment;
        }
    }
}

struct RenderTask {
    sector_index: usize,
    x_min: usize,
    x_max: usize,
}

pub struct Renderer {
    tasks: VecDeque<RenderTask>,

    pub framebuffer: Framebuffer,
    camera: Camera,
    frustum: Polygon,

    h_fov: f32,
    v_fov: f32,
    focal_width: f32,
    focal_height: f32,
    pitch_shear: f32,

    /// For each X coordinate, defines the upper (inc.) and lower (exc.) bounds of the portal.
    /// This is used as the clipping region for the next sector to be rendered within.
    portal_bounds_min: Vec<u16>,
    portal_bounds_max: Vec<u16>,

    /// For each X coordinate, defines the upper (inc.) and lower (exc.) bounds of the walls that
    /// have been rendered.
    wall_bounds_min: Vec<u16>,
    wall_bounds_max: Vec<u16>,

    /// For each Y coordinate, stores starting (inc.) X coordinate of the horizontal span used to draw the
    /// floor/ceiling.
    span_start: Vec<u16>,

    /// For each Y coordinate, stores the ratio of the Y coordinate to the focal height, where the
    /// middle of the screen is 0.0. This is used in calculating the depth of a floor/ceiling, based on
    /// where it appears on the screen.
    focal_height_ratios: Vec<f32>,
}

impl Renderer {
    pub fn new(screen_width: usize, screen_height: usize, h_fov: f32) -> Self {
        let tasks = VecDeque::new();

        let framebuffer = Framebuffer::new(screen_width, screen_height);
        let camera = Camera::new(Vec2f::ZERO, 0.0);

        let aspect_ratio = framebuffer.width as f32 / framebuffer.height as f32;
        let v_fov = h_fov / aspect_ratio;

        // This is used during perspective projection to convert from camera space to screen space.
        // It is essentially a scaling factor that is used to get a pixel coordinate from a
        // coordinate in camera space, taking into account the field of view and screen size.
        let focal_width = framebuffer.half_width / (h_fov * 0.5).to_radians().tan();
        let focal_height = framebuffer.half_height / (v_fov * 0.5).to_radians().tan();

        let pitch_shear = camera.pitch_tan * focal_height;

        let tan = (h_fov * 0.5).to_radians().tan();
        let opp_far = FAR * tan;
        let opp_near = NEAR * tan;
        let frustum = Polygon::from_vertices(vec![
            Vec2f::new(-opp_far, FAR),
            Vec2f::new(opp_far, FAR),
            Vec2f::new(opp_near, NEAR),
            Vec2f::new(-opp_near, NEAR),
        ]);

        let portal_bounds_min = vec![0; screen_width];
        let portal_bounds_max = vec![screen_height as u16; screen_width];

        let wall_bounds_min = vec![0; screen_width];
        let wall_bounds_max = vec![0; screen_width];

        let span_start = vec![0; screen_height];

        let focal_height_ratios = vec![0.0; screen_height];

        Self {
            tasks,

            framebuffer,
            camera,
            frustum,

            h_fov,
            v_fov,
            focal_width,
            focal_height,
            pitch_shear,

            portal_bounds_min,
            portal_bounds_max,

            wall_bounds_min,
            wall_bounds_max,

            span_start,

            focal_height_ratios,
        }
    }

    pub fn update(&mut self, player: &Player, textures: &Textures, sectors: &[Sector]) {
        // reset buffers
        // self.framebuffer.fill(BGRA8::CYAN);
        self.portal_bounds_min.fill(0);
        self.portal_bounds_max.fill(self.framebuffer.height as u16);
        self.wall_bounds_min.fill(0);
        self.wall_bounds_max.fill(0);

        // use player camera
        self.camera = player.camera.clone();
        self.pitch_shear = self.camera.pitch_tan * self.focal_height;

        // Precalculate focal height ratios
        for y in 0..self.framebuffer.height {
            let y_offset = y as f32 - self.framebuffer.half_height - self.pitch_shear;
            self.focal_height_ratios[y] = self.focal_height / y_offset;
        }

        // Add initial task to render the sector that the player is in
        self.tasks.push_back(RenderTask {
            sector_index: 0,
            x_min: 0,
            x_max: self.framebuffer.width,
        });

        // Sectors are rendered in a breadth-first manner, and each portal encountered is added
        // to the queue of tasks to be rendered.
        while self.tasks.len() > 0 {
            self.draw_sector(sectors, textures);
        }
    }

    pub fn set_viewport(&mut self, width: usize, height: usize) {
        if self.framebuffer.width == width && self.framebuffer.height == height {
            return;
        }

        self.framebuffer = Framebuffer::new(width, height);
        let aspect_ratio = self.framebuffer.width as f32 / self.framebuffer.height as f32;
        self.v_fov = self.h_fov / aspect_ratio;

        // This is used during perspective projection to convert from camera space to screen space.
        // It is essentially a scaling factor that is used to get a pixel coordinate from a
        // coordinate in camera space, taking into account the field of view and screen size.
        self.focal_width = self.framebuffer.half_width / (self.h_fov * 0.5).to_radians().tan();
        self.focal_height = self.framebuffer.half_height / (self.v_fov * 0.5).to_radians().tan();

        self.portal_bounds_min = vec![0; width];
        self.portal_bounds_max = vec![height as u16; width];

        self.wall_bounds_min = vec![0; width];
        self.wall_bounds_max = vec![0; width];

        self.span_start = vec![0; height];

        self.focal_height_ratios = vec![0.0; height];
    }

    pub fn transform_view(&self, point: Vec2f) -> Vec2f {
        (point - self.camera.position).rotate(self.camera.yaw_sin, self.camera.yaw_cos)
    }

    pub fn project_screen_space(&self, point: Vec2f, height_offset: f32) -> (Vec2f, f32) {
        let z = point.y;
        debug_assert!(z > 0.0);
        let inv_z = 1.0 / z;

        // construct pseudo vertical camera space coordinate
        let y = self.camera.height_offset - height_offset;

        // perspective projection into screen space
        let mut screen_space_x = (point.x * self.focal_width) * inv_z;
        let mut screen_space_y = (y * self.focal_height) * inv_z;

        // shift everything along Y-axis to simulate pitch (y-shearing)
        screen_space_y += self.pitch_shear;

        // move origin to centre of screen
        screen_space_x += self.framebuffer.half_width;
        screen_space_y += self.framebuffer.half_height;

        (Vec2f::new(screen_space_x, screen_space_y), inv_z)
    }

    pub fn draw_sector(&mut self, sectors: &[Sector], textures: &Textures) {
        let task = self.tasks.pop_front().unwrap();
        let sector = &sectors[task.sector_index];

        for wall in sector.walls.iter() {
            let texture = textures.get(wall.texture_index).unwrap();

            self.draw_wall(sectors, texture, sector, wall, &task);
        }

        // Draw sector ceiling
        self.draw_plane(
            textures.get(sector.ceiling.texture_index).unwrap(),
            sector.ceiling.texture_offset,
            &sector.ceiling.texture_scale_rotate,
            unsafe { &*(&self.portal_bounds_min as *const Vec<u16>) },
            unsafe { &*(&self.wall_bounds_min as *const Vec<u16>) },
            self.camera.height_offset - sector.ceiling.height,
            task.x_min,
            task.x_max,
        );

        // Draw sector floor
        self.draw_plane(
            textures.get(sector.floor.texture_index).unwrap(),
            sector.floor.texture_offset,
            &sector.floor.texture_scale_rotate,
            unsafe { &*(&self.wall_bounds_max as *const Vec<u16>) },
            unsafe { &*(&self.portal_bounds_max as *const Vec<u16>) },
            self.camera.height_offset - sector.floor.height,
            task.x_min,
            task.x_max,
        );
    }

    fn draw_wall(
        &mut self,
        sectors: &[Sector],
        texture: &Texture,
        sector: &Sector,
        wall: &Wall,
        task: &RenderTask,
    ) {
        // Transform coordinates based on camera position and orientation
        let mut vs_a = self.transform_view(wall.a);
        let mut vs_b = self.transform_view(wall.b);

        // Frustum culling
        if !Segment::new(vs_a, vs_b).overlaps_polygon(&self.frustum) {
            return;
        }

        // TODO: Consider precalculating these values, but we must then make sure to update
        // texture coordinates for all walls in a sector when the sector's ceiling/floor height changes.
        let mut tex_coord_a = Vec2f::new(0.0, 0.0);
        let mut tex_coord_b = Vec2f::new(wall.width, sector.ceiling.height - sector.floor.height);
        tex_coord_a += wall.texture_offset;
        tex_coord_b += wall.texture_offset;
        tex_coord_a *= wall.texture_scale;
        tex_coord_b *= wall.texture_scale;

        // Near plane clipping
        // Due to frustum culling, at most one vertex will be clipped
        if vs_a.y < NEAR {
            let t = (NEAR - vs_a.y) / (vs_b.y - vs_a.y);

            vs_a += (vs_b - vs_a) * t;
            tex_coord_a.x += (tex_coord_b.x - tex_coord_a.x) * t;
        } else if vs_b.y < NEAR {
            let t = (NEAR - vs_b.y) / (vs_a.y - vs_b.y);

            vs_b += (vs_a - vs_b) * t;
            tex_coord_b.x += (tex_coord_a.x - tex_coord_b.x) * t;
        }

        // Perspective projection into screen space
        let top_a = self.project_screen_space(vs_a, sector.ceiling.height);
        let top_b = self.project_screen_space(vs_b, sector.ceiling.height);
        let bottom_a = self.project_screen_space(vs_a, sector.floor.height);
        let bottom_b = self.project_screen_space(vs_b, sector.floor.height);

        // Early out if wall is back-facing
        if top_a.0.x >= top_b.0.x {
            return;
        }

        // Let's clamp X to the screen space bounds
        let x1_clamp = (top_a.0.x as usize).clamp(task.x_min, task.x_max);
        let x2_clamp = (top_b.0.x as usize).clamp(task.x_min, task.x_max);

        let renderable_wall = RenderableWall {
            vertices: [top_a.0, top_b.0, bottom_a.0, bottom_b.0],
            tex_coords: [tex_coord_a, tex_coord_b],
            inv_depths: [top_a.1, top_b.1],
        };

        self.rasterise_wall(&renderable_wall, texture, x1_clamp, x2_clamp);

        if let Some(sector_index) = wall.portal {
            // let neighbour_top_a =
            //     self.project_screen_space(&vs_a, sectors[sector_index].ceiling.height);
            // let neighbour_top_b =
            //     self.project_screen_space(&vs_b, sectors[sector_index].ceiling.height);
            // let neighbour_bottom_a =
            //     self.project_screen_space(&vs_a, sectors[sector_index].floor.height);
            // let neighbour_bottom_b =
            //     self.project_screen_space(&vs_b, sectors[sector_index].floor.height);

            // Rasterise portal wall

            // TODO:
            // Portal wall defines new portal bounds. These bounds define where floor/ceiling is drawn for
            // the current sector, which happens AFTER all walls (including portal walls) have been drawn.
            // So, we need to store these bounds separately (maybe)

            self.tasks.push_back(RenderTask {
                sector_index,
                x_min: x1_clamp,
                x_max: x2_clamp,
            })
        }
    }

    fn rasterise_wall(
        &mut self,
        wall: &RenderableWall,
        texture: &Texture,
        x_min: usize,
        x_max: usize,
    ) {
        const TOP_A: usize = 0;
        const TOP_B: usize = 1;
        const BOTTOM_A: usize = 2;
        const BOTTOM_B: usize = 3;

        let x_delta = wall.vertices[TOP_B].x - wall.vertices[TOP_A].x;
        debug_assert!(x_delta > 0.0);
        let inv_x_delta = 1.0 / x_delta;

        // Divide texture coordinates by depth to account for perspective projection during interpolation.
        // After interpolation, multiply by depth to recover the original texture coordinates.
        // For walls, we only need to do this for the X coordinate, as the Y coordinate has constant depth.
        let u_depth_a = wall.tex_coords[TOP_A].x * wall.inv_depths[TOP_A];
        let u_depth_b = wall.tex_coords[TOP_B].x * wall.inv_depths[TOP_B];

        // Gradients with respect to X
        let inv_depth_m = (wall.inv_depths[TOP_B] - wall.inv_depths[TOP_A]) * inv_x_delta;
        let top_y_m = (wall.vertices[TOP_B].y - wall.vertices[TOP_A].y) * inv_x_delta;
        let bottom_y_m = (wall.vertices[BOTTOM_B].y - wall.vertices[BOTTOM_A].y) * inv_x_delta;
        let u_depth_m = (u_depth_b - u_depth_a) * inv_x_delta;

        // Interpolator start values
        let mut inv_depth = wall.inv_depths[TOP_A];
        let mut top_y = wall.vertices[TOP_A].y;
        let mut bottom_y = wall.vertices[BOTTOM_A].y;
        let mut u_depth = u_depth_a;

        // X offset caused by clamping vertices to screen space bounds
        let x_clamp_offset = x_min as f32 - wall.vertices[TOP_A].x;

        // We need to update our interpolators to account difference in X caused by clamping
        inv_depth += inv_depth_m * x_clamp_offset;
        top_y += top_y_m * x_clamp_offset;
        bottom_y += bottom_y_m * x_clamp_offset;
        u_depth += u_depth_m * x_clamp_offset;

        // Draw wall, one column at a time
        for x in x_min..x_max {
            let min_portal_bound = self.portal_bounds_min[x] as usize;
            let max_portal_bound = self.portal_bounds_max[x] as usize;

            // Clamp Y to the screen space bounds
            let y_min = (top_y as usize).clamp(min_portal_bound, max_portal_bound);
            let y_max = (bottom_y as usize).clamp(min_portal_bound, max_portal_bound);
            let y_clamp_offset = y_min as f32 - top_y;

            // Interpolate V with respect to Y
            let v_m = (wall.tex_coords[TOP_B].y - wall.tex_coords[TOP_A].y) / (bottom_y - top_y);
            let mut v = wall.tex_coords[TOP_A].y;

            // Account for difference caused by clamping Y
            v += v_m * y_clamp_offset;

            // Recover U texture coordinate after interpolating in depth space
            let depth = 1.0 / inv_depth;
            let u = u_depth * depth;

            let normal_depth = normalise_depth(depth);
            let mip_level = mip_level(normal_depth);
            let mip_scale = MIP_SCALES[mip_level];

            let texture_x = (u * mip_scale) as usize & (texture.levels[mip_level].width - 1);

            for y in y_min..y_max {
                let texture_y = (v * mip_scale) as usize & (texture.levels[mip_level].height - 1);

                let colour = texture.sample(texture_x, texture_y, mip_level);
                self.framebuffer.set_pixel(x, y, colour);

                v += v_m;
            }

            self.wall_bounds_min[x] = y_min as u16;
            self.wall_bounds_max[x] = y_max as u16;

            inv_depth += inv_depth_m;
            top_y += top_y_m;
            bottom_y += bottom_y_m;
            u_depth += u_depth_m;
        }
    }

    fn draw_plane(
        &mut self,
        texture: &Texture,
        texture_offset: Vec2f,
        texture_scale_rotate: &Mat2f,
        min_bounds: &[u16],
        max_bounds: &[u16],
        height_offset: f32,
        x_min: usize,
        x_max: usize,
    ) {
        let inv_focal_width = 1.0 / self.focal_width;

        // Portal and wall bounds are collected during rasterisation of walls. We can use these to
        // draw floors and ceilings horizontally, which allows for fewer depth calculations as
        // depth is constant horizontally.

        // Vertical bounds which indicate 'open' hozizontal lines
        let mut y_min = min_bounds[x_min];
        let mut y_max = y_min;

        for x in x_min..x_max {
            let min_bound = min_bounds[x];
            let max_bound = max_bounds[x];

            while min_bound < y_min {
                y_min -= 1;
                self.span_start[y_min as usize] = x as u16;
            }

            while max_bound > y_max {
                self.span_start[y_max as usize] = x as u16;
                y_max += 1;
            }

            while min_bound > y_min {
                self.rasterise_plane_span(
                    texture,
                    texture_offset,
                    texture_scale_rotate,
                    height_offset,
                    inv_focal_width,
                    y_min as usize,
                    self.span_start[y_min as usize] as usize,
                    x,
                );
                y_min += 1;
            }

            while max_bound < y_max {
                y_max -= 1;
                self.rasterise_plane_span(
                    texture,
                    texture_offset,
                    texture_scale_rotate,
                    height_offset,
                    inv_focal_width,
                    y_max as usize,
                    self.span_start[y_max as usize] as usize,
                    x,
                );
            }
        }

        // Draw remaining for any horizontal lines that are still 'open'
        for y in y_min..y_max {
            self.rasterise_plane_span(
                texture,
                texture_offset,
                texture_scale_rotate,
                height_offset,
                inv_focal_width,
                y as usize,
                self.span_start[y as usize] as usize,
                x_max,
            );
        }
    }

    fn rasterise_plane_span(
        &mut self,
        texture: &Texture,
        texture_offset: Vec2f,
        texture_scale_rotate: &Mat2f,
        height_offset: f32,
        inv_focal_width: f32,
        y: usize,
        x_min: usize,
        x_max: usize,
    ) {
        let focal_height_ratio = self.focal_height_ratios[y];
        let depth = focal_height_ratio * height_offset;
        let normal_depth = normalise_depth(depth);
        let mip_level = mip_level(normal_depth);
        let mip_scale = MIP_SCALES[mip_level];

        let ws_1 = Vec2f::new(
            ((x_min as f32 - self.framebuffer.half_width) * depth) * inv_focal_width,
            -depth,
        )
        .rotate(self.camera.yaw_sin, self.camera.yaw_cos)
            + Vec2f::new(self.camera.position.x, -self.camera.position.y);

        let ws_2 = Vec2f::new(
            ((x_max as f32 - self.framebuffer.half_width) * depth) * inv_focal_width,
            -depth,
        )
        .rotate(self.camera.yaw_sin, self.camera.yaw_cos)
            + Vec2f::new(self.camera.position.x, -self.camera.position.y);

        let mut tex_coord_a = ws_1 * mip_scale;
        let mut tex_coord_b = ws_2 * mip_scale;
        tex_coord_a += texture_offset;
        tex_coord_b += texture_offset;
        tex_coord_a = *texture_scale_rotate * tex_coord_a;
        tex_coord_b = *texture_scale_rotate * tex_coord_b;

        let inv_x_delta = 1.0 / (x_max - x_min) as f32;

        let v_m = (tex_coord_b.y - tex_coord_a.y) * inv_x_delta;
        let mut v = tex_coord_a.y;

        let u_m = (tex_coord_b.x - tex_coord_a.x) * inv_x_delta;
        let mut u = tex_coord_a.x;

        for x in x_min..x_max {
            let texture_x = u.abs() as usize & (texture.levels[mip_level].width - 1);
            let texture_y = v.abs() as usize & (texture.levels[mip_level].height - 1);

            let colour = texture.sample(texture_x, texture_y, mip_level);

            self.framebuffer.set_pixel(x, y, colour);

            u += u_m;
            v += v_m;
        }
    }
}

fn normalise_depth(depth: f32) -> f32 {
    (depth - NEAR) * MAP_DEPTH_RANGE
}

fn mip_level(normal_depth: f32) -> usize {
    ((MIP_FACTOR * normal_depth) as usize).min(MIP_LEVELS - 1)
}

struct RenderableWall {
    vertices: [Vec2f; 4],
    tex_coords: [Vec2f; 2],
    inv_depths: [f32; 2],
}
