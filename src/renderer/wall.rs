use maths::{geometry::Segment, linear::Vec2f};

use crate::{
    consts::{MIP_SCALES, NEAR},
    surface::{Sector, Wall},
    textures::Texture,
};

use super::{
    portal::{PortalNode, PortalTree},
    util::{diminish_lighting, mip_level, normalise_depth},
    RendererState,
};

pub struct WallRenderer {
    /// For each X coordinate, defines the lower (inc.) Y coordinate of the walls that have been rendered.
    wall_bounds_min: Vec<u16>,
    /// For each X coordinate, defines the upper (exc.) Y coordinate of the walls that have been rendered.
    wall_bounds_max: Vec<u16>,
}

impl WallRenderer {
    pub fn new(state: &RendererState) -> Self {
        Self {
            wall_bounds_min: vec![0; state.framebuffer.width()],
            wall_bounds_max: vec![state.framebuffer.height() as u16; state.framebuffer.width()],
        }
    }

    pub fn set_viewport(&mut self, state: &RendererState) {
        self.wall_bounds_min.resize(state.framebuffer.width(), 0);
        self.wall_bounds_max
            .resize(state.framebuffer.width(), state.framebuffer.height() as u16);
    }

    pub fn update(&mut self, state: &RendererState) {}

    pub fn get_wall_bounds(&self) -> (&[u16], &[u16]) {
        (&self.wall_bounds_min, &self.wall_bounds_max)
    }

    pub fn render(
        &mut self,
        state: &mut RendererState,
        portals: &mut PortalTree,
        sectors: &[Sector],
        textures: &[Texture],
        portal_index: usize,
        wall: &Wall,
    ) {
        if let Some(wall_portal_data) = wall.portal {
            let sector_index = portals.nodes[portal_index].sector_index;
            let sector = &sectors[sector_index];
            let next_sector = &sectors[wall_portal_data.sector];

            let upper_texture = textures.get(wall_portal_data.upper_texture.index).unwrap();
            let lower_texture = textures.get(wall_portal_data.lower_texture.index).unwrap();

            self.draw_portal_wall(
                state,
                portals,
                portal_index,
                sector,
                next_sector,
                wall,
                upper_texture,
                lower_texture,
            );
        } else {
            let portal = portals.nodes.get_mut(portal_index).unwrap();
            let sector = &sectors[portal.sector_index];
            let texture = textures.get(wall.texture_data.index).unwrap();
            let y_bounds = (
                portals.portal_bounds_min[portal.tree_depth].as_slice(),
                portals.portal_bounds_max[portal.tree_depth].as_slice(),
            );

            self.draw_wall(state, portal, y_bounds, sector, wall, texture);
        }
    }

    pub fn draw_wall(
        &mut self,
        state: &mut RendererState,
        portal: &mut PortalNode,
        y_bounds: (&[u16], &[u16]),
        sector: &Sector,
        wall: &Wall,
        texture: &Texture,
    ) {
        // Transform coordinates based on camera position and orientation
        let mut vs_a = state.transform_view(wall.segment.a);
        let mut vs_b = state.transform_view(wall.segment.b);

        // Frustum culling
        if !Segment::new(vs_a, vs_b).overlaps_polygon(&state.frustum) {
            return;
        }

        // TODO: Consider precalculating these values, but we must then make sure to update
        // texture coordinates for all walls in a sector when the sector's ceiling/floor height changes.
        let mut tex_a = Vec2f::new(0.0, 0.0);
        let mut tex_b = Vec2f::new(wall.width, sector.ceiling.height - sector.floor.height);
        tex_a += wall.texture_data.offset;
        tex_b += wall.texture_data.offset;
        tex_a *= wall.texture_data.scale;
        tex_b *= wall.texture_data.scale;

        // Near plane clipping
        if vs_a.y < NEAR {
            let t = (NEAR - vs_a.y) / (vs_b.y - vs_a.y);

            vs_a.y = NEAR;
            vs_a.x += (vs_b.x - vs_a.x) * t;
            tex_a.x += (tex_b.x - tex_a.x) * t;
        } else if vs_b.y < NEAR {
            let t = (NEAR - vs_b.y) / (vs_a.y - vs_b.y);

            vs_b.y = NEAR;
            vs_b.x += (vs_a.x - vs_b.x) * t;
            tex_b.x += (tex_a.x - tex_b.x) * t;
        }

        // Perspective projection into screen space
        let top_a = state.project_screen_space(vs_a, sector.ceiling.height);
        let top_b = state.project_screen_space(vs_b, sector.ceiling.height);
        let bottom_a = state.project_screen_space(vs_a, sector.floor.height);
        let bottom_b = state.project_screen_space(vs_b, sector.floor.height);

        let inv_depth_a = top_a.1;
        let inv_depth_b = top_b.1;

        // Early out if wall is back-facing
        if top_a.0.x >= top_b.0.x {
            return;
        }

        // Early out if outside of portal bounds
        if top_b.0.x < portal.x_min as f32 || top_a.0.x > portal.x_max as f32 {
            return;
        }

        // Let's clamp X to the screen space bounds
        let x_min = (top_a.0.x as usize).clamp(portal.x_min, portal.x_max);
        let x_max = (top_b.0.x as usize).clamp(portal.x_min, portal.x_max);

        let x_delta = top_b.0.x - top_a.0.x;
        debug_assert!(x_delta > 0.0); // This should never be zero, as we cull back-facing walls
        let inv_x_delta = 1.0 / x_delta;

        let wall_lerp = WallInterpolator::new(
            top_a.0,
            top_b.0,
            bottom_a.0,
            bottom_b.0,
            tex_a,
            tex_b,
            inv_depth_a,
            inv_depth_b,
            x_min as f32,
            inv_x_delta,
        );

        
        let depth_a = vs_a.magnitude_sq();
        let depth_b = vs_b.magnitude_sq();
        let depth_gradient = (depth_b - depth_a) * inv_x_delta;

        let x_min_offset = x_min as f32 - top_a.0.x;
        let x_max_offset = top_a.0.x - x_max as f32;

        let depth_a = depth_a + depth_gradient * x_min_offset;
        let depth_b = depth_b + depth_gradient * x_max_offset;

        let max_depth = depth_a.max(depth_b);
        portal.depth_max = portal.depth_max.max(max_depth);

        // HACK: This should not live here, but exists for testing if it's worth using angle-based lighting
        let light_direction = Vec2f::new(0.0, 1.0);
        let light_angle = light_direction.dot(-wall.normal) * 0.5 + 0.5;
        let light_intensity = 0.7;
        let lighting = (1.0 - light_intensity) + (light_angle * light_intensity);

        self.rasterise_wall(state, wall_lerp, lighting, texture, y_bounds, x_min, x_max);
    }

    pub fn draw_portal_wall(
        &mut self,
        state: &mut RendererState,
        portals: &mut PortalTree,
        portal_index: usize,
        sector: &Sector,
        next_sector: &Sector,
        wall: &Wall,
        upper_texture: &Texture,
        lower_texture: &Texture,
    ) {
        let portal = unsafe { portals.get_node_mut_unchecked(portal_index) };

        // Transform coordinates based on camera position and orientation
        let mut vs_a = state.transform_view(wall.segment.a);
        let mut vs_b = state.transform_view(wall.segment.b);

        // Frustum culling
        if !Segment::new(vs_a, vs_b).overlaps_polygon(&state.frustum) {
            return;
        }

        // TODO: Consider precalculating these values, but we must then make sure to update
        // texture coordinates for all walls in a sector when the sector's ceiling/floor height changes.
        let mut upper_tex_a = Vec2f::new(0.0, 0.0);
        let mut upper_tex_b = Vec2f::new(
            wall.width,
            sector.ceiling.height - next_sector.ceiling.height,
        );
        upper_tex_a += wall.texture_data.offset;
        upper_tex_b += wall.texture_data.offset;
        upper_tex_a *= wall.texture_data.scale;
        upper_tex_b *= wall.texture_data.scale;

        let mut lower_tex_a = Vec2f::new(0.0, sector.ceiling.height - next_sector.floor.height);
        let mut lower_tex_b = Vec2f::new(wall.width, sector.ceiling.height - sector.floor.height);
        lower_tex_a += wall.texture_data.offset;
        lower_tex_b += wall.texture_data.offset;
        lower_tex_a *= wall.texture_data.scale;
        lower_tex_b *= wall.texture_data.scale;

        // Near plane clipping
        if vs_a.y < NEAR {
            let t = (NEAR - vs_a.y) / (vs_b.y - vs_a.y);

            vs_a.y = NEAR;
            vs_a.x += (vs_b.x - vs_a.x) * t;
            upper_tex_a.x += (upper_tex_b.x - upper_tex_a.x) * t;
            lower_tex_a.x += (lower_tex_b.x - lower_tex_a.x) * t;
        } else if vs_b.y < NEAR {
            let t = (NEAR - vs_b.y) / (vs_a.y - vs_b.y);

            vs_b.y = NEAR;
            vs_b.x += (vs_a.x - vs_b.x) * t;
            upper_tex_b.x += (upper_tex_a.x - upper_tex_b.x) * t;
            lower_tex_b.x += (lower_tex_a.x - lower_tex_b.x) * t;
        }

        // Perspective projection into screen space
        let top_a = state.project_screen_space(vs_a, sector.ceiling.height);
        let top_b = state.project_screen_space(vs_b, sector.ceiling.height);

        // Early out if wall is back-facing
        if top_a.0.x >= top_b.0.x {
            return;
        }

        // Early out if outside of portal bounds
        if top_b.0.x < portal.x_min as f32 || top_a.0.x > portal.x_max as f32 {
            return;
        }

        let bottom_a = state.project_screen_space(vs_a, sector.floor.height);
        let bottom_b = state.project_screen_space(vs_b, sector.floor.height);
        let next_top_a = state.project_screen_space(vs_a, next_sector.ceiling.height);
        let next_top_b = state.project_screen_space(vs_b, next_sector.ceiling.height);
        let next_bottom_a = state.project_screen_space(vs_a, next_sector.floor.height);
        let next_bottom_b = state.project_screen_space(vs_b, next_sector.floor.height);

        let inv_depth_a = top_a.1;
        let inv_depth_b = top_b.1;

        // Let's clamp X to the screen space bounds
        let x_min = (top_a.0.x as usize).clamp(portal.x_min, portal.x_max);
        let x_max = (top_b.0.x as usize).clamp(portal.x_min, portal.x_max);

        let x_delta = top_b.0.x - top_a.0.x;
        debug_assert!(x_delta > 0.0); // This should never be zero, as we cull back-facing walls
        let inv_x_delta = 1.0 / x_delta;

        // Rasterise portal wall
        let upper_wall_lerp = WallInterpolator::new(
            top_a.0,
            top_b.0,
            next_top_a.0,
            next_top_b.0,
            upper_tex_a,
            upper_tex_b,
            inv_depth_a,
            inv_depth_b,
            x_min as f32,
            inv_x_delta,
        );
        let lower_wall_lerp = WallInterpolator::new(
            next_bottom_a.0,
            next_bottom_b.0,
            bottom_a.0,
            bottom_b.0,
            lower_tex_a,
            lower_tex_b,
            inv_depth_a,
            inv_depth_b,
            x_min as f32,
            inv_x_delta,
        );

        let depth_a = vs_a.magnitude_sq();
        let depth_b = vs_b.magnitude_sq();
        let depth_gradient = (depth_b - depth_a) * inv_x_delta;

        let x_min_offset = x_min as f32 - top_a.0.x;
        let x_max_offset = top_a.0.x - x_max as f32;

        let depth_a = depth_a + depth_gradient * x_min_offset;
        let depth_b = depth_b + depth_gradient * x_max_offset;

        let min_depth = depth_a.min(depth_b);
        let max_depth = depth_a.max(depth_b);
        portal.depth_max = portal.depth_max.max(max_depth);

        let current_tree_depth = portal.tree_depth;
        portals.push_node(PortalNode {
            tree_depth: current_tree_depth + 1,
            sector_index: next_sector.id,
            x_min,
            x_max,
            depth_min: min_depth,
            depth_max: max_depth,
        });

        let (read_y_bounds, write_y_bounds) =
            portals.get_many_bounds_mut_unchecked(current_tree_depth, current_tree_depth + 1);

        let read_y_bounds = (&*read_y_bounds.0, &*read_y_bounds.1);

        // HACK: This should not live here, but exists for testing if it's worth using angle-based lighting
        let light_direction = Vec2f::new(0.0, 1.0);
        let light_angle = light_direction.dot(-wall.normal) * 0.5 + 0.5;
        let light_intensity = 0.7;
        let lighting = (1.0 - light_intensity) + (light_angle * light_intensity);

        self.rasterise_portal_wall(
            state,
            upper_wall_lerp,
            lower_wall_lerp,
            lighting,
            upper_texture,
            lower_texture,
            read_y_bounds,
            write_y_bounds,
            x_min,
            x_max,
        );
    }

    fn rasterise_wall(
        &mut self,
        state: &mut RendererState,
        mut wall: WallInterpolator,
        lighting: f32,
        texture: &Texture,
        y_bounds: (&[u16], &[u16]),
        x_min: usize,
        x_max: usize,
    ) {
        // Draw wall, one column at a time
        for x in x_min..x_max {
            let min_portal_bound = y_bounds.0[x] as usize;
            let max_portal_bound = y_bounds.1[x] as usize;

            // Clamp Y to the screen portal bounds
            let y_min = (wall.top_y as usize).clamp(min_portal_bound, max_portal_bound);
            let y_max = (wall.bottom_y as usize).clamp(min_portal_bound, max_portal_bound);

            // TODO: I don't actually think we need to clamp Y here, as this sort of thing is only
            // necessary for portal rasterisation due to the way they interact with the next sector,
            // potentially causing strangeness.

            // Ensure that max >= min using min as boundary (for no particular reason)
            // let y_max = y_max.max(y_min);

            self.rasterise_wall_span(state, &mut wall, lighting, texture, x, y_min, y_max);

            self.wall_bounds_min[x] = y_min as u16;
            self.wall_bounds_max[x] = y_max as u16;

            wall.step_x();
        }
    }

    fn rasterise_portal_wall(
        &mut self,
        state: &mut RendererState,
        mut upper_wall: WallInterpolator,
        mut lower_wall: WallInterpolator,
        lighting: f32,
        upper_texture: &Texture,
        lower_texture: &Texture,
        read_y_bounds: (&[u16], &[u16]),
        write_y_bounds: (&mut [u16], &mut [u16]),
        x_min: usize,
        x_max: usize,
    ) {
        // Draw wall, one column at a time
        for x in x_min..x_max {
            let min_portal_bound = read_y_bounds.0[x] as usize;
            let max_portal_bound = read_y_bounds.1[x] as usize;

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

            self.rasterise_wall_span(
                state,
                &mut upper_wall,
                lighting,
                upper_texture,
                x,
                upper_y_min,
                upper_y_max,
            );
            self.rasterise_wall_span(
                state,
                &mut lower_wall,
                lighting,
                lower_texture,
                x,
                lower_y_min,
                lower_y_max,
            );

            self.wall_bounds_min[x] = upper_y_min as u16;
            self.wall_bounds_max[x] = lower_y_max as u16;

            write_y_bounds.0[x] = upper_y_max as u16;
            write_y_bounds.1[x] = lower_y_min as u16;

            upper_wall.step_x();
            lower_wall.step_x();
        }
    }

    fn rasterise_wall_span(
        &mut self,
        state: &mut RendererState,
        wall: &mut WallInterpolator,
        lighting: f32,
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

        let lighting = unsafe {
            (diminish_lighting(normal_depth) * lighting * 255.0).to_int_unchecked::<u8>()
        };

        // Recover U texture coordinate after interpolating in depth space
        let u = wall.u_depth * depth;

        let width_mask = texture.levels[mip_level].width - 1;
        let height_mask = texture.levels[mip_level].height - 1;

        let texture_x = unsafe { (u * mip_scale).to_int_unchecked::<usize>() } & width_mask;

        for y in y_min..y_max {
            let texture_y =
                unsafe { (wall.v * mip_scale).to_int_unchecked::<usize>() } & height_mask;

            unsafe {
                let colour = texture
                    .sample_unchecked(texture_x, texture_y, mip_level)
                    .darken(lighting);
                state.framebuffer.set_pixel_unchecked(x, y, colour);
            }

            wall.step_y();
        }
    }
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
        let inv_depth;
        let top_y;
        let bottom_y;
        let u_depth;

        // We need to set our interpolation start values based on the X offset caused by clamping vertex a to
        // the screen space bounds.

        // We would usually do this by starting with vertex a and interpolating towards vertex b to a degree
        // based on the X offset. However, we can also start with vertex b and interpolate towards vertex a
        // to a negative degree based on the X offset.

        // If the wall has been clipped to the near plane, it is important to start with the vertex that was
        // not clipped as, in cases where the near Z is a very small number, the X offset can be extremely
        // large and cause precision issues when interpolating towards the non-clipped vertex.
        if inv_depth_a < inv_depth_b {
            let x_clamp_offset = x_min - top_a.x;

            inv_depth = inv_depth_a + inv_depth_m * x_clamp_offset;
            top_y = top_a.y + top_y_m * x_clamp_offset;
            bottom_y = bottom_a.y + bottom_y_m * x_clamp_offset;
            u_depth = u_depth_a + u_depth_m * x_clamp_offset;
        } else {
            let x_clamp_offset = top_b.x - x_min;

            inv_depth = inv_depth_b - inv_depth_m * x_clamp_offset;
            top_y = top_b.y - top_y_m * x_clamp_offset;
            bottom_y = bottom_b.y - bottom_y_m * x_clamp_offset;
            u_depth = u_depth_b - u_depth_m * x_clamp_offset;
        }

        // These will be reset/updated when we step in Y
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
