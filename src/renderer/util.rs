use maths::{geometry::Polygon, linear::Vec2f};

use crate::consts::{FAR, MAP_DEPTH_RANGE, MIP_FACTOR, MIP_LEVELS, NEAR};

/// Map a linear depth value, ranging from [NEAR] to [FAR], to a normalised depth value, ranging from 0.0 to 1.0.
pub fn normalise_depth(depth: f32) -> f32 {
    (depth - NEAR) * MAP_DEPTH_RANGE
}

/// Calculates an appropriate mip level based on the normalised depth and a bias.
pub fn mip_level(normal_depth: f32, bias: f32) -> usize {
    (((MIP_FACTOR + bias) * normal_depth) as usize).min(MIP_LEVELS - 1)
}

/// This is used during perspective projection to convert from camera space to screen space.
/// It is essentially a scaling factor that is used to get a pixel coordinate from a
/// coordinate in camera space, taking into account the field of view and screen size.
pub fn focal_dimensions(h_fov: f32, v_fov: f32, half_width: f32, half_height: f32) -> (f32, f32) {
    // Use similar triangles to calculate focal width/height, based on the screen we are
    // projecting onto and the field of view.
    let focal_width = half_width / (h_fov * 0.5).to_radians().tan();
    let focal_height = half_height / (v_fov * 0.5).to_radians().tan();

    (focal_width, focal_height)
}

/// Returns a polygon representing the view frustum, based on the given horizontal field of view.
pub fn view_frustum(h_fov: f32) -> Polygon {
    let tan = (h_fov * 0.5).to_radians().tan();
    let opp_far = FAR * tan;
    let opp_near = NEAR * tan;

    Polygon::from_vertices(vec![
        Vec2f::new(-opp_far, FAR),
        Vec2f::new(opp_far, FAR),
        Vec2f::new(opp_near, NEAR),
        Vec2f::new(-opp_near, NEAR),
    ])
}

/// Returns a lighting scaling factor, between 0.33 and 1.0, based on the normalised depth.
pub fn diminish_lighting(normal_depth: f32) -> f32 {
    let l = 1.0 - normal_depth;
    ((l * l * l) * 1.5).min(1.0)
}
