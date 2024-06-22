use bevy::prelude::Vec3;

// TODO: Move to shared, as you want to use it also on the server - but we need to de-Bevy-ise it
#[must_use]
#[allow(clippy::many_single_char_names)]
pub fn line_segment_intersection_with_sphere(
    segment: (Vec3, Vec3),
    sphere: (Vec3, f32),
) -> Vec<Vec3> {
    let (p0, p1) = segment;
    let (center, radius) = sphere;

    let d = p1 - p0;
    let f = p0 - center;

    let a = d.dot(d);
    let b = 2.0 * f.dot(d);
    let c = f.dot(f) - radius * radius;

    let discriminant = b * b - 4.0 * a * c;

    if discriminant < 0.0 {
        // No intersection
        Vec::new()
    } else {
        let discriminant_sqrt = discriminant.sqrt();

        let t1 = (-b - discriminant_sqrt) / (2.0 * a);
        let t2 = (-b + discriminant_sqrt) / (2.0 * a);

        let mut intersections = Vec::new();

        if (0.0 ..= 1.0).contains(&t1) {
            intersections.push(p0 + t1 * d);
        }

        if (0.0 ..= 1.0).contains(&t2) {
            intersections.push(p0 + t2 * d);
        }

        intersections
    }
}

#[cfg(test)]
mod tests {
    use bevy::math::Vec3;

    use super::*;

    #[test]
    fn test_no_intersection() {
        let segment = (Vec3::new(1.0, 1.0, 1.0), Vec3::new(2.0, 2.0, 2.0));
        let sphere = (Vec3::new(0.0, 0.0, 0.0), 0.5);

        let intersections = line_segment_intersection_with_sphere(segment, sphere);
        assert!(intersections.is_empty());
    }

    #[test]
    fn test_tangent_intersection() {
        let segment = (Vec3::new(1.0, 1.0, 0.0), Vec3::new(1.0, -1.0, 0.0));
        let sphere = (Vec3::new(0.0, 0.0, 0.0), 1.0);

        let intersections = line_segment_intersection_with_sphere(segment, sphere);
        // Later: We could fix this to return a single intersection point in tangent cases, but it doesn't matter
        assert_eq!(intersections[0], Vec3::new(1.0, 0.0, 0.0));
    }

    #[test]
    fn test_two_intersections_1() {
        let segment = (Vec3::new(1.0, 0.0, 0.0), Vec3::new(-1.0, 0.0, 0.0));
        let sphere = (Vec3::new(0.0, 0.0, 0.0), 1.0);

        let intersections = line_segment_intersection_with_sphere(segment, sphere);
        assert_eq!(intersections.len(), 2);
        assert!(intersections.contains(&Vec3::new(1.0, 0.0, 0.0)));
        assert!(intersections.contains(&Vec3::new(-1.0, 0.0, 0.0)));
    }

    #[test]
    fn test_two_intersections_2() {
        let segment = (Vec3::new(2.0, 0.0, 0.0), Vec3::new(-2.0, 0.0, 0.0));
        let sphere = (Vec3::new(0.0, 0.0, 0.0), 1.0);

        let intersections = line_segment_intersection_with_sphere(segment, sphere);
        assert_eq!(intersections.len(), 2);
        assert!(intersections.contains(&Vec3::new(1.0, 0.0, 0.0)));
        assert!(intersections.contains(&Vec3::new(-1.0, 0.0, 0.0)));
    }

    #[test]
    fn test_segment_inside_sphere() {
        let segment = (Vec3::new(0.5, 0.5, 0.5), Vec3::new(-0.5, -0.5, -0.5));
        let sphere = (Vec3::new(0.0, 0.0, 0.0), 1.0);

        let intersections = line_segment_intersection_with_sphere(segment, sphere);
        assert_eq!(intersections.len(), 0);
    }

    #[test]
    fn test_segment_on_sphere_surface() {
        let segment = (Vec3::new(1.0, 0.0, 0.0), Vec3::new(0.0, 1.0, 0.0));
        let sphere = (Vec3::new(0.0, 0.0, 0.0), 1.0);

        let intersections = line_segment_intersection_with_sphere(segment, sphere);
        assert_eq!(intersections.len(), 2);
    }
}
