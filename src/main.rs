mod app;
mod camera;
mod colour;
mod player;
mod renderer;
mod surface;
mod timer;
mod textures;

use app::App;

fn main() {
    let app = App::new();
    app.run();
}
