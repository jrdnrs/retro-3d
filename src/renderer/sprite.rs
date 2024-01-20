use maths::{geometry::Segment, linear::Vec2f};

use crate::{consts::MIP_SCALES, surface::Sprite, textures::Texture};

use super::{
    portal::PortalTree,
    util::{diminish_lighting, mip_level, normalise_depth},
    RendererState,
};

pub struct SpriteRenderer {
    clip_min: Vec<u16>,
    clip_max: Vec<u16>,
}

impl SpriteRenderer {
    pub fn new(state: &RendererState) -> Self {
        let clip_min = vec![0; state.framebuffer.width()];
        let clip_max = vec![state.framebuffer.height() as u16; state.framebuffer.width()];

        Self { clip_min, clip_max }
    }

    pub fn set_viewport(&mut self, state: &RendererState) {
        self.clip_min.resize(state.framebuffer.width(), 0);
        self.clip_max
            .resize(state.framebuffer.width(), state.framebuffer.height() as u16);
    }

    pub fn update(&mut self, state: &RendererState) {}

    pub fn draw_sprite(
        &mut self,
        state: &mut RendererState,
        portals: &PortalTree,
        sprite: &Sprite,
        texture: &Texture,
    ) {
        let vs = state.transform_view(sprite.position);

        let depth = vs.y;

        // Reconstruct view space based on single transformed point, so that it always faces the camera
        let vs_a = vs - Vec2f::new(sprite.width * 0.5, 0.0);
        let vs_b = vs + Vec2f::new(sprite.width * 0.5, 0.0);

        // Frustum culling
        if !Segment::new(vs_a, vs_b).overlaps_polygon(&state.frustum) {
            return;
        }

        // Perspective projection into screen space
        // We only need to project the top-left and bottom-right corners, as the sprite is always
        // parallel to the screen.
        let top_left = state.project_screen_space(vs_a, sprite.height);
        let bottom_right = state.project_screen_space(vs_b, 0.0);

        // Early out if outside of screen space
        // We only check Y here, as frustum culling should have already taken care of X
        if bottom_right.0.y < 0.0 || top_left.0.y > state.framebuffer.height() as f32 {
            return;
        }

        // Clamp sprite coordinates to screen space bounds
        let sprite_x_min = (top_left.0.x as usize).clamp(0, state.framebuffer.width());
        let sprite_x_max = (bottom_right.0.x as usize).clamp(0, state.framebuffer.width());

        let mut portals_x_min = state.framebuffer.width();
        let mut portals_x_max = 0;

        for portal in portals.nodes.iter() {
            if portal.sector_index != sprite.sector_index {
                continue;
            }

            let portal_bounds = unsafe { portals.get_bounds_unchecked(portal.tree_depth) };

            // X bounds overlap between the sprite and this portal
            let overlap_x_min = sprite_x_min.max(portal.x_min as usize);
            let overlap_x_max = sprite_x_max.min(portal.x_max as usize);

            // Update the sprite's Y clip bounds for the overlapping X range
            for x in overlap_x_min..overlap_x_max {
                self.clip_min[x] = portal_bounds.0[x];
                self.clip_max[x] = portal_bounds.1[x];
            }

            // Update the collated X bounds for all relevant portals
            portals_x_min = portals_x_min.min(overlap_x_min);
            portals_x_max = portals_x_max.max(overlap_x_max);
        }

        // Update the X bounds for the sprite with the collated X bounds for all relevant portals as this
        // is the viewable portion
        let sprite_x_min = portals_x_min;
        let sprite_x_max = portals_x_max;

        // Early out if the sprite is completely obscured
        if sprite_x_min >= sprite_x_max {
            return;
        }

        // TODO: Consider precalculating these values, but we must then make sure to update
        // texture coordinates when the sprite changes shape
        let mut tex_coord_a = Vec2f::new(0.0, 0.0);
        let mut tex_coord_b = Vec2f::new(sprite.width, sprite.height);
        tex_coord_a += sprite.texture_data.offset;
        tex_coord_b += sprite.texture_data.offset;
        tex_coord_a *= sprite.texture_data.scale;
        tex_coord_b *= sprite.texture_data.scale;

        let sprite_lerp = SpriteInterpolator::new(
            top_left.0,
            bottom_right.0,
            tex_coord_a,
            tex_coord_b,
            sprite_x_min as f32,
        );

        self.rasterise_sprite(
            state,
            sprite_lerp,
            texture,
            depth,
            sprite_x_min,
            sprite_x_max,
        );
    }

    fn rasterise_sprite(
        &mut self,
        state: &mut RendererState,
        mut sprite: SpriteInterpolator,
        texture: &Texture,
        depth: f32,
        x_min: usize,
        x_max: usize,
    ) {
        let normal_depth = normalise_depth(depth);
        let mip_level = mip_level(normal_depth, 0.0);
        let mip_scale = MIP_SCALES[mip_level];

        let lighting =
            unsafe { (diminish_lighting(normal_depth) * 255.0).to_int_unchecked::<u8>() };

        // Draw sprite, one column at a time
        for x in x_min..x_max {
            let min_bound = self.clip_min[x] as usize;
            let max_bound = self.clip_max[x] as usize;

            let y_min = (sprite.top_y as usize).clamp(min_bound, max_bound);
            let y_max = (sprite.bottom_y as usize).clamp(min_bound, max_bound);

            self.rasterise_sprite_span(
                state,
                &mut sprite,
                texture,
                mip_level,
                mip_scale,
                lighting,
                x,
                y_min,
                y_max,
            );

            sprite.step_x();
        }
    }

    fn rasterise_sprite_span(
        &mut self,
        state: &mut RendererState,
        sprite: &mut SpriteInterpolator,
        texture: &Texture,
        mip_level: usize,
        mip_scale: f32,
        lighting: u8,
        x: usize,
        y_min: usize,
        y_max: usize,
    ) {
        sprite.init_y(y_min);

        let width_mask = texture.levels[mip_level].width - 1;
        let height_mask = texture.levels[mip_level].height - 1;

        let texture_x = unsafe { (sprite.u * mip_scale).to_int_unchecked::<usize>() } & width_mask;

        for y in y_min..y_max {
            let texture_y =
                unsafe { (sprite.v * mip_scale).to_int_unchecked::<usize>() } & height_mask;

            let colour = unsafe {
                texture
                    .sample_unchecked(texture_x, texture_y, mip_level)
                    .darken(lighting)
            };

            if colour.a != 0 {
                unsafe { state.framebuffer.set_pixel_unchecked(x, y, colour) };
            }
            // unsafe { state.framebuffer.blend_pixel_unchecked(x, y, colour)}

            sprite.step_y();
        }
    }
}

struct SpriteInterpolator {
    top_y: f32,
    bottom_y: f32,

    u: f32,
    u_m: f32,

    v_start: f32,
    v: f32,
    v_m: f32,
}

impl SpriteInterpolator {
    fn new(
        top_left: Vec2f,
        bottom_right: Vec2f,
        tex_coord_a: Vec2f,
        tex_coord_b: Vec2f,
        x_min: f32,
    ) -> Self {
        let x_delta = bottom_right.x - top_left.x;
        let y_delta = bottom_right.y - top_left.y;

        // These should always be positive, as sprites are always front-facing
        debug_assert!(x_delta > 0.0);
        debug_assert!(y_delta > 0.0);

        let inv_x_delta = 1.0 / x_delta;
        let inv_y_delta = 1.0 / y_delta;

        // Gradients with respect to X
        let u_m = (tex_coord_b.x - tex_coord_a.x) * inv_x_delta;

        // Offsets caused by clamping vertices to screen space bounds
        let x_clamp_offset = x_min - top_left.x;

        // Interpolator start values (includes clamping offsets)
        let u = tex_coord_a.x + u_m * x_clamp_offset;

        // Store V start value explicitly, as we need to reset V when we step in Y.
        // No need to store V delta, as it is constant, unlike with walls.
        let v_start = tex_coord_a.y;
        let v = v_start;
        let v_m = (tex_coord_b.y - tex_coord_a.y) * inv_y_delta;

        Self {
            top_y: top_left.y,
            bottom_y: bottom_right.y,

            u,
            u_m,

            v_start,
            v,
            v_m,
        }
    }

    fn init_y(&mut self, y_min: usize) {
        // Y offset caused by clamping vertices to screen space bounds
        let y_clamp_offset = y_min as f32 - self.top_y;

        // Unlike with walls, we don't need to recalculate the gradient of V with respect to Y,
        // as it is constant, as the original top/bottom Y coordinates are constant.

        // Interpolator start value
        self.v = self.v_start;

        // Account for difference caused by clamping Y
        self.v += self.v_m * y_clamp_offset;
    }

    fn step_x(&mut self) {
        self.u += self.u_m;
    }

    fn step_y(&mut self) {
        self.v += self.v_m;
    }
}
