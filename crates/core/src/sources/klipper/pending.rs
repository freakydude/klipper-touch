//! Heater targets sent but not yet reported by Klipper.

use super::status::Heater;
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// A sent target that Klipper has not reported back by then is given up (e.g. rejected command).
const PENDING_TARGET_TIMEOUT: Duration = Duration::from_secs(3);

/// Shows a sent target at once and lets quick taps on + and − build on each other, until Klipper reports it back.
#[derive(Debug, Default)]
pub struct PendingTargets {
    targets: HashMap<String, (f64, Instant)>,
}

impl PendingTargets {
    pub fn request(&mut self, object: &str, target: f64, now: Instant) {
        self.targets.insert(object.to_owned(), (target, now));
    }

    /// Shows pending targets on the reported heaters; forgets confirmed and outdated ones.
    pub fn apply(&mut self, heaters: &mut [Heater], now: Instant) {
        self.targets.retain(|object, (target, sent)| {
            now.duration_since(*sent) < PENDING_TARGET_TIMEOUT
                && heaters.iter().any(|heater| &heater.object == object && heater.target != *target)
        });
        for heater in heaters {
            if let Some((target, _)) = self.targets.get(&heater.object) {
                heater.target = *target;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pending_targets_cover_the_delay_until_klipper_reports_back() {
        let start = Instant::now();
        let reported = |target| vec![Heater { object: "extruder".into(), target, ..Heater::default() }];
        let mut pending = PendingTargets::default();
        pending.request("extruder", 45.0, start);
        pending.request("extruder", 50.0, start);

        let mut heaters = reported(45.0);
        pending.apply(&mut heaters, start + Duration::from_millis(200));
        assert_eq!(heaters[0].target, 50.0);

        let mut heaters = reported(50.0);
        pending.apply(&mut heaters, start + Duration::from_millis(400));
        pending.request("extruder", 55.0, start + Duration::from_millis(500));
        let mut heaters = reported(50.0);
        pending.apply(&mut heaters, start + PENDING_TARGET_TIMEOUT + Duration::from_secs(1));
        assert_eq!(heaters[0].target, 50.0);
    }
}
