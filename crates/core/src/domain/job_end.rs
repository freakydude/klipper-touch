//! When the job Klipper shows ended: Klipper's `print_stats` joined with the newest job of Moonraker's history.

use crate::sources::klipper::{PrintJob, PrintState};
use crate::sources::moonraker_server::history::FinishedJob;

/// When `job` ended, if the newest history entry is that job: the same file and an outcome that matches its state.
/// Every outcome besides completed and cancelled is a failure (shutdown, disconnect, server exit).
pub fn ended_at(job: &PrintJob, newest: &FinishedJob) -> Option<f64> {
    let matches = match job.state {
        PrintState::Complete => newest.status == "completed",
        PrintState::Cancelled => newest.status == "cancelled",
        PrintState::Error => !matches!(newest.status.as_str(), "completed" | "cancelled" | "in_progress"),
        _ => false,
    };
    (matches && newest.filename == job.filename).then_some(newest.end_time)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sources::moonraker_server::history::parse_print_history;
    use serde_json::json;

    fn job(state: PrintState, filename: &str) -> PrintJob {
        PrintJob { state, filename: filename.into(), ..PrintJob::default() }
    }

    #[test]
    fn the_newest_job_tells_when_the_shown_job_ended() {
        let history = parse_print_history(&json!({ "jobs": [
            { "filename": "a.gcode", "status": "cancelled", "end_time": 1_700_000_600.0 },
            { "filename": "a.gcode", "status": "completed", "end_time": 1_600_000_000.0 }
        ] }));
        let newest = history.newest.unwrap();

        assert_eq!(ended_at(&job(PrintState::Cancelled, "a.gcode"), &newest), Some(1_700_000_600.0));
        assert_eq!(ended_at(&job(PrintState::Complete, "a.gcode"), &newest), None);
        assert_eq!(ended_at(&job(PrintState::Cancelled, "b.gcode"), &newest), None);
    }

    #[test]
    fn every_other_outcome_is_the_end_of_a_failed_job() {
        let newest = FinishedJob { filename: "a.gcode".into(), status: "klippy_shutdown".into(), end_time: 5.0 };

        assert_eq!(ended_at(&job(PrintState::Error, "a.gcode"), &newest), Some(5.0));
    }

    #[test]
    fn a_running_job_has_not_ended() {
        let newest = FinishedJob { filename: "a.gcode".into(), status: "in_progress".into(), end_time: 5.0 };

        assert_eq!(ended_at(&job(PrintState::Printing, "a.gcode"), &newest), None);
    }
}
