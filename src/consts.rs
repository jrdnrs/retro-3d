/*
  Render visual settings
*/
pub const INTERNAL_WIDTH: usize = 640;
pub const INTERNAL_HEIGHT: usize = 400;
pub const HFOV: f32 = 75.0;
pub const FPS: f32 = 120.0;

/*
  Render clipping planes
*/
pub const NEAR: f32 = 0.00001;
pub const FAR: f32 = 512.0;
pub const MAP_DEPTH_RANGE: f32 = 1.0 / (FAR - NEAR);

/*
  Texture mip mapping
*/
/// The number of mip levels to generate for each texture, where level 0 is the original size and
/// subsequent levels are half the size of the previous level
pub const MIP_LEVELS: usize = 3;
/// Arbitrary factor to scale the mip level distance thresholds by. A higher value will result in
/// more mip levels being used for a given distance
pub const MIP_FACTOR: f32 = 4.0;
/// As subsequent mip maps are smaller resolutions, we use this to scale texture coordinates
pub const MIP_SCALES: [f32; MIP_LEVELS] = [1.0 / 1.0, 1.0 / 2.0, 1.0 / 4.0];

/*
  Textures
*/
// Tile texture paths
pub const TEXTURE_TILE_PATHS: [&str; 13] = [
    "assets/textures/tile/placeholder.png",
    "assets/textures/tile/brick.png",
    "assets/textures/tile/rock.png",
    "assets/textures/tile/stone.png",
    "assets/textures/tile/stone_brick.png",
    "assets/textures/tile/plank.png",
    "assets/textures/tile/grass.png",
    "assets/textures/tile/dirt.png",
    "assets/textures/tile/sand.png",
    "assets/textures/tile/concrete.png",
    "assets/textures/tile/leaf.png",
    "assets/textures/tile/obsidian.png",
    "assets/textures/tile/portal.png",
];
pub const TEXTURE_SPRITE_PATHS: [&str; 1] = ["assets/textures/entity/goblin.png"];

// Tile texture indices
pub const PLACEHOLDER: usize = 0;
pub const BRICK: usize = 1;
pub const ROCK: usize = 2;
pub const STONE: usize = 3;
pub const STONE_BRICK: usize = 4;
pub const PLANK: usize = 5;
pub const GRASS: usize = 6;
pub const DIRT: usize = 7;
pub const SAND: usize = 8;
pub const CONCRETE: usize = 9;
pub const LEAF: usize = 10;
pub const OBSIDIAN: usize = 11;
pub const PORTAL: usize = 12;

// Sprite texture indices
pub const GOBLIN: usize = 13;

/*
  Fonts
*/
// Font paths
pub const FONT_PATHS: [&str; 3] = [
    "assets/fonts/8pt_20.png",
    "assets/fonts/8pt_15.png",
    "assets/fonts/12pt_12.png",
];

// Font widths and heights (in pixels)
pub const FONT_SIZES: [(usize, usize); 3] = [(5, 9), (6, 9), (8, 12)];

// Font indices
pub const FONT_DEFAULT: usize = 0;
