use super::{Segment, Shape};

pub fn overlaps(shape_a: &impl Shape, shape_b: &impl Shape) -> bool {
    let mut points = [shape_a.points(), shape_b.points()];

    // Run through twice, first with shape A as the reference and then with shape B
    for _ in 0..2 {
        let points_a = points[0];
        let points_b = points[1];

        // Project each point of each shape onto the normal of each edge of shape A
        for i in 0..points_a.len() {
            let j = (i + 1) % points_a.len();

            let normal = Segment::new(points_a[i], points_a[j])
                .direction()
                .perpendicular();

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
                return false;
            }
        }

        points.swap(0, 1);
    }

    // All axes overlap so the shapes must intersect
    return true;
}

#[cfg(test)]
mod tests {
    use crate::{geometry::Triangle, linear::Vec2f};

    use super::*;

    #[test]
    fn overlaps_test() {
        let triangle = Triangle::new(
            Vec2f::new(0.0, 0.0),
            Vec2f::new(1.0, 0.0),
            Vec2f::new(0.0, 1.0),
        );

        // intersects
        let segment = Segment::new(Vec2f::new(0.2, 0.2), Vec2f::new(2.0, 2.0));
        assert!(overlaps(&triangle, &segment));

        // does not intersect
        let segment = Segment::new(Vec2f::new(2.0, 2.0), Vec2f::new(4.0, 4.0));
        assert!(!overlaps(&triangle, &segment));
    }
}
