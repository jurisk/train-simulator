use std::time::Duration;

use crate::transport::track_length::TrackLength;

pub trait Metrics {
    fn track_pathfinding_duration(&self, duration: Duration, result: Option<(usize, TrackLength)>);
    fn track_planning_duration(&self, duration: Duration, result: Option<(usize, TrackLength)>);
}

#[expect(clippy::module_name_repetitions)]
#[derive(Default)]
pub struct NoopMetrics {}

impl Metrics for NoopMetrics {
    fn track_pathfinding_duration(
        &self,
        _duration: Duration,
        _result: Option<(usize, TrackLength)>,
    ) {
    }

    fn track_planning_duration(&self, _duration: Duration, _result: Option<(usize, TrackLength)>) {}
}
