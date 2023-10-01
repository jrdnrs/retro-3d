use maths::{geometry::Polygon, linear::Vec2f};

use crate::{
    app::{FAR, NEAR},
    camera::Camera,
    colour::BGRA8,
    player::Player,
    surface::{Sector, Sprite},
    textures::Textures,
};

use super::{
    framebuffer::Framebuffer,
    portal::{PortalNode, PortalTree},
    sector::SectorRenderer,
    sprite::SpriteRenderer,
    util::{focal_dimensions, view_frustum},
};

pub struct RendererState {
    pub framebuffer: Framebuffer,
    pub camera: Camera,
    pub frustum: Polygon,

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

    pub debug: bool,
}

impl RendererState {
    pub fn new(screen_width: usize, screen_height: usize, h_fov: f32) -> Self {
        let framebuffer = Framebuffer::new(screen_width, screen_height);
        let camera = Camera::new(Vec2f::ZERO, 0.0);

        let v_fov = h_fov / framebuffer.aspect_ratio();

        let (focal_width, focal_height) = focal_dimensions(
            h_fov,
            v_fov,
            framebuffer.half_width(),
            framebuffer.half_height(),
        );
        let inv_focal_width = 1.0 / focal_width;
        let inv_focal_height = 1.0 / focal_height;

        let pitch_shear = camera.pitch_tan * focal_height;

        let frustum = view_frustum(h_fov);

        Self {
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

            debug: false,
        }
    }

    fn set_viewport(&mut self, width: usize, height: usize) {
        self.framebuffer = Framebuffer::new(width, height);
        self.v_fov = self.h_fov / self.framebuffer.aspect_ratio();

        (self.focal_width, self.focal_height) = focal_dimensions(
            self.h_fov,
            self.v_fov,
            self.framebuffer.half_width(),
            self.framebuffer.half_height(),
        );
        self.inv_focal_width = 1.0 / self.focal_width;
        self.inv_focal_height = 1.0 / self.focal_height;

        // No need to recalculate pitch shear and focal height ratios here, as they are initialised
        // at the start of `update`.
    }

    fn set_fov(&mut self, h_fov: f32) {
        self.h_fov = h_fov;
        self.v_fov = self.h_fov / self.framebuffer.aspect_ratio();

        (self.focal_width, self.focal_height) = focal_dimensions(
            self.h_fov,
            self.v_fov,
            self.framebuffer.half_width(),
            self.framebuffer.half_height(),
        );
        self.inv_focal_width = 1.0 / self.focal_width;
        self.inv_focal_height = 1.0 / self.focal_height;

        self.frustum = view_frustum(h_fov);

        // No need to recalculate pitch shear and focal height ratios here, as they are initialised
        // at the start of `update`.
    }

    fn update(&mut self, player: &Player) {
        // Use player camera
        self.camera = player.camera.clone();

        self.pitch_shear = self.camera.pitch_tan * self.focal_height;
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
        screen_space_x += self.framebuffer.half_width();
        screen_space_y += self.framebuffer.half_height();

        (Vec2f::new(screen_space_x, screen_space_y), inv_z)
    }

    pub fn h_fov(&self) -> f32 {
        self.h_fov
    }

    pub fn v_fov(&self) -> f32 {
        self.v_fov
    }

    pub fn focal_width(&self) -> f32 {
        self.focal_width
    }

    pub fn focal_height(&self) -> f32 {
        self.focal_height
    }

    pub fn inv_focal_width(&self) -> f32 {
        self.inv_focal_width
    }

    pub fn inv_focal_height(&self) -> f32 {
        self.inv_focal_height
    }

    pub fn pitch_shear(&self) -> f32 {
        self.pitch_shear
    }
}

pub struct Renderer {
    state: RendererState,
    portal_tree: PortalTree,
    sector_renderer: SectorRenderer,
    sprite_renderer: SpriteRenderer,
}

impl Renderer {
    pub fn new(screen_width: usize, screen_height: usize, h_fov: f32) -> Self {
        let state = RendererState::new(screen_width, screen_height, h_fov);
        let portal_tree = PortalTree::with_depth(4, screen_width, screen_height);
        let sector_renderer = SectorRenderer::new(&state);
        let sprite_renderer = SpriteRenderer::new(&state);

        Self {
            state,
            portal_tree,
            sector_renderer,
            sprite_renderer,
        }
    }

    pub fn state(&self) -> &RendererState {
        &self.state
    }

    pub fn state_mut(&mut self) -> &mut RendererState {
        &mut self.state
    }

    pub fn framebuffer(&self) -> &Framebuffer {
        &self.state.framebuffer
    }

    pub fn set_viewport(&mut self, screen_width: usize, screen_height: usize) {
        if screen_width == self.state.framebuffer.width()
            && screen_height == self.state.framebuffer.height()
        {
            return;
        }

        self.state.set_viewport(screen_width, screen_height);

        self.portal_tree.resize_bounds(screen_width, screen_height);
        self.sector_renderer.set_viewport(&self.state);
        self.sprite_renderer.set_viewport(&self.state);
    }

    pub fn set_fov(&mut self, h_fov: f32) {
        if h_fov == self.state.h_fov() {
            return;
        }

        self.state.set_fov(h_fov);
    }

    pub fn update(
        &mut self,
        player: &Player,
        textures: &Textures,
        sectors: &[Sector],
        sprites: &[Sprite],
    ) {
        self.state.update(player);

        self.portal_tree.reset();
        self.sector_renderer.update(&self.state);
        self.sprite_renderer.update(&self.state);

        // Add initial task to render the sector that the player is in
        self.portal_tree.push_node(PortalNode {
            tree_depth: 0,
            sector_index: player.sector_index,
            x_min: 0,
            x_max: self.state.framebuffer.width(),
            depth_min: NEAR,
            depth_max: NEAR,
        });

        // Sectors are rendered in a breadth-first manner, and each portal encountered is added
        // to the queue of tasks to be rendered.
        let mut portal_index = 0;
        while portal_index < self.portal_tree.nodes_len() {
            self.sector_renderer.draw_sector(
                &mut self.state,
                &mut self.portal_tree,
                sectors,
                textures,
                portal_index,
            );

            portal_index += 1;
        }


        for sprite in sprites {
            self.sprite_renderer.draw_sprite(
                &mut self.state,
                &self.portal_tree,
                sprite,
                textures.get(sprite.texture_data.index).unwrap(),
            );
        }

        if self.state.debug {
            self.debug_draw_portals();
        }
    }

    fn debug_draw_portals(&mut self) {
        for portal in self.portal_tree.nodes.iter() {
            // Portal is less than 1 pixel wide, so skip
            if portal.x_min == portal.x_max {
                continue;
            }

            let colour = BGRA8::new(
                192u8.wrapping_mul(portal.tree_depth as u8).wrapping_add(64),
                64u8.wrapping_mul(portal.tree_depth as u8).wrapping_add(32),
                32u8.wrapping_mul(portal.tree_depth as u8).wrapping_add(192),
                255,
            );

            let y_bounds = unsafe { self.portal_tree.get_bounds_unchecked(portal.tree_depth) };
            let left_x = portal.x_min;
            let right_x = portal.x_max.saturating_sub(1);

            // Draw left side
            for y in y_bounds.0[left_x]..y_bounds.1[left_x] {
                unsafe {
                    self.state
                        .framebuffer
                        .set_pixel_unchecked(left_x, y as usize, colour)
                };
            }

            // Draw right side
            for y in y_bounds.0[right_x]..y_bounds.1[right_x] {
                unsafe {
                    self.state
                        .framebuffer
                        .set_pixel_unchecked(right_x, y as usize, colour)
                };
            }

            // Draw top/bottom sides
            for x in portal.x_min..portal.x_max {
                // Portal is less than 1 pixel tall, so skip
                if y_bounds.0[x] == y_bounds.1[x] {
                    continue;
                }

                let top_y = y_bounds.0[x] as usize;
                let bottom_y = (y_bounds.1[x].saturating_sub(1)) as usize;

                unsafe { self.state.framebuffer.set_pixel_unchecked(x, top_y, colour) };

                unsafe {
                    self.state
                        .framebuffer
                        .set_pixel_unchecked(x, bottom_y, colour)
                };
            }
        }
    }
}
