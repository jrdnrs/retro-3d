use std::time::{Duration, Instant};

use input::Input;
use maths::{
    geometry::AABB,
    linear::{Mat2f, Vec2f},
};
use physics::collider::Collider;
use window::{
    application::WindowApplication,
    event::{Event, KeyCode, MouseButton, RenderEvent, WindowEvent},
    Window, WindowAttributes, WindowPosition, WindowSize,
};
#[cfg(target_os = "windows")]
use windows_sys::Win32::Media::{timeBeginPeriod, timeEndPeriod};

use crate::{
    player::Player,
    renderer::Renderer,
    surface::{Plane, Sector, Wall, WallTexture, PlaneTexture, Portal},
    textures::{self, Textures},
    timer::Timer,
};

pub const DEFAULT_WIDTH: usize = 640;
pub const DEFAULT_HEIGHT: usize = 400;
pub const DEFAULT_FOV: f32 = 75.0;
pub const FPS: f32 = 120.0;

pub const NEAR: f32 = 1.0;
pub const FAR: f32 = 512.0;
pub const MAP_DEPTH_RANGE: f32 = 1.0 / (FAR - NEAR);

pub struct App {
    window: Window,
    input: Input,
    timer: Timer,
    renderer: Renderer,

    textures: Textures,
    player: Player,
    sectors: Vec<Sector>,
}

impl App {
    pub fn new() -> Self {
        let window = Window::new(WindowAttributes {
            title: format!("Pseudo3D  {}x{}  ({}x)", DEFAULT_WIDTH, DEFAULT_HEIGHT, 2),
            size: WindowSize::new(DEFAULT_WIDTH * 2, DEFAULT_HEIGHT * 2),
            surface_size: Some(WindowSize::new(DEFAULT_WIDTH, DEFAULT_HEIGHT)),
            position: WindowPosition::new(200, 200),
            resizable: false,
            ..Default::default()
        });
        let input = Input::new();
        let timer = Timer::new();
        let renderer = Renderer::new(DEFAULT_WIDTH, DEFAULT_HEIGHT, DEFAULT_FOV);

        let textures = Textures::new();
        let player = Player::new(
            Vec2f::ZERO,
            15.0,
            Collider::new_aabb(AABB::from_dimensions(Vec2f::ZERO, Vec2f::new(10.0, 10.0))),
        );

        App {
            window,
            input,
            timer,
            renderer,

            textures,
            player,
            sectors: Vec::new(),
        }
    }

    pub fn setup(&mut self) {
        // This is used to reduce the minimum sleep time on Windows from ~15ms to ~1ms
        #[cfg(target_os = "windows")]
        unsafe {
            timeBeginPeriod(1)
        };

        self.textures.load_default();


        // Place player in sector 0
        self.player.camera.position = Vec2f::new(105.0, 180.0);

        // Point player towards sector 1
        self.player.camera.yaw = core::f32::consts::PI;
        self.player.camera.update(Vec2f::ZERO, Vec2f::ZERO);

        let x5_scale = Mat2f::rotation(1.1) * Mat2f::scale(Vec2f::uniform(5.0));

        let stone_brick_wall = WallTexture::new(textures::STONE_BRICK, Vec2f::ZERO, Vec2f::uniform(5.0));
        let grass_floor = PlaneTexture::new(textures::GRASS, Vec2f::ZERO, Vec2f::uniform(5.0), 0.0);
        let wood_ceiling = PlaneTexture::new(textures::PLANK, Vec2f::ZERO, Vec2f::uniform(5.0), 0.0);

        self.sectors = vec![
            Sector {
               walls: vec![
                   Wall::new(Vec2f::new(80.0, 200.0), Vec2f::new(130.0, 200.0), stone_brick_wall, None),
                   Wall::new(Vec2f::new(130.0, 200.0), Vec2f::new(130.0, 140.0), stone_brick_wall, None),
                   Wall::new(Vec2f::new(130.0, 140.0), Vec2f::new(90.0, 140.0), stone_brick_wall, Some(Portal::new(1, stone_brick_wall, stone_brick_wall))),
                   Wall::new(Vec2f::new(90.0, 140.0), Vec2f::new(80.0, 160.0), stone_brick_wall, Some(Portal::new(4, stone_brick_wall, stone_brick_wall))),
                   Wall::new(Vec2f::new(80.0, 160.0), Vec2f::new(80.0, 200.0), stone_brick_wall, None),
                   ],
               floor: Plane::new(0.0, grass_floor),
               ceiling: Plane::new(25.0, wood_ceiling),
               },
            
            Sector {
               walls: vec![
                   Wall::new(Vec2f::new(90.0, 140.0), Vec2f::new(130.0, 140.0), stone_brick_wall, Some(Portal::new(0, stone_brick_wall, stone_brick_wall))),	 		
                   Wall::new(Vec2f::new(130.0, 140.0), Vec2f::new(130.0, 100.0), stone_brick_wall, None),
                   Wall::new(Vec2f::new(130.0, 100.0), Vec2f::new(80.0, 100.0), stone_brick_wall, Some(Portal::new(2, stone_brick_wall, stone_brick_wall))),
                   Wall::new(Vec2f::new(80.0, 100.0), Vec2f::new(80.0, 130.0), stone_brick_wall, None),
                   Wall::new(Vec2f::new(80.0, 130.0), Vec2f::new(90.0, 140.0), stone_brick_wall, Some(Portal::new(4, stone_brick_wall, stone_brick_wall))),
                   ],
               floor: Plane::new(0.0, grass_floor),
               ceiling: Plane::new(25.0, wood_ceiling),
               },
            
            Sector {
               walls: vec![
                   Wall::new(Vec2f::new(80.0, 100.0), Vec2f::new(130.0, 100.0), stone_brick_wall, Some(Portal::new(1, stone_brick_wall, stone_brick_wall))),
                   Wall::new(Vec2f::new(130.0, 100.0), Vec2f::new(150.0,  80.0), stone_brick_wall, None),
                   Wall::new(Vec2f::new(150.0,  80.0), Vec2f::new(150.0,  60.0), stone_brick_wall, None),
                   Wall::new(Vec2f::new(150.0,  60.0), Vec2f::new(100.0,  60.0), stone_brick_wall, Some(Portal::new(3, stone_brick_wall, stone_brick_wall))),
                   Wall::new(Vec2f::new(100.0,  60.0), Vec2f::new(60.0,  60.0), stone_brick_wall, None),
                   Wall::new(Vec2f::new(60.0,  60.0), Vec2f::new(60.0,  80.0), stone_brick_wall, None),
                   Wall::new(Vec2f::new(60.0,  80.0), Vec2f::new(80.0, 100.0), stone_brick_wall, None),
                   ],
               floor: Plane::new(0.0, grass_floor),
               ceiling: Plane::new(25.0, wood_ceiling),
               },
           
            Sector {
               walls: vec![
                   Wall::new(Vec2f::new(100.0,  60.0), Vec2f::new(150.0,  60.0), stone_brick_wall, Some(Portal::new(2, stone_brick_wall, stone_brick_wall))),
                   Wall::new(Vec2f::new(150.0,  60.0), Vec2f::new(150.0,  30.0), stone_brick_wall, None),
                   Wall::new(Vec2f::new(150.0,  30.0), Vec2f::new(100.0,  30.0), stone_brick_wall, None),
                   Wall::new(Vec2f::new(100.0,  30.0), Vec2f::new(100.0,  60.0), stone_brick_wall, None),
                   ],
               floor: Plane::new(5.0, grass_floor),
               ceiling: Plane::new(25.0, wood_ceiling),
               },
           
            Sector {
               walls: vec![
                   Wall::new(Vec2f::new(40.0, 160.0), Vec2f::new(80.0, 160.0), stone_brick_wall, None),
                   Wall::new(Vec2f::new(80.0, 160.0), Vec2f::new(90.0, 140.0), stone_brick_wall, Some(Portal::new(0, stone_brick_wall, stone_brick_wall))),
                   Wall::new(Vec2f::new(90.0, 140.0), Vec2f::new(80.0, 130.0), stone_brick_wall, Some(Portal::new(1, stone_brick_wall, stone_brick_wall))),
                   Wall::new(Vec2f::new(80.0, 130.0), Vec2f::new(40.0, 130.0), stone_brick_wall, None),
                   Wall::new(Vec2f::new(40.0, 130.0), Vec2f::new(40.0, 160.0), stone_brick_wall, None),
                   ],
               floor: Plane::new(10.0, grass_floor),
               ceiling: Plane::new(20.0, wood_ceiling),
               },
           ];
      
    }

    pub fn update(&mut self) {
        let delta_seconds = self.timer.delta_seconds();

        self.player.update(delta_seconds, &self.input);

        // No need to render if window is minimised
        if self.window.get_minimised() {
            return;
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

        // Scale window
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
                        DEFAULT_WIDTH * scale,
                        DEFAULT_HEIGHT * scale,
                    ));
                    window.set_title(&format!(
                        "Pseudo3D  {}x{}  ({}x)",
                        DEFAULT_WIDTH, DEFAULT_HEIGHT, scale
                    ));

                    break;
                }
            }
        }

        // Test changing sector ceiling height
        if self.input.keyboard.is_key_held(KeyCode::ArrowUp) {
            self.sectors[0].ceiling.height += 10.0 * delta_seconds
        } else if self.input.keyboard.is_key_held(KeyCode::ArrowDown) {
            self.sectors[0].ceiling.height -= 10.0 * delta_seconds
        }

        // Test changing floor/ceiling texture rotation
        if self.input.keyboard.is_key_held(KeyCode::ArrowLeft) {
            self.sectors[0].floor.texture_data.scale_rotate =
                Mat2f::rotation(0.01 * self.timer.frame_count as f32) * Mat2f::scale(Vec2f::uniform(5.0));
            self.sectors[0].ceiling.texture_data.scale_rotate =
                Mat2f::rotation(0.01 * self.timer.frame_count as f32) * Mat2f::scale(Vec2f::uniform(5.0));
        } else if self.input.keyboard.is_key_held(KeyCode::ArrowRight) {
            self.sectors[0].floor.texture_data.scale_rotate =
                Mat2f::rotation(-0.01 * self.timer.frame_count as f32) * Mat2f::scale(Vec2f::uniform(5.0));
            self.sectors[0].ceiling.texture_data.scale_rotate =
                Mat2f::rotation(-0.01 * self.timer.frame_count as f32) * Mat2f::scale(Vec2f::uniform(5.0));
        }

        self.renderer
            .update(&self.player, &self.textures, &self.sectors);
        self.input.update();
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
                    let pixels = self.renderer.framebuffer.pixels_as_u32();
                    buffer.copy_from_slice(pixels);

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






