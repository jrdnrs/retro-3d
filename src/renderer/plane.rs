use maths::linear::{Mat2f, Vec2f};

use crate::{
    app::{FAR, NEAR},
    renderer::util::{diminish_lighting, mip_level, normalise_depth},
    textures::{Texture, MIP_SCALES},
};

use super::{portal::PortalNode, RendererState};

pub struct PlaneRenderer {
    /// For each Y coordinate, stores starting X coordinate (inc.) of the horizontal spans that are
    /// currently being used to draw the floor/ceiling.
    span_start: Vec<u16>,

    /// When drawing floor/ceiling spans, we derive the depth based on its screen space Y coordinate.
    /// This is possible because we know the height of the floor/ceiling in camera space (which is constant),
    /// so we can reverse the perspective projection to get the depth.
    ///
    /// Most of this can be precalculated, and is stored here for each screen space Y coordinate. The result
    /// can be multiplied by the camera space height of the plane to get the depth.
    focal_height_ratios: Vec<f32>,
}

impl PlaneRenderer {
    pub fn new(state: &RendererState) -> Self {
        let span_start = vec![0; state.framebuffer.height()];
        let focal_height_ratios = vec![0.0; state.framebuffer.height()];

        Self {
            span_start,
            focal_height_ratios,
        }
    }

    pub fn set_viewport(&mut self, state: &RendererState) {
        self.span_start.resize(state.framebuffer.height(), 0);
        self.focal_height_ratios
            .resize(state.framebuffer.height(), 0.0);
    }

    pub fn update(&mut self, state: &RendererState) {
        // Precalculate focal height ratios using current pitch shear
        for y in 0..state.framebuffer.height() {
            let y_offset = y as f32 - state.framebuffer.half_height() - state.pitch_shear();
            if y_offset == 0.0 {
                self.focal_height_ratios[y] = 0.0;
            } else {
                self.focal_height_ratios[y] = state.focal_height() / y_offset;
            }
        }
    }

    pub fn draw_plane(
        &mut self,
        state: &mut RendererState,
        portal: &PortalNode,
        y_bounds: (&[u16], &[u16]),
        height_offset: f32,
        texture: &Texture,
        texture_offset: Vec2f,
        texture_scale_rotate: &Mat2f,
    ) {
        // Portal and wall bounds are collected during rasterisation of walls. We can use these to
        // draw floors and ceilings horizontally, which allows for fewer depth calculations as
        // depth is constant horizontally.

        // These indicate the current Y range of horizontal lines that are 'open'. The starting
        // X coordinate of each open line is stored in `span_start`. At the point they are found to be
        // 'closed', the span is drawn. It can be considered closed when outside of the bounds.
        let mut y_min = y_bounds.0[portal.x_min];
        let mut y_max = y_min;

        for x in portal.x_min..portal.x_max {
            let min_bound = y_bounds.0[x];
            let max_bound = y_bounds.1[x];

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
                    state,
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
                    state,
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
                state,
                texture,
                texture_offset,
                texture_scale_rotate,
                height_offset,
                y as usize,
                self.span_start[y as usize] as usize,
                portal.x_max,
            );
        }
    }

    fn rasterise_plane_span(
        &mut self,
        state: &mut RendererState,
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

        // When at the horizon, depth tends towards infinity, so skip drawing if so
        if depth < NEAR || depth > FAR {
            return;
        }

        let normal_depth = normalise_depth(depth);
        let mip_level = mip_level(normal_depth, focal_height_ratio.abs());
        let mip_scale = MIP_SCALES[mip_level];

        let lighting =
            unsafe { (diminish_lighting(normal_depth) * 255.0).to_int_unchecked::<u16>() };

        // Calculate world space coordinates of either end of the span, via reversing the perspective
        // projection, and use these as the texture coordinates.
        let ws_1 = Vec2f::new(
            ((x_min as f32 - state.framebuffer.half_width()) * depth) * state.inv_focal_width(),
            -depth,
        )
        .rotate(state.camera.yaw_sin, state.camera.yaw_cos)
            + Vec2f::new(state.camera.position.x, -state.camera.position.y);

        let ws_2 = Vec2f::new(
            ((x_max as f32 - state.framebuffer.half_width()) * depth) * state.inv_focal_width(),
            -depth,
        )
        .rotate(state.camera.yaw_sin, state.camera.yaw_cos)
            + Vec2f::new(state.camera.position.x, -state.camera.position.y);

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

            let colour = unsafe {
                texture
                    .sample_unchecked(texture_x, texture_y, mip_level)
                    .lightness(lighting)
            };
            unsafe { state.framebuffer.set_pixel_unchecked(x, y, colour) };

            u += u_m;
            v += v_m;
        }
    }
}
