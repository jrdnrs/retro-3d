use maths::{
    geometry::{Circle, Segment},
    linear::Vec2f,
};

#[derive(Clone, Copy, Debug)]
pub struct Collision {
    normal: Vec2f,
    depth: f32,
}

impl Collision {
    pub fn new(normal: Vec2f, depth: f32) -> Self {
        Self { normal, depth }
    }

    pub fn normal(&self) -> Vec2f {
        self.normal
    }

    pub fn depth(&self) -> f32 {
        self.depth
    }
}

pub fn sat_collision_polygons(points_a: &[Vec2f], points_b: &[Vec2f]) -> Option<Collision> {
    let mut points = [points_a, points_b];

    let mut min_depth = f32::MAX;
    let mut collision_normal = Vec2f::ZERO;

    // Run through twice, first with shape A as the reference and then with shape B
    for _ in 0..2 {
        let points_a = points[0];
        let points_b = points[1];

        // Project each point of each shape onto the normal of each edge of shape A
        for i in 0..points_a.len() {
            let j = (i + 1) % points_a.len();

            let normal = (points_a[j] - points_a[i]).perpendicular();

            let mut min_a = f32::MAX;
            let mut max_a = f32::MIN;
            for point in points_a {
                let q = point.dot(normal);
                min_a = min_a.min(q);
                max_a = max_a.max(q);
            }

            let mut min_b = f32::MAX;
            let mut max_b = f32::MIN;
            for point in points_b {
                let q = point.dot(normal);
                min_b = min_b.min(q);
                max_b = max_b.max(q);
            }

            if max_a < min_b || min_a > max_b {
                return None;
            }

            let depth = f32::min(max_a - min_b, max_b - min_a);
            if depth < min_depth {
                min_depth = depth;
                collision_normal = normal;
            }
        }

        points.swap(0, 1);
    }

    // All axes overlap so the shapes must intersect
    return Some(Collision::new(collision_normal.normalise(), min_depth));
}

pub fn sat_collision_circle_polygon(circle: &Circle, points: &[Vec2f]) -> Option<Collision> {
    let mut closest_point = Vec2f::ZERO;
    let mut closest_distance = f32::MAX;
    for point in points {
        let distance = (circle.centre - *point).magnitude_sq();
        if distance < closest_distance {
            closest_distance = distance;
            closest_point = *point;
        }
    }

    let normal = circle.centre - closest_point;

    let mut min_polygon = f32::MAX;
    let mut max_polygon = f32::MIN;
    for point in points {
        let q = point.dot(normal);
        min_polygon = min_polygon.min(q);
        max_polygon = max_polygon.max(q);
    }

    let min_circle = circle.centre.dot(normal) - circle.radius;
    let max_circle = circle.centre.dot(normal) + circle.radius;

    if max_polygon < min_circle || min_polygon > max_circle {
        return None;
    }

    let depth = f32::min(max_polygon - min_circle, max_circle - min_polygon);

    return Some(Collision::new(normal.normalise(), depth));
}

pub fn collision_circles(circle_a: &Circle, circle_b: &Circle) -> Option<Collision> {
    let distance_sq = (circle_a.centre - circle_b.centre).magnitude_sq();
    let radius_sum = circle_a.radius + circle_b.radius;

    if distance_sq > radius_sum * radius_sum {
        return None;
    }

    let displacement = circle_a.centre - circle_b.centre;
    let distance = displacement.magnitude();
    let depth = radius_sum - distance;

    let normal = if distance == 0.0 {
        Vec2f::new(1.0, 0.0)
    } else {
        displacement / distance
    };

    return Some(Collision::new(normal, depth));
}

pub fn collision_circle_segment(circle: &Circle, segment: &Segment) -> Option<Collision> {
    let distance_sq = segment.point_distance_sq(circle.centre);

    if distance_sq > circle.radius * circle.radius {
        return None;
    }

    let distance = distance_sq.sqrt();
    let depth = circle.radius - distance;

    let normal = segment.normal();

    return Some(Collision::new(normal, depth));
}
