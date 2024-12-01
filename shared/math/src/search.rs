#[expect(clippy::missing_panics_doc)]
pub fn bisection_search_for_minimum<F>(min: f32, max: f32, tolerance: f32, f: F) -> f32
where
    F: Fn(f32) -> f32,
{
    assert!(min <= max, "min should be <= max");
    assert!(tolerance >= 0.0, "tolerance should be >= 0.0");

    let mut low = min;
    let mut high = max;

    while high - low > tolerance {
        let mid1 = low + (high - low) / 3.0;
        let mid2 = low + 2.0 * (high - low) / 3.0;

        if f(mid1) < f(mid2) {
            high = mid2;
        } else {
            low = mid1;
        }
    }

    (low + high) / 2.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bisection_search_for_minimum() {
        let f = |x: f32| (x - 2.0).powi(2) + 1.0; // minimum at x=2
        let min_x = bisection_search_for_minimum(-10.0, 10.0, 0.01, f);

        assert!((min_x - 2.0).abs() < 0.01);
    }
}
