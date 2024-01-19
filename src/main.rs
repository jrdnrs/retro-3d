mod app;
mod bitmap;
mod camera;
mod colour;
mod consts;
mod font;
mod player;
mod renderer;
mod surface;
mod textures;
mod timer;
mod collision;
mod enemy;

use app::App;

fn main() {
    let app = App::new();
    app.run();
}
