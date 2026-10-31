//! The height of the layer being printed, from the Z of the G-code file (`gcode_move.gcode_position`).
//!
//! A Z hop lifts the nozzle for a moment on travel moves, so the height is the lowest Z of the last seconds. A pause
//! lifts it too (the `PAUSE` macro parks the head): the height stays as it was before, which is where the print goes
//! on.

use super::job::{PrintJob, PrintState};
use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// Longer than a Z hop on a travel move; a new layer shows this much later.
const WINDOW: Duration = Duration::from_secs(2);

/// The Z values of the last [`WINDOW`] while printing; forgotten when the job ends.
#[derive(Debug, Default)]
pub struct HeightWatch {
    samples: VecDeque<(Instant, f64)>,
}

impl HeightWatch {
    pub fn apply(&mut self, job: &mut PrintJob, z: f64, now: Instant) {
        match job.state {
            PrintState::Printing => {
                self.samples.push_back((now, z));
                while self.samples.front().is_some_and(|(time, _)| now.duration_since(*time) > WINDOW) {
                    self.samples.pop_front();
                }
            }
            // The height before the pause stays and counts as fresh, so the parked head does not show on resume.
            PrintState::Paused => {
                if let Some(height) = self.lowest() {
                    self.samples = VecDeque::from([(now, height)]);
                }
            }
            _ => self.samples.clear(),
        }
        job.height = self.lowest();
    }

    fn lowest(&self) -> Option<f64> {
        self.samples.iter().map(|(_, z)| *z).reduce(f64::min)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Run {
        watch: HeightWatch,
        start: Instant,
    }

    impl Run {
        fn new() -> Self {
            Self { watch: HeightWatch::default(), start: Instant::now() }
        }

        fn at(&mut self, millis: u64, state: PrintState, z: f64) -> Option<f64> {
            let mut job = PrintJob { state, ..PrintJob::default() };
            self.watch.apply(&mut job, z, self.start + Duration::from_millis(millis));
            job.height
        }
    }

    #[test]
    fn a_z_hop_does_not_lift_the_height() {
        let mut run = Run::new();
        run.at(0, PrintState::Printing, 2.4);

        assert_eq!(run.at(300, PrintState::Printing, 2.8), Some(2.4));
        assert_eq!(run.at(600, PrintState::Printing, 2.4), Some(2.4));
    }

    #[test]
    fn a_new_layer_shows_once_the_window_has_passed() {
        let mut run = Run::new();
        run.at(0, PrintState::Printing, 2.4);
        run.at(500, PrintState::Printing, 2.6);

        assert_eq!(run.at(1500, PrintState::Printing, 2.6), Some(2.4));
        assert_eq!(run.at(2600, PrintState::Printing, 2.6), Some(2.6));
    }

    #[test]
    fn the_height_before_a_pause_stays_while_parked_and_on_resume() {
        let mut run = Run::new();
        run.at(0, PrintState::Printing, 10.0);

        assert_eq!(run.at(60_000, PrintState::Paused, 30.0), Some(10.0));
        assert_eq!(run.at(60_500, PrintState::Printing, 30.0), Some(10.0));
        assert_eq!(run.at(61_000, PrintState::Printing, 10.0), Some(10.0));
    }

    #[test]
    fn there_is_no_height_without_a_running_job() {
        let mut run = Run::new();
        run.at(0, PrintState::Printing, 10.0);

        assert_eq!(run.at(100, PrintState::Complete, 50.0), None);
        assert_eq!(run.at(200, PrintState::Standby, 0.0), None);
    }
}
