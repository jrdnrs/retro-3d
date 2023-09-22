use std::collections::VecDeque;

use maths::{
    geometry::{Polygon, Segment},
    linear::Vec2f,
};

use crate::{
    app::{FAR, MAP_DEPTH_RANGE, NEAR},
    camera::Camera,
    colour::Colour,
    player::Player,
    surface::{Basis, Sector, Wall},
    textures::{Texture, Textures, MIP_FACTOR, MIP_LEVELS, MIP_SCALES},
};

struct Framebuffer {
    width: usize,
    height: usize,
    half_width: f32,
    half_height: f32,
    pixels: Vec<u32>,
}

impl Framebuffer {
    pub fn new(width: usize, height: usize) -> Self {
        let len = width * height;
        let pixels = vec![0; len];

        Self {
            width,
            height,
            half_width: width as f32 * 0.5,
            half_height: height as f32 * 0.5,
            pixels,
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

    framebuffer: Framebuffer,
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

    pub fn get_pixels(&self) -> &[u32] {
        &self.framebuffer.pixels
    }

    pub fn update(&mut self, player: &Player, textures: &Textures, sectors: &[Sector]) {
        // reset buffers
        // self.clear_colour(Colour::CYAN);
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

    pub fn clear_colour(&mut self, colour: Colour) {
        for pixel in self.framebuffer.pixels.iter_mut() {
            *pixel = colour.rgb;
        }
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, colour: Colour) {
        debug_assert!(x < self.framebuffer.width && y < self.framebuffer.height);
        let index = (y * self.framebuffer.width) + x;

        self.framebuffer.pixels[index] = colour.rgb;
    }

    pub fn blend_pixel(&mut self, x: usize, y: usize, colour: Colour, alpha: u8) {
        debug_assert!(x < self.framebuffer.width && y < self.framebuffer.height);
        let index = (y * self.framebuffer.width) + x;

        let blended = Colour::new(self.framebuffer.pixels[index]).blend(colour, alpha);
        self.framebuffer.pixels[index] = blended.rgb;
    }

    pub fn draw_line(&mut self, mut start: Vec2f, mut end: Vec2f, colour: Colour) {
        start.x = start.x.clamp(0.0, (self.framebuffer.width - 1) as f32);
        start.y = start.y.clamp(0.0, (self.framebuffer.height - 1) as f32);
        end.x = end.x.clamp(0.0, (self.framebuffer.width - 1) as f32);
        end.y = end.y.clamp(0.0, (self.framebuffer.height - 1) as f32);

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
            unsafe { &*(&self.portal_bounds_min as *const Vec<u16>) },
            unsafe { &*(&self.wall_bounds_min as *const Vec<u16>) },
            self.camera.height_offset - sector.ceiling.height,
            task.x_min,
            task.x_max,
        );

        // Draw sector floor
        self.draw_plane(
            textures.get(sector.floor.texture_index).unwrap(),
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

        // TEMP:
        // Testing texture coordinates based on world space position
        let mut tex_coord_a = wall.texture_basis.apply(Vec2f::new(0.0, 0.0));
        let mut tex_coord_b = wall.texture_basis.apply(Vec2f::new(
            (wall.b - wall.a).magnitude(),
            sector.ceiling.height - sector.floor.height,
        ));

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
                self.set_pixel(x, y, colour);

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

        let tex_coord_a = ws_1 * mip_scale;
        let tex_coord_b = ws_2 * mip_scale;

        let inv_x_delta = 1.0 / (x_max - x_min) as f32;

        let v_m = (tex_coord_b.y - tex_coord_a.y) * inv_x_delta;
        let mut v = tex_coord_a.y;

        let u_m = (tex_coord_b.x - tex_coord_a.x) * inv_x_delta;
        let mut u = tex_coord_a.x;

        for x in x_min..x_max {
            let texture_x = u.abs() as usize & (texture.levels[mip_level].width - 1);
            let texture_y = v.abs() as usize & (texture.levels[mip_level].height - 1);

            let colour = texture.sample(texture_x, texture_y, mip_level);

            self.set_pixel(x, y, colour);

            u += u_m;
            v += v_m;
        }
    }

    // pub fn draw_mesh(&mut self, mesh: &Mesh, texture: &Texture) {
    //     for index in (0..mesh.indices.len()).step_by(3) {
    //         let a = mesh.vertices[mesh.indices[index] as usize];
    //         let b = mesh.vertices[mesh.indices[index + 1] as usize];
    //         let c = mesh.vertices[mesh.indices[index + 2] as usize];

    //         self.draw_triangle(
    //             &Triangle {
    //                 a,
    //                 b,
    //                 c,
    //                 height_offset: mesh.height_offset,
    //                 colour: mesh.colour,
    //                 texture_index: mesh.texture_index,
    //             },
    //             texture,
    //         );
    //     }
    // }

    // pub fn draw_triangle(&mut self, triangle: &Triangle, texture: &Texture) {
    //     // World to view space
    //     let vertices = [
    //         self.world_to_view_space(&triangle.a),
    //         self.world_to_view_space(&triangle.b),
    //         self.world_to_view_space(&triangle.c),
    //     ];

    //     // Frustum culling
    //     if !self.frustum.overlaps(&maths::geometry::Triangle::new(
    //         vertices[0].position,
    //         vertices[1].position,
    //         vertices[2].position,
    //     )) {
    //         return;
    //     }

    //     // Near plane clipping
    //     let mut inside = StackVec::<usize, 3>::new();
    //     let mut outside = StackVec::<usize, 2>::new();
    //     for i in 0..3 {
    //         if vertices[i].position.y < NEAR {
    //             outside.push(i);
    //         } else {
    //             inside.push(i);
    //         }
    //     }

    //     // triangle may be split due to clipping
    //     let mut clipped_vertices = StackVec::<[Vertex; 3], 2>::new();

    //     match outside.len() {
    //         // No points outside means the triangle is completely visible
    //         0 => {
    //             clipped_vertices.push(vertices);
    //         }

    //         // One point outside means the triangle is split into two
    //         1 => {
    //             let inside_1 = vertices[inside[0]];
    //             let inside_2 = vertices[inside[1]];
    //             let outside = vertices[outside[0]];

    //             let y_offset = NEAR - outside.position.y;
    //             let t1 = y_offset / (inside_1.position.y - outside.position.y);
    //             let t2 = y_offset / (inside_2.position.y - outside.position.y);

    //             let vertex1 = outside.lerp(inside_1, t1);
    //             let vertex2 = outside.lerp(inside_2, t2);

    //             clipped_vertices.push([inside_1, vertex2, vertex1]);
    //             clipped_vertices.push([inside_1, inside_2, vertex2]);
    //         }

    //         // Two points outside means we can reconstruct the single triangle
    //         2 => {
    //             let outside_1 = vertices[outside[0]];
    //             let outside_2 = vertices[outside[1]];
    //             let inside = vertices[inside[0]];

    //             let y_offset_1 = NEAR - outside_1.position.y;
    //             let y_offset_2 = NEAR - outside_2.position.y;
    //             let t1 = y_offset_1 / (inside.position.y - outside_1.position.y);
    //             let t2 = y_offset_2 / (inside.position.y - outside_2.position.y);

    //             let vertex1 = outside_1.lerp(inside, t1);
    //             let vertex2 = outside_2.lerp(inside, t2);

    //             clipped_vertices.push([inside, vertex2, vertex1]);
    //         }

    //         // Three points outside shouldn't be possible due to frustum culling
    //         _ => unreachable!(),
    //     }

    //     // Perspective projection into screen space
    //     for [a, b, c] in clipped_vertices.iter_mut() {
    //         *a = self.view_to_screen_space(a, -triangle.height_offset);
    //         *b = self.view_to_screen_space(b, -triangle.height_offset);
    //         *c = self.view_to_screen_space(c, -triangle.height_offset);

    //         // We need to scale the texture coordinates by the inverse depth so we can interpolate
    //         // them correctly in screen space
    //         a.tex_coords *= a.inv_depth;
    //         b.tex_coords *= b.inv_depth;
    //         c.tex_coords *= c.inv_depth;

    //         // Order vertices from top to bottom
    //         if b.position.y < a.position.y {
    //             core::mem::swap(a, b);
    //         }
    //         if c.position.y < b.position.y {
    //             core::mem::swap(b, c);
    //         }
    //         if b.position.y < a.position.y {
    //             core::mem::swap(a, b);
    //         }
    //     }

    //     for vertices in clipped_vertices.iter() {
    //         // Inverse is precalculated for performance
    //         let inv_delta_y = [
    //             1.0 / (vertices[1].position.y - vertices[0].position.y),
    //             1.0 / (vertices[2].position.y - vertices[1].position.y),
    //             1.0 / (vertices[2].position.y - vertices[0].position.y),
    //         ];

    //         // Calculate the gradients for each edge
    //         let x_m = [
    //             (vertices[1].position.x - vertices[0].position.x) * inv_delta_y[0],
    //             (vertices[2].position.x - vertices[1].position.x) * inv_delta_y[1],
    //             (vertices[2].position.x - vertices[0].position.x) * inv_delta_y[2],
    //         ];

    //         let u_m = [
    //             (vertices[1].tex_coords.x - vertices[0].tex_coords.x) * inv_delta_y[0],
    //             (vertices[2].tex_coords.x - vertices[1].tex_coords.x) * inv_delta_y[1],
    //             (vertices[2].tex_coords.x - vertices[0].tex_coords.x) * inv_delta_y[2],
    //         ];

    //         let v_m = [
    //             (vertices[1].tex_coords.y - vertices[0].tex_coords.y) * inv_delta_y[0],
    //             (vertices[2].tex_coords.y - vertices[1].tex_coords.y) * inv_delta_y[1],
    //             (vertices[2].tex_coords.y - vertices[0].tex_coords.y) * inv_delta_y[2],
    //         ];

    //         // We only need to interpolate depth from the top-most vertex to the bottom to get changes across
    //         // the Y axis, as depth is constant horizontally
    //         let inv_depth_m = (vertices[2].inv_depth - vertices[0].inv_depth) * inv_delta_y[2];

    //         // We will first be interpolating from the top-most point (a) and diverge into two lines
    //         // (a to b) and (a to c). We just need to figure out which side is left and which is right.
    //         let (left, right) = if x_m[0] < x_m[2] { (0, 2) } else { (2, 0) };

    //         let mut left_x_m = x_m[left];
    //         let mut right_x_m = x_m[right];
    //         let mut left_u_m = u_m[left];
    //         let mut right_u_m = u_m[right];
    //         let mut left_v_m = v_m[left];
    //         let mut right_v_m = v_m[right];

    //         // These are the interpolators for the left and right sides of the triangle
    //         let mut x1 = vertices[0].position.x;
    //         let mut x2 = vertices[0].position.x;
    //         let mut u1 = vertices[0].tex_coords.x;
    //         let mut u2 = vertices[0].tex_coords.x;
    //         let mut v1 = vertices[0].tex_coords.y;
    //         let mut v2 = vertices[0].tex_coords.y;
    //         let mut inv_depth = vertices[0].inv_depth;

    //         // These are the three points that we will be interpolating between
    //         let y1_clamp = (vertices[0].position.y as usize).clamp(0, self.framebuffer.height);
    //         let y2_clamp = (vertices[1].position.y as usize).clamp(0, self.framebuffer.height);
    //         let y3_clamp = (vertices[2].position.y as usize).clamp(0, self.framebuffer.height);

    //         // Potential offset caused by clamping
    //         let y1_offset = y1_clamp as f32 - vertices[0].position.y;

    //         // Step interpolators for the difference caused by clamping
    //         x1 += left_x_m * y1_offset;
    //         x2 += right_x_m * y1_offset;
    //         u1 += left_u_m * y1_offset;
    //         u2 += right_u_m * y1_offset;
    //         v1 += left_v_m * y1_offset;
    //         v2 += right_v_m * y1_offset;
    //         inv_depth += inv_depth_m * y1_offset;

    //         for y in y1_clamp..y2_clamp {
    //             // We now need to inerpolate across horizontally, using the interpolated values for each edge
    //             let mut u = u1;
    //             let mut v = v1;
    //             let inv_delta_x = 1.0 / (x2 - x1);
    //             let u_m = (u2 - u1) * inv_delta_x;
    //             let v_m = (v2 - v1) * inv_delta_x;

    //             // Interpolate between these two points
    //             let x1_clamp = (x1 as usize).clamp(0, self.framebuffer.width);
    //             let x2_clamp = (x2 as usize).clamp(0, self.framebuffer.width);

    //             // Potential offset caused by clamping
    //             let x1_offset = x1_clamp as f32 - x1;

    //             // Step interpolators for the difference caused by clamping
    //             u += u_m * x1_offset;
    //             v += v_m * x1_offset;

    //             let linear_depth = 1.0 / inv_depth;

    //             for x in x1_clamp..x2_clamp {
    //                 let texture_x = (u * linear_depth * texture.width as f32) as usize;
    //                 let texture_y = (v * linear_depth * texture.height as f32) as usize;

    //                 let texture_index = texture_y * texture.width + texture_x;
    //                 let colour = Colour::new(texture.pixels[texture_index]);

    //                 self.set_pixel(x, y, colour, linear_depth);

    //                 // Increment interpolators between edges
    //                 u += u_m;
    //                 v += v_m;
    //             }

    //             // Increment these interpolators across the respective edges
    //             x1 += left_x_m;
    //             x2 += right_x_m;
    //             u1 += left_u_m;
    //             u2 += right_u_m;
    //             v1 += left_v_m;
    //             v2 += right_v_m;
    //             inv_depth += inv_depth_m;
    //         }

    //         let y2_offset = y2_clamp as f32 - vertices[1].position.y;

    //         // TODO: Consider placing interpolators/gradients in an array so we can swap edges
    //         // via an index that we compute earlier

    //         // We are now halfway through the triangle, so either the left or right side will be
    //         // the same as the first half, but the other will be changed to the third edge.
    //         // This means one half will need new interpolators, and the other will need to continue
    //         if left == 0 {
    //             // The left side is the 'short' one (in terms of Y), so it will change to the next 'short' edge
    //             x1 = vertices[1].position.x;
    //             u1 = vertices[1].tex_coords.x;
    //             v1 = vertices[1].tex_coords.y;

    //             left_x_m = x_m[1];
    //             left_u_m = u_m[1];
    //             left_v_m = v_m[1];

    //             x1 += left_x_m * y2_offset;
    //             u1 += left_u_m * y2_offset;
    //             v1 += left_v_m * y2_offset;
    //         } else {
    //             // The right side is the 'short' one (in terms of Y), so it will change to the next 'short' edge
    //             x2 = vertices[1].position.x;
    //             u2 = vertices[1].tex_coords.x;
    //             v2 = vertices[1].tex_coords.y;

    //             right_x_m = x_m[1];
    //             right_u_m = u_m[1];
    //             right_v_m = v_m[1];

    //             x2 += right_x_m * y2_offset;
    //             u2 += right_u_m * y2_offset;
    //             v2 += right_v_m * y2_offset;
    //         }

    //         // TODO: This loop is almost identical to the first one, so we should try to pull it out into a function

    //         for y in y2_clamp..y3_clamp {
    //             // We now need to inerpolate across horizontally, using the interpolated values for each edge
    //             let mut u = u1;
    //             let mut v = v1;
    //             let inv_delta_x = 1.0 / (x2 - x1);
    //             let u_m = (u2 - u1) * inv_delta_x;
    //             let v_m = (v2 - v1) * inv_delta_x;

    //             // Interpolate between these two points
    //             let x1_clamp = (x1 as usize).clamp(0, self.framebuffer.width);
    //             let x2_clamp = (x2 as usize).clamp(0, self.framebuffer.width);

    //             // Potential offset caused by clamping
    //             let x1_offset = x1_clamp as f32 - x1;

    //             // Step interpolators for the difference caused by clamping
    //             u += u_m * x1_offset;
    //             v += v_m * x1_offset;

    //             let linear_depth = 1.0 / inv_depth;

    //             for x in x1_clamp..x2_clamp {
    //                 let texture_x = ((u % 1.0) * linear_depth * texture.width as f32) as usize;
    //                 let texture_y = ((v % 1.0) * linear_depth * texture.height as f32) as usize;

    //                 let texture_index = texture_y * texture.width + texture_x;
    //                 let colour = Colour::new(texture.pixels[texture_index]);

    //                 self.set_pixel(x, y, colour, linear_depth);

    //                 // Increment interpolators between edges
    //                 u += u_m;
    //                 v += v_m;
    //             }

    //             // Increment these interpolators across the respective edges
    //             x1 += left_x_m;
    //             x2 += right_x_m;
    //             u1 += left_u_m;
    //             u2 += right_u_m;
    //             v1 += left_v_m;
    //             v2 += right_v_m;
    //             inv_depth += inv_depth_m;
    //         }
    //     }

    //     // barycentric rasterization
    //     // for triangle in triangles.iter() {
    //     //     let geo_triangle = maths::geometry::Triangle::new(
    //     //         triangle.a.position,
    //     //         triangle.b.position,
    //     //         triangle.c.position,
    //     //     );
    //     //     let bounds = geo_triangle.extents();
    //     //     let inv_area = 1.0 / geo_triangle.area();

    //     //     let x1 = (bounds.min.x.ceil() as usize).clamp(0, self.framebuffer.width);
    //     //     let x2 = (bounds.max.x.ceil() as usize).clamp(0, self.framebuffer.width);
    //     //     let y1 = (bounds.min.y.ceil() as usize).clamp(0, self.framebuffer.height);
    //     //     let y2 = (bounds.max.y.ceil() as usize).clamp(0, self.framebuffer.height);

    //     //     for y in y1..y2 {
    //     //         for x in x1..x2 {
    //     //             let point = Vec2f::new(x as f32, y as f32);

    //     //             if !geo_triangle.contains_point(point) {
    //     //                 continue;
    //     //             }

    //     //             let barycentric = geo_triangle.barycentric_with_inv_area(point, inv_area);

    //     //             let vertex = triangle.a * barycentric.x
    //     //                 + triangle.b * barycentric.y
    //     //                 + triangle.c * barycentric.z;

    //     //             let linear_depth = 1.0 / vertex.inv_depth;

    //     //             self.set_pixel(x, y, triangle.colour, linear_depth);
    //     //         }
    //     //     }
    //     // }
    // }
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
