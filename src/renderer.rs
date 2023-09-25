use std::collections::VecDeque;

use maths::{
    geometry::{Polygon, Segment, AABB},
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
    aspect_ratio: f32,
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
            aspect_ratio: width as f32 / height as f32,
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

    // Field of view in degrees
    h_fov: f32,
    v_fov: f32,

    // Scaling factors used to convert from camera space to screen space whereby, after applying, the
    // resultant X and Y coordinates can be drawn to the respective location on the screen if within
    // the bounds of the screen.
    focal_width: f32,
    focal_height: f32,
    inv_focal_width: f32,
    inv_focal_height: f32,

    // Number of pixels to shift along Y-axis to simulate pitch (y-shearing)
    pitch_shear: f32,

    // For each X coordinate, defines the lower (inc.) and upper (exc.) bounds of the portal.
    // This is used as the clipping region for the next sector to be rendered within.
    portal_bounds_min: Vec<u16>,
    portal_bounds_max: Vec<u16>,
    new_portal_bounds_min: Vec<u16>,
    new_portal_bounds_max: Vec<u16>,

    // For each X coordinate, defines the lower (inc.) and upper (exc.) bounds of the walls that
    // have been rendered.
    wall_bounds_min: Vec<u16>,
    wall_bounds_max: Vec<u16>,

    // For each X coordinate, stores the depth of the wall that has been rendered on this vertical span.
    depth: Vec<f32>,

    // For each Y coordinate, stores starting X coordinate (inc.) of the horizontal span used to draw the
    // floor/ceiling.
    span_start: Vec<u16>,

    // When drawing floor/ceiling spans, we derive the depth based on its screen space Y coordinate.
    // This is possible because we know the height of the floor/ceiling in camera space (which is constant),
    // so we can reverse the perspective projection to get the depth.
    // Most of this can be precalculated, and is stored here for each screen space Y coordinate. The result
    // can be multiplied by the camera space height of the plane to get the depth.
    focal_height_ratios: Vec<f32>,
}

impl Renderer {
    pub fn new(screen_width: usize, screen_height: usize, h_fov: f32) -> Self {
        let tasks = VecDeque::new();

        let framebuffer = Framebuffer::new(screen_width, screen_height);
        let camera = Camera::new(Vec2f::ZERO, 0.0);

        let v_fov = h_fov / framebuffer.aspect_ratio;

        let (focal_width, focal_height) = focal_dimensions(
            h_fov,
            v_fov,
            framebuffer.half_width,
            framebuffer.half_height,
        );
        let inv_focal_width = 1.0 / focal_width;
        let inv_focal_height = 1.0 / focal_height;

        let pitch_shear = camera.pitch_tan * focal_height;

        let frustum = view_frustum(h_fov);

        let portal_bounds_min = vec![0; screen_width];
        let portal_bounds_max = vec![screen_height as u16; screen_width];
        let new_portal_bounds_min = vec![0; screen_width];
        let new_portal_bounds_max = vec![screen_height as u16; screen_width];
        let wall_bounds_min = vec![0; screen_width];
        let wall_bounds_max = vec![0; screen_width];
        let depth = vec![0.0; screen_width];
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
            inv_focal_width,
            inv_focal_height,
            pitch_shear,

            portal_bounds_min,
            portal_bounds_max,
            new_portal_bounds_min,
            new_portal_bounds_max,

            wall_bounds_min,
            wall_bounds_max,

            depth,

            span_start,

            focal_height_ratios,
        }
    }

    pub fn update(&mut self, player: &Player, textures: &Textures, sectors: &[Sector]) {
        // Only buffer that needs to be cleared is the portal bounds buffer, as they
        // are read from before being overwritten.
        self.portal_bounds_min.fill(0);
        self.portal_bounds_max.fill(self.framebuffer.height as u16);

        // Use player camera
        self.camera = player.camera.clone();
        self.pitch_shear = self.camera.pitch_tan * self.focal_height;

        // Precalculate focal height ratios
        for y in 0..self.framebuffer.height {
            let y_offset = y as f32 - self.framebuffer.half_height - self.pitch_shear;
            if y_offset == 0.0 {
                self.focal_height_ratios[y] = 0.0;
            } else {
                self.focal_height_ratios[y] = self.focal_height / y_offset;
            }
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
        self.v_fov = self.h_fov / self.framebuffer.aspect_ratio;

        (self.focal_width, self.focal_height) = focal_dimensions(
            self.h_fov,
            self.v_fov,
            self.framebuffer.half_width,
            self.framebuffer.half_height,
        );
        self.inv_focal_width = 1.0 / self.focal_width;
        self.inv_focal_height = 1.0 / self.focal_height;

        // No need to recalculate pitch shear and focal height ratios here, as they are initialised
        // at the start of `update`.

        // Width and height have changed, so resize buffers
        self.portal_bounds_min = vec![0; width];
        self.portal_bounds_max = vec![height as u16; width];
        self.new_portal_bounds_min = vec![0; width];
        self.new_portal_bounds_max = vec![height as u16; width];
        self.wall_bounds_min = vec![0; width];
        self.wall_bounds_max = vec![0; width];
        self.depth = vec![0.0; width];
        self.span_start = vec![0; height];
        self.focal_height_ratios = vec![0.0; height];
    }

    pub fn set_fov(&mut self, h_fov: f32) {
        self.h_fov = h_fov;
        self.v_fov = self.h_fov / self.framebuffer.aspect_ratio;

        (self.focal_width, self.focal_height) = focal_dimensions(
            self.h_fov,
            self.v_fov,
            self.framebuffer.half_width,
            self.framebuffer.half_height,
        );
        self.inv_focal_width = 1.0 / self.focal_width;
        self.inv_focal_height = 1.0 / self.focal_height;

        self.frustum = view_frustum(h_fov);

        // No need to recalculate pitch shear and focal height ratios here, as they are initialised
        // at the start of `update`.
    }

    pub fn transform_view(&self, point: Vec2f) -> Vec2f {
        (point - self.camera.position).rotate(self.camera.yaw_sin, self.camera.yaw_cos)
    }

    pub fn project_screen_space(&self, point: Vec2f, height_offset: f32) -> (Vec2f, f32) {
        let z = point.y;
        debug_assert!(z > 0.0); // This should never be zero, as we clip against the near plane
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

        // HACK:
        for x in task.x_min..task.x_max {
            self.new_portal_bounds_min[x] = self.portal_bounds_min[x];
            self.new_portal_bounds_max[x] = self.portal_bounds_max[x];
        }

        for wall in sector.walls.iter() {
            self.draw_wall(sectors, textures, sector, wall, &task);
        }

        // Draw sector ceiling
        self.draw_plane(
            textures.get(sector.ceiling.texture_data.index).unwrap(),
            sector.ceiling.texture_data.offset,
            &sector.ceiling.texture_data.scale_rotate,
            unsafe { &*(&self.portal_bounds_min as *const Vec<u16>) },
            unsafe { &*(&self.wall_bounds_min as *const Vec<u16>) },
            self.camera.height_offset - sector.ceiling.height,
            task.x_min,
            task.x_max,
        );

        // Draw sector floor
        self.draw_plane(
            textures.get(sector.floor.texture_data.index).unwrap(),
            sector.floor.texture_data.offset,
            &sector.floor.texture_data.scale_rotate,
            unsafe { &*(&self.wall_bounds_max as *const Vec<u16>) },
            unsafe { &*(&self.portal_bounds_max as *const Vec<u16>) },
            self.camera.height_offset - sector.floor.height,
            task.x_min,
            task.x_max,
        );

        // HACK:
        for x in task.x_min..task.x_max {
            self.portal_bounds_min[x] = self.new_portal_bounds_min[x];
            self.portal_bounds_max[x] = self.new_portal_bounds_max[x];
        }
    }

    fn draw_wall(
        &mut self,
        sectors: &[Sector],
        textures: &Textures,
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
        tex_coord_a += wall.texture_data.offset;
        tex_coord_b += wall.texture_data.offset;
        tex_coord_a *= wall.texture_data.scale;
        tex_coord_b *= wall.texture_data.scale;

        // Near plane clipping
        // Due to frustum culling, at most one vertex will be clipped. This also means that the divisor
        // will never be zero as the Y coordinates will never be equal.
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

        let inv_depth_a = top_a.1;
        let inv_depth_b = top_b.1;

        // Early out if wall is back-facing
        if top_a.0.x >= top_b.0.x {
            return;
        }

        // Early out if outside of portal bounds
        if top_b.0.x < task.x_min as f32 || top_a.0.x > task.x_max as f32 {
            return;
        }

        // Let's clamp X to the screen space bounds
        let x_min = (top_a.0.x as usize).clamp(task.x_min, task.x_max);
        let x_max = (top_b.0.x as usize).clamp(task.x_min, task.x_max);

        let x_delta = top_b.0.x - top_a.0.x;
        debug_assert!(x_delta > 0.0); // This should never be zero, as we cull back-facing walls
        let inv_x_delta = 1.0 / x_delta;

        if let Some(portal) = wall.portal {
            let neighbour_top_a =
                self.project_screen_space(vs_a, sectors[portal.sector].ceiling.height);
            let neighbour_top_b =
                self.project_screen_space(vs_b, sectors[portal.sector].ceiling.height);
            let neighbour_bottom_a =
                self.project_screen_space(vs_a, sectors[portal.sector].floor.height);
            let neighbour_bottom_b =
                self.project_screen_space(vs_b, sectors[portal.sector].floor.height);

            let mut upper_tex_coord_a = Vec2f::new(0.0, 0.0);
            let mut upper_tex_coord_b = Vec2f::new(
                wall.width,
                sector.ceiling.height - sectors[portal.sector].ceiling.height,
            );
            upper_tex_coord_a += portal.upper_texture.offset;
            upper_tex_coord_b += portal.upper_texture.offset;
            upper_tex_coord_a *= portal.upper_texture.scale;
            upper_tex_coord_b *= portal.upper_texture.scale;

            let mut lower_tex_coord_a = Vec2f::new(0.0, 0.0);
            let mut lower_tex_coord_b = Vec2f::new(
                wall.width,
                sectors[portal.sector].floor.height - sector.floor.height,
            );
            lower_tex_coord_a += portal.lower_texture.offset;
            lower_tex_coord_b += portal.lower_texture.offset;
            lower_tex_coord_a *= portal.lower_texture.scale;
            lower_tex_coord_b *= portal.lower_texture.scale;

            // Rasterise portal wall
            let upper_wall_lerp = WallInterpolator::new(
                top_a.0,
                top_b.0,
                neighbour_top_a.0,
                neighbour_top_b.0,
                upper_tex_coord_a,
                upper_tex_coord_b,
                inv_depth_a,
                inv_depth_b,
                x_min as f32,
                inv_x_delta,
            );
            let lower_wall_lerp = WallInterpolator::new(
                neighbour_bottom_a.0,
                neighbour_bottom_b.0,
                bottom_a.0,
                bottom_b.0,
                lower_tex_coord_a,
                lower_tex_coord_b,
                inv_depth_a,
                inv_depth_b,
                x_min as f32,
                inv_x_delta,
            );

            let upper_texture = textures.get(portal.upper_texture.index).unwrap();
            let lower_texture = textures.get(portal.lower_texture.index).unwrap();

            // TODO:
            // Upper and lower walls of portal can have different textures, thus also texture coordinates.
            self.rasterise_portal(
                upper_wall_lerp,
                lower_wall_lerp,
                upper_texture,
                lower_texture,
                x_min,
                x_max,
            );

            self.tasks.push_back(RenderTask {
                sector_index: portal.sector,
                x_min,
                x_max,
            })
        } else {
            let wall_lerp = WallInterpolator::new(
                top_a.0,
                top_b.0,
                bottom_a.0,
                bottom_b.0,
                tex_coord_a,
                tex_coord_b,
                inv_depth_a,
                inv_depth_b,
                x_min as f32,
                inv_x_delta,
            );

            let texture = textures.get(wall.texture_data.index).unwrap();

            self.rasterise_wall(wall_lerp, texture, x_min, x_max);
        }
    }

    fn rasterise_wall(
        &mut self,
        mut wall: WallInterpolator,
        texture: &Texture,
        x_min: usize,
        x_max: usize,
    ) {
        // Draw wall, one column at a time
        for x in x_min..x_max {
            let min_portal_bound = self.portal_bounds_min[x] as usize;
            let max_portal_bound = self.portal_bounds_max[x] as usize;

            // Clamp Y to the screen portal bounds
            let y_min = (wall.top_y as usize).clamp(min_portal_bound, max_portal_bound);
            let y_max = (wall.bottom_y as usize).clamp(min_portal_bound, max_portal_bound);

            // TODO: I don't actually think we need to clamp Y here, as this sort of thing is only
            // necessary for portal rasterisation due to the way they interact with the next sector,
            // potentially causing strangeness.

            // Ensure that max >= min using min as boundary (for no perticular reason)
            // let y_max = y_max.max(y_min);

            self.rasterise_wall_span(&mut wall, texture, x, y_min, y_max);

            self.wall_bounds_min[x] = y_min as u16;
            self.wall_bounds_max[x] = y_max as u16;

            wall.step_x();
        }
    }

    fn rasterise_portal(
        &mut self,
        mut upper_wall: WallInterpolator,
        mut lower_wall: WallInterpolator,
        upper_texture: &Texture,
        lower_texture: &Texture,
        x_min: usize,
        x_max: usize,
    ) {
        // Draw wall, one column at a time
        for x in x_min..x_max {
            let min_portal_bound = self.portal_bounds_min[x] as usize;
            let max_portal_bound = self.portal_bounds_max[x] as usize;

            let upper_y_min = (upper_wall.top_y as usize).clamp(min_portal_bound, max_portal_bound);
            let upper_y_max =
                (upper_wall.bottom_y as usize).clamp(min_portal_bound, max_portal_bound);
            // Ensure that max >= min using min as boundary (to bias towards upper portal)
            let upper_y_max = upper_y_max.max(upper_y_min);

            let lower_y_max =
                (lower_wall.bottom_y as usize).clamp(min_portal_bound, max_portal_bound);
            let lower_y_min = (lower_wall.top_y as usize).clamp(min_portal_bound, max_portal_bound);
            // Ensure that min <= max using max as boundary (to bias towards lower portal)
            let lower_y_min = lower_y_min.min(lower_y_max);

            // Ensure that lower portal is always below upper portal
            let lower_y_max = lower_y_max.max(upper_y_max);
            let lower_y_min = lower_y_min.max(upper_y_max);

            self.rasterise_wall_span(&mut upper_wall, upper_texture, x, upper_y_min, upper_y_max);
            self.rasterise_wall_span(&mut lower_wall, lower_texture, x, lower_y_min, lower_y_max);

            self.wall_bounds_min[x] = upper_y_min as u16;
            self.wall_bounds_max[x] = lower_y_max as u16;

            self.new_portal_bounds_min[x] = upper_y_max as u16;
            self.new_portal_bounds_max[x] = lower_y_min as u16;

            upper_wall.step_x();
            lower_wall.step_x();
        }
    }

    fn rasterise_wall_span(
        &mut self,
        wall: &mut WallInterpolator,
        texture: &Texture,
        x: usize,
        y_min: usize,
        y_max: usize,
    ) {
        wall.init_y(y_min);

        let depth = 1.0 / wall.inv_depth;
        let normal_depth = normalise_depth(depth);
        // TODO: Bias mip level based on surface angle?
        let mip_level = mip_level(normal_depth, 0.0);
        let mip_scale = MIP_SCALES[mip_level];

        // Store depth for this vertical span
        self.depth[x] = depth;

        // Recover U texture coordinate after interpolating in depth space
        let u = wall.u_depth * depth;

        let width_mask = texture.levels[mip_level].width - 1;
        let height_mask = texture.levels[mip_level].height - 1;

        let texture_x = unsafe { (u * mip_scale).to_int_unchecked::<usize>() } & width_mask;

        for y in y_min..y_max {
            let texture_y =
                unsafe { (wall.v * mip_scale).to_int_unchecked::<usize>() } & height_mask;

            let colour = texture.sample(texture_x, texture_y, mip_level);
            self.framebuffer.set_pixel(x, y, colour);

            wall.step_y();
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
        // Generally, min is inclusive and max is exclusive

        // Portal and wall bounds are collected during rasterisation of walls. We can use these to
        // draw floors and ceilings horizontally, which allows for fewer depth calculations as
        // depth is constant horizontally.

        // These indicate the current Y range of horizontal lines that are 'open'. The starting
        // X coordinate of each open line is stored in `span_start`. At the point they are found to be
        // 'closed', the span is drawn. It can be considered closed when outside of the bounds.
        let mut y_min = min_bounds[x_min];
        let mut y_max = y_min;

        for x in x_min..x_max {
            let min_bound = min_bounds[x];
            let max_bound = max_bounds[x];

            // Bounds have widened, so mark new lines as 'open'
            while min_bound < y_min {
                y_min -= 1;
                self.span_start[y_min as usize] = x as u16;
            }
            while max_bound > y_max {
                self.span_start[y_max as usize] = x as u16;
                y_max += 1;
            }

            // Bounds have narrowed, so draw any horizontal lines that are now 'closed'
            while min_bound > y_min {
                self.rasterise_plane_span(
                    texture,
                    texture_offset,
                    texture_scale_rotate,
                    height_offset,
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
        y: usize,
        x_min: usize,
        x_max: usize,
    ) {
        // This shouldn't be possible, when called from `draw_plane`, but I think sometimes
        // `x_min` and `x_max` can be equal, which is fine as it will just draw nothing.
        // However, if `x_min` is greater than `x_max`, this is indicative of a bug.
        debug_assert!(
            x_min <= x_max,
            "x_max is greater than x_min :: x_min: {}, x_max: {}",
            x_min,
            x_max
        );
        if x_max <= x_min {
            return;
        }

        let focal_height_ratio = self.focal_height_ratios[y];
        let depth = focal_height_ratio * height_offset;
        let normal_depth = normalise_depth(depth);
        let mip_level = mip_level(normal_depth, 6.0);
        let mip_scale = MIP_SCALES[mip_level];

        // Calculate world space coordinates of either end of the span, via reversing the perspective
        // projection, and use these as the texture coordinates.
        let ws_1 = Vec2f::new(
            ((x_min as f32 - self.framebuffer.half_width) * depth) * self.inv_focal_width,
            -depth,
        )
        .rotate(self.camera.yaw_sin, self.camera.yaw_cos)
            + Vec2f::new(self.camera.position.x, -self.camera.position.y);

        let ws_2 = Vec2f::new(
            ((x_max as f32 - self.framebuffer.half_width) * depth) * self.inv_focal_width,
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

        let width_mask = texture.levels[mip_level].width - 1;
        let height_mask = texture.levels[mip_level].height - 1;

        for x in x_min..x_max {
            // U and V are in world space, thus could be negative. We would need to `abs` these,
            // but an unchecked cast works too.
            let texture_x = unsafe { u.to_int_unchecked::<usize>() } & width_mask;
            let texture_y = unsafe { v.to_int_unchecked::<usize>() } & height_mask;

            let colour = texture.sample(texture_x, texture_y, mip_level);
            self.framebuffer.set_pixel(x, y, colour);

            u += u_m;
            v += v_m;
        }
    }
}

/// Map a linear depth value, ranging from [NEAR] to [FAR], to a normalised depth value, ranging from 0.0 to 1.0.
fn normalise_depth(depth: f32) -> f32 {
    (depth - NEAR) * MAP_DEPTH_RANGE
}

/// Calculates an appropriate mip level based on the normalised depth and a bias.
fn mip_level(normal_depth: f32, bias: f32) -> usize {
    (((MIP_FACTOR + bias) * normal_depth) as usize).min(MIP_LEVELS - 1)
}

/// This is used during perspective projection to convert from camera space to screen space.
/// It is essentially a scaling factor that is used to get a pixel coordinate from a
/// coordinate in camera space, taking into account the field of view and screen size.
fn focal_dimensions(h_fov: f32, v_fov: f32, half_width: f32, half_height: f32) -> (f32, f32) {
    // Use similar triangles to calculate focal width/height, based on the screen we are
    // projecting onto and the field of view.
    let focal_width = half_width / (h_fov * 0.5).to_radians().tan();
    let focal_height = half_height / (v_fov * 0.5).to_radians().tan();

    (focal_width, focal_height)
}

/// Returns a polygon representing the view frustum, based on the given horizontal field of view.
fn view_frustum(h_fov: f32) -> Polygon {
    let tan = (h_fov * 0.5).to_radians().tan();
    let opp_far = FAR * tan;
    let opp_near = NEAR * tan;

    Polygon::from_vertices(vec![
        Vec2f::new(-opp_far, FAR),
        Vec2f::new(opp_far, FAR),
        Vec2f::new(opp_near, NEAR),
        Vec2f::new(-opp_near, NEAR),
    ])
}

struct WallInterpolator {
    inv_depth: f32,
    top_y: f32,
    bottom_y: f32,
    u_depth: f32,
    inv_depth_m: f32,
    top_y_m: f32,
    bottom_y_m: f32,
    u_depth_m: f32,

    v_start: f32,
    v_delta: f32,
    v: f32,
    v_m: f32,
}

impl WallInterpolator {
    fn new(
        top_a: Vec2f,
        top_b: Vec2f,
        bottom_a: Vec2f,
        bottom_b: Vec2f,
        tex_coord_a: Vec2f,
        tex_coord_b: Vec2f,
        inv_depth_a: f32,
        inv_depth_b: f32,
        x_min: f32,
        inv_x_delta: f32,
    ) -> Self {
        // Divide texture coordinates by depth to account for perspective projection during interpolation.
        // After interpolation, multiply by depth to recover the original texture coordinates.
        // For walls, we only need to do this for the X coordinate, as the Y coordinate has constant depth.
        let u_depth_a = tex_coord_a.x * inv_depth_a;
        let u_depth_b = tex_coord_b.x * inv_depth_b;

        // Gradients with respect to X
        let inv_depth_m = (inv_depth_b - inv_depth_a) * inv_x_delta;
        let top_y_m = (top_b.y - top_a.y) * inv_x_delta;
        let bottom_y_m = (bottom_b.y - bottom_a.y) * inv_x_delta;
        let u_depth_m = (u_depth_b - u_depth_a) * inv_x_delta;

        // Interpolator start values
        let mut inv_depth = inv_depth_a;
        let mut top_y = top_a.y;
        let mut bottom_y = bottom_a.y;
        let mut u_depth = u_depth_a;

        // X offset caused by clamping vertices to screen space bounds
        let x_clamp_offset = x_min - top_a.x;

        // We need to update our interpolators to account difference in X caused by clamping
        inv_depth += inv_depth_m * x_clamp_offset;
        top_y += top_y_m * x_clamp_offset;
        bottom_y += bottom_y_m * x_clamp_offset;
        u_depth += u_depth_m * x_clamp_offset;

        let v_start = tex_coord_a.y;
        let v_delta = tex_coord_b.y - tex_coord_a.y;
        let v = 0.0;
        let v_m = 0.0;

        Self {
            inv_depth,
            top_y,
            bottom_y,
            u_depth,
            inv_depth_m,
            top_y_m,
            bottom_y_m,
            u_depth_m,

            v_start,
            v_delta,
            v,
            v_m,
        }
    }

    fn init_y(&mut self, y_min: usize) {
        // Y offset caused by clamping vertices to screen space bounds
        let y_clamp_offset = y_min as f32 - self.top_y;

        // Gradient with respect to Y
        self.v_m = self.v_delta / (self.bottom_y - self.top_y);

        // Interpolator start value
        self.v = self.v_start;

        // Account for difference caused by clamping Y
        self.v += self.v_m * y_clamp_offset;
    }

    fn step_x(&mut self) {
        self.inv_depth += self.inv_depth_m;
        self.top_y += self.top_y_m;
        self.bottom_y += self.bottom_y_m;
        self.u_depth += self.u_depth_m;
    }

    fn step_y(&mut self) {
        self.v += self.v_m;
    }
}
