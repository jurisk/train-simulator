// Later: This is clumsy, not sure how to improve this.
// We minimise the first tuple value, but also return the second tuple values.
#[expect(clippy::missing_panics_doc)]
pub fn bisection_search_for_minimum<F, T>(
    min: f32,
    max: f32,
    tolerance: f32,
    f: F,
) -> (f32, (Option<T>, Option<T>))
where
    F: Fn(f32) -> (f32, T),
{
    assert!(min <= max, "min should be <= max");
    assert!(tolerance >= 0.0, "tolerance should be >= 0.0");

    let mut low = min;
    let mut low_t = None;
    let mut high = max;
    let mut high_t = None;

    while high - low > tolerance {
        let mid1 = low + (high - low) / 3.0;
        let mid2 = low + 2.0 * (high - low) / 3.0;

        let (f_mid1, f_mid1_t) = f(mid1);
        let (f_mid2, f_mid2_t) = f(mid2);
        if f_mid1 < f_mid2 {
            high = mid2;
            high_t = Some(f_mid2_t);
        } else {
            low = mid1;
            low_t = Some(f_mid1_t);
        }
    }

    ((low + high) / 2.0, (low_t, high_t))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bisection_search_for_minimum() {
        let f = |x: f32| ((x - 2.0).powi(2) + 1.0, 0.0); // minimum at x=2
        let (min_x, _) = bisection_search_for_minimum(-10.0, 10.0, 0.01, f);

        assert!((min_x - 2.0).abs() < 0.01);
    }
}
