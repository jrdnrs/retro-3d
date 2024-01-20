use std::time::{Duration, Instant};

use input::Input;
use maths::{
    geometry::Segment,
    linear::{Mat2f, Vec2f},
};
use window::{
    application::WindowApplication,
    event::{Event, KeyCode, MouseButton, RenderEvent, WindowEvent},
    Window, WindowAttributes, WindowPosition, WindowSize,
};
#[cfg(target_os = "windows")]
use windows_sys::Win32::Media::{timeBeginPeriod, timeEndPeriod};

use crate::{
    colour::BGRA8,
    consts::*,
    font::{AlignHeight, AlignWidth, Font},
    player::Player,
    renderer::Renderer,
    surface::{Plane, PlaneTexture, Portal, Sector, Sprite, Wall, WallTexture},
    textures::Texture,
    timer::Timer,
};

pub struct App {
    window: Window,
    input: Input,
    timer: Timer,
    renderer: Renderer,

    player: Player,
    textures: Vec<Texture>,
    fonts: Vec<Font>,
    sectors: Vec<Sector>,
    sprites: Vec<Sprite>,
}

impl App {
    pub fn new() -> Self {
        let window = Window::new(WindowAttributes {
            title: format!("Pseudo3D  {}x{}  ({}x)", INTERNAL_WIDTH, INTERNAL_HEIGHT, 2),
            size: WindowSize::new(INTERNAL_WIDTH * 2, INTERNAL_HEIGHT * 2),
            surface_size: Some(WindowSize::new(INTERNAL_WIDTH, INTERNAL_HEIGHT)),
            position: WindowPosition::new(200, 200),
            resizable: false,
            ..Default::default()
        });
        let input = Input::new();
        let timer = Timer::new();
        let renderer = Renderer::new(INTERNAL_WIDTH, INTERNAL_HEIGHT, HFOV);

        let textures = Vec::new();
        let fonts = Vec::new();
        let player = Player::new(Vec2f::ZERO, 15.0, 0);

        App {
            window,
            input,
            timer,
            renderer,

            player,
            textures,
            fonts,
            sectors: Vec::new(),
            sprites: Vec::new(),
        }
    }

    pub fn setup(&mut self) {
        // This is used to reduce the minimum sleep time on Windows from ~15ms to ~1ms
        #[cfg(target_os = "windows")]
        unsafe {
            timeBeginPeriod(1)
        };

        // Load default assets
        for path in TEXTURE_TILE_PATHS {
            self.textures.push(Texture::from_path_png(path).unwrap());
        }
        for path in TEXTURE_SPRITE_PATHS {
            self.textures.push(Texture::from_path_png(path).unwrap());
        }
        for (path, size) in FONT_PATHS.iter().zip(FONT_SIZES.iter()) {
            self.fonts
                .push(Font::from_path_png(path, size.0, size.1, 1).unwrap());
        }

        // Enable debug drawing by default
        self.renderer.state_mut().debug = true;

        // Place player in sector 0
        self.player.camera.position = Vec2f::new(105.0, 180.0);

        // Point player towards sector 1
        self.player.camera.yaw = core::f32::consts::PI;
        self.player.camera.translate(Vec2f::ZERO);
        self.player.camera.rotate(Vec2f::ZERO);

        let stone_brick_wall = WallTexture::new(STONE_BRICK, Vec2f::ZERO, Vec2f::uniform(5.0));
        let leaf_wall = WallTexture::new(LEAF, Vec2f::ZERO, Vec2f::uniform(5.0));
        let grass_floor = PlaneTexture::new(GRASS, Vec2f::ZERO, Vec2f::uniform(5.0), 0.0);
        let wood_ceiling = PlaneTexture::new(PLANK, Vec2f::ZERO, Vec2f::uniform(5.0), 0.0);

        self.sectors = vec![
            Sector {
                id: 0,
                walls: vec![
                    Wall::new(
                        Vec2f::new(80.0, 600.0),
                        Vec2f::new(130.0, 600.0),
                        stone_brick_wall,
                        None,
                    ),
                    Wall::new(
                        Vec2f::new(130.0, 600.0),
                        Vec2f::new(130.0, 140.0),
                        stone_brick_wall,
                        None,
                    ),
                    Wall::new(
                        Vec2f::new(130.0, 140.0),
                        Vec2f::new(90.0, 140.0),
                        stone_brick_wall,
                        Some(Portal::new(1, stone_brick_wall, stone_brick_wall)),
                    ),
                    Wall::new(
                        Vec2f::new(90.0, 140.0),
                        Vec2f::new(80.0, 160.0),
                        stone_brick_wall,
                        Some(Portal::new(4, stone_brick_wall, stone_brick_wall)),
                    ),
                    Wall::new(
                        Vec2f::new(80.0, 160.0),
                        Vec2f::new(80.0, 600.0),
                        stone_brick_wall,
                        None,
                    ),
                ],
                floor: Plane::new(0.0, grass_floor),
                ceiling: Plane::new(25.0, wood_ceiling),
            },
            Sector {
                id: 1,
                walls: vec![
                    Wall::new(
                        Vec2f::new(90.0, 140.0),
                        Vec2f::new(130.0, 140.0),
                        stone_brick_wall,
                        Some(Portal::new(0, stone_brick_wall, stone_brick_wall)),
                    ),
                    Wall::new(
                        Vec2f::new(130.0, 140.0),
                        Vec2f::new(130.0, 100.0),
                        stone_brick_wall,
                        None,
                    ),
                    Wall::new(
                        Vec2f::new(130.0, 100.0),
                        Vec2f::new(80.0, 100.0),
                        stone_brick_wall,
                        Some(Portal::new(2, stone_brick_wall, stone_brick_wall)),
                    ),
                    Wall::new(
                        Vec2f::new(80.0, 100.0),
                        Vec2f::new(80.0, 130.0),
                        stone_brick_wall,
                        None,
                    ),
                    Wall::new(
                        Vec2f::new(80.0, 130.0),
                        Vec2f::new(90.0, 140.0),
                        stone_brick_wall,
                        Some(Portal::new(4, stone_brick_wall, stone_brick_wall)),
                    ),
                ],
                floor: Plane::new(0.0, grass_floor),
                ceiling: Plane::new(25.0, wood_ceiling),
            },
            Sector {
                id: 2,
                walls: vec![
                    Wall::new(
                        Vec2f::new(80.0, 100.0),
                        Vec2f::new(130.0, 100.0),
                        stone_brick_wall,
                        Some(Portal::new(1, stone_brick_wall, stone_brick_wall)),
                    ),
                    Wall::new(
                        Vec2f::new(130.0, 100.0),
                        Vec2f::new(150.0, 80.0),
                        stone_brick_wall,
                        None,
                    ),
                    Wall::new(
                        Vec2f::new(150.0, 80.0),
                        Vec2f::new(150.0, 60.0),
                        stone_brick_wall,
                        None,
                    ),
                    Wall::new(
                        Vec2f::new(150.0, 60.0),
                        Vec2f::new(100.0, 60.0),
                        stone_brick_wall,
                        Some(Portal::new(3, stone_brick_wall, stone_brick_wall)),
                    ),
                    Wall::new(
                        Vec2f::new(100.0, 60.0),
                        Vec2f::new(60.0, 60.0),
                        stone_brick_wall,
                        None,
                    ),
                    Wall::new(
                        Vec2f::new(60.0, 60.0),
                        Vec2f::new(60.0, 80.0),
                        stone_brick_wall,
                        None,
                    ),
                    Wall::new(
                        Vec2f::new(60.0, 80.0),
                        Vec2f::new(80.0, 100.0),
                        stone_brick_wall,
                        None,
                    ),
                ],
                floor: Plane::new(0.0, grass_floor),
                ceiling: Plane::new(30.0, wood_ceiling),
            },
            Sector {
                id: 3,
                walls: vec![
                    Wall::new(
                        Vec2f::new(100.0, 60.0),
                        Vec2f::new(150.0, 60.0),
                        stone_brick_wall,
                        Some(Portal::new(2, stone_brick_wall, stone_brick_wall)),
                    ),
                    Wall::new(
                        Vec2f::new(150.0, 60.0),
                        Vec2f::new(150.0, 30.0),
                        stone_brick_wall,
                        None,
                    ),
                    Wall::new(
                        Vec2f::new(150.0, 30.0),
                        Vec2f::new(100.0, 30.0),
                        leaf_wall,
                        None,
                    ),
                    Wall::new(
                        Vec2f::new(100.0, 30.0),
                        Vec2f::new(100.0, 60.0),
                        stone_brick_wall,
                        None,
                    ),
                ],
                floor: Plane::new(2.0, grass_floor),
                ceiling: Plane::new(25.0, wood_ceiling),
            },
            Sector {
                id: 4,
                walls: vec![
                    Wall::new(
                        Vec2f::new(40.0, 160.0),
                        Vec2f::new(80.0, 160.0),
                        stone_brick_wall,
                        None,
                    ),
                    Wall::new(
                        Vec2f::new(80.0, 160.0),
                        Vec2f::new(90.0, 140.0),
                        stone_brick_wall,
                        Some(Portal::new(0, stone_brick_wall, stone_brick_wall)),
                    ),
                    Wall::new(
                        Vec2f::new(90.0, 140.0),
                        Vec2f::new(80.0, 130.0),
                        stone_brick_wall,
                        Some(Portal::new(1, stone_brick_wall, stone_brick_wall)),
                    ),
                    Wall::new(
                        Vec2f::new(80.0, 130.0),
                        Vec2f::new(40.0, 130.0),
                        stone_brick_wall,
                        None,
                    ),
                    Wall::new(
                        Vec2f::new(40.0, 130.0),
                        Vec2f::new(40.0, 160.0),
                        stone_brick_wall,
                        None,
                    ),
                ],
                floor: Plane::new(10.0, grass_floor),
                ceiling: Plane::new(20.0, wood_ceiling),
            },
        ];

        self.sprites = vec![
            Sprite::new(
                Vec2f::new(140.0, 80.0),
                2,
                WallTexture::new(GOBLIN, Vec2f::ZERO, Vec2f::uniform(8.0)),
                15.0,
                15.0,
            ),
            Sprite::new(
                Vec2f::new(80.0, 80.0),
                2,
                WallTexture::new(GOBLIN, Vec2f::ZERO, Vec2f::uniform(8.0)),
                15.0,
                15.0,
            ),
        ];
    }

    pub fn update(&mut self) {
        let delta_seconds = self.timer.delta_seconds();

        self.player.update_movement(delta_seconds, &self.input);

        // Update current sector
        let displacement_segment =
            Segment::new(self.player.camera.position, self.player.prev_position);
        for wall in self.sectors[self.player.sector_index].walls.iter() {
            if let Some(portal) = wall.portal {
                let wall_segment = Segment::new(wall.segment.a, wall.segment.b);
                if displacement_segment.intersects(&wall_segment) {
                    // Adjust z position for new sector
                    let z_delta = self.sectors[portal.sector].floor.height
                        - self.sectors[self.player.sector_index].floor.height;

                    self.player.camera.z += z_delta;
                    self.player.head_z += z_delta;
                    self.player.knee_z += z_delta;

                    self.player.sector_index = portal.sector;

                    break;
                }
            }
        }

        // Wall collision
        for wall in self.sectors[self.player.sector_index].walls.iter() {
            let distance_sq = wall.segment.point_distance_sq(self.player.camera.position);

            if distance_sq <= self.player.collider.radius * self.player.collider.radius {
                if let Some(portal) = wall.portal {
                    let next_sector_floor_z = self.sectors[portal.sector].floor.height;
                    let next_sector_ceiling_z = self.sectors[portal.sector].ceiling.height;

                    // If player fits through portal, don't collide
                    if self.player.head_z < next_sector_ceiling_z
                        && self.player.knee_z > next_sector_floor_z
                    {
                        continue;
                    }
                }

                let depth = self.player.collider.radius - distance_sq.sqrt();
                let correction = wall.normal * depth;
                self.player.translate(-correction);

                self.player.velocity -= wall.normal * wall.normal.dot(self.player.velocity) * 0.5;
            }
        }

        // No need to render if window is minimised
        if self.window.get_minimised() {
            return;
        }

        // Toggle debug drawing
        if self.input.keyboard.is_key_pressed(KeyCode::F3) {
            self.renderer.state_mut().debug = !self.renderer.state().debug;
        }

        // Toggle cursor grab
        if !self.window.get_cursor_grab() && self.input.mouse.is_button_pressed(MouseButton::Left) {
            self.window.set_cursor_grab(true);
            self.window.set_cursor_visible(false);
            self.input.mouse.grabbed = true;
        } else if self.window.get_cursor_grab()
            && self.input.keyboard.is_key_pressed(KeyCode::Escape)
        {
            self.window.set_cursor_grab(false);
            self.window.set_cursor_visible(true);
            self.input.mouse.grabbed = false;
        }

        // Integer window scaling
        if self.input.keyboard.is_key_held(KeyCode::ControlLeft)
            || self.input.keyboard.is_key_held(KeyCode::ControlRight)
        {
            let keys = [
                KeyCode::Digit1,
                KeyCode::Digit2,
                KeyCode::Digit3,
                KeyCode::Digit4,
                KeyCode::Digit5,
            ];

            for (i, key) in keys.iter().enumerate() {
                if self.input.keyboard.is_key_pressed(*key) {
                    let window = self.get_window_mut();
                    let scale = i + 1;

                    window.set_size(WindowSize::new(
                        INTERNAL_WIDTH * scale,
                        INTERNAL_HEIGHT * scale,
                    ));
                    window.set_title(&format!(
                        "Pseudo3D  {}x{}  ({}x)",
                        INTERNAL_WIDTH, INTERNAL_HEIGHT, scale
                    ));

                    break;
                }
            }
        }

        // Test changing sector ceiling height
        if self.input.keyboard.is_key_held(KeyCode::ArrowUp) {
            self.sectors[self.player.sector_index].ceiling.height += 10.0 * delta_seconds
        } else if self.input.keyboard.is_key_held(KeyCode::ArrowDown) {
            self.sectors[self.player.sector_index].ceiling.height -= 10.0 * delta_seconds
        }

        // Test changing floor/ceiling texture rotation
        if self.input.keyboard.is_key_held(KeyCode::ArrowLeft) {
            self.sectors[self.player.sector_index]
                .floor
                .texture_data
                .scale_rotate = Mat2f::rotation(0.01 * self.timer.frame_count as f32)
                * Mat2f::scale(Vec2f::uniform(5.0));
            self.sectors[self.player.sector_index]
                .ceiling
                .texture_data
                .scale_rotate = Mat2f::rotation(0.01 * self.timer.frame_count as f32)
                * Mat2f::scale(Vec2f::uniform(5.0));
        } else if self.input.keyboard.is_key_held(KeyCode::ArrowRight) {
            self.sectors[self.player.sector_index]
                .floor
                .texture_data
                .scale_rotate = Mat2f::rotation(-0.01 * self.timer.frame_count as f32)
                * Mat2f::scale(Vec2f::uniform(5.0));
            self.sectors[self.player.sector_index]
                .ceiling
                .texture_data
                .scale_rotate = Mat2f::rotation(-0.01 * self.timer.frame_count as f32)
                * Mat2f::scale(Vec2f::uniform(5.0));
        }

        self.renderer.update(
            &self.timer,
            &self.player,
            &self.textures,
            &self.sectors,
            &self.sprites,
        );
        self.input.update();

        // Draw debug text
        if self.renderer.state().debug {
            self.renderer.draw_text(
                &self.fonts[0],
                BGRA8::ORANGE,
                (AlignWidth::Left, AlignHeight::Top),
                0.01,
                0.01,
                &format!(
                    "Sector:   {:>3}
Position: {:>6.2} {:>6.2} {:>6.2}
Rotation: {:>6.2} {:>6.2}
Velocity: {:>6.2} {:>6.2}
Speed:    {:>6.2}",
                    self.player.sector_index,
                    self.player.camera.position.x,
                    self.player.camera.position.y,
                    self.player.camera.z,
                    self.player.camera.yaw,
                    self.player.camera.pitch,
                    self.player.velocity.x,
                    self.player.velocity.y,
                    self.player.velocity.magnitude()
                ),
            );
        }
    }

    pub fn run(mut self) -> ! {
        self.setup();

        // Execute the event loop of the window
        <Self as WindowApplication>::run(self)
    }
}

impl WindowApplication for App {
    fn get_window(&self) -> &Window {
        &self.window
    }

    fn get_window_mut(&mut self) -> &mut Window {
        &mut self.window
    }

    fn on_event(&mut self, event: &Event) {
        self.input.handle_event(event);

        match event {
            Event::RenderEvent(render_event) => match render_event {
                RenderEvent::RedrawRequested => {
                    self.update();

                    // Copy renderer framebuffer to window framebuffer
                    let mut ctx = self.window.graphics_context();
                    let buffer = ctx.framebuffer_mut();
                    let pixels = self.renderer.framebuffer().pixels_as_u32();
                    unsafe {
                        std::ptr::copy_nonoverlapping(
                            pixels.as_ptr(),
                            buffer.as_mut_ptr(),
                            pixels.len(),
                        );
                    }

                    // Debug timings
                    let update_time_elapsed = Instant::now() - self.timer.prev_update;
                    self.timer.time_buffer[self.timer.frame_count % 128] =
                        update_time_elapsed.as_secs_f32();
                    let average = Duration::from_secs_f32(
                        self.timer.time_buffer.iter().sum::<f32>()
                            / self.timer.time_buffer.len() as f32,
                    );
                    if self.timer.frame_count % 256 == 0 {
                        let fps = self.timer.frame_count as f32
                            / self.timer.start.elapsed().as_secs_f32();

                        print!("Update: {:?}, FPS: {:?}\r", average, fps);
                        std::io::Write::flush(&mut std::io::stdout()).unwrap();
                    }

                    // Sleep until next frame
                    let target = self.timer.prev_frame + self.timer.frame_time;
                    // round down to nearest millisecond as sleep is not accurate and often overshoots
                    let delta = Duration::from_millis((target - Instant::now()).as_millis() as u64);
                    self.timer.prev_frame = target;
                    self.timer.frame_count += 1;

                    if !delta.is_zero() {
                        std::thread::sleep(delta)
                    };
                }
            },

            Event::WindowEvent(window_event) => match window_event {
                WindowEvent::Resized(size) => {
                    if size.width == 0 && size.height == 0 {
                        return;
                    }

                    // self.renderer.set_viewport(size.width, size.height);
                }

                WindowEvent::CloseRequested => {
                    // This must match the call to `timeBeginPeriod` at the start of the program
                    #[cfg(target_os = "windows")]
                    unsafe {
                        timeEndPeriod(1);
                    };
                }

                _ => {}
            },

            _ => {}
        }
    }
}
