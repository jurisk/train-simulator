use std::time::Duration;

use bevy::prelude::Resource;
use metrics::{metadata_var, Histogram, Key, Level, Recorder};
use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle, PrometheusRecorder};
use shared_domain::metrics::Metrics;
use shared_domain::transport::track_length::TrackLength;

// Safe to clone because everything inside is wrapped in `Arc`
#[derive(Clone, Resource)]
pub struct PrometheusMetrics {
    prometheus_handle:           PrometheusHandle,
    track_pathfinding_histogram: Histogram,
    track_planning_histogram:    Histogram,
}

impl Metrics for PrometheusMetrics {
    fn track_pathfinding_duration(
        &self,
        duration: Duration,
        _result: Option<(usize, TrackLength)>,
    ) {
        self.track_pathfinding_histogram.record(duration);
    }

    fn track_planning_duration(&self, duration: Duration, _result: Option<(usize, TrackLength)>) {
        self.track_planning_histogram.record(duration);
    }
}

impl PrometheusMetrics {
    fn create_histogram(recorder: &PrometheusRecorder, name: &'static str) -> Histogram {
        let key = Key::from_static_name(name);
        let metadata = metadata_var!(module_path!(), Level::INFO);
        recorder.register_histogram(&key, metadata)
    }

    #[must_use]
    pub fn new() -> Self {
        let recorder = PrometheusBuilder::new().build_recorder();
        let prometheus_handle = recorder.handle();
        let track_pathfinding_histogram =
            Self::create_histogram(&recorder, "track_pathfinding_duration");
        let track_planning_histogram = Self::create_histogram(&recorder, "track_planning_duration");

        Self {
            prometheus_handle,
            track_pathfinding_histogram,
            track_planning_histogram,
        }
    }

    #[must_use]
    pub fn render(&self) -> String {
        self.prometheus_handle.render()
    }
}
