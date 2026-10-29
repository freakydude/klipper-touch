//! Fans that should turn but whose tachometer stays at 0 rpm.

use super::devices::Fan;
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Time a fan gets after it was switched on before 0 rpm counts: it has to spin up, and Klipper's tachometer averages
/// over a second.
const SPIN_UP_TIME: Duration = Duration::from_secs(5);

/// Remembers since when each fan with a tachometer should turn, and marks it as stalled when it still reports 0 rpm
/// after the spin-up time.
#[derive(Debug, Default)]
pub struct FanWatch {
    turning: HashMap<String, Turning>,
}

#[derive(Debug)]
struct Turning {
    since: Instant,
    /// The spin-up time was over at the last look, so the fan has been judged.
    spun_up: bool,
}

impl FanWatch {
    pub fn apply(&mut self, fans: &mut [Fan], now: Instant) {
        self.turning.retain(|object, _| fans.iter().any(|fan| &fan.object == object && watched(fan)));
        for fan in fans.iter_mut().filter(|fan| watched(fan)) {
            let turning = self.turning.entry(fan.object.clone()).or_insert(Turning { since: now, spun_up: false });
            turning.spun_up = now.duration_since(turning.since) >= SPIN_UP_TIME;
            fan.stalled = turning.spun_up && fan.rpm.is_some_and(|rpm| rpm <= 0.0);
        }
    }

    /// A fan is still spinning up: the state has to be looked at again, even if Klipper reports nothing new.
    pub fn awaits_spin_up(&self) -> bool {
        self.turning.values().any(|turning| !turning.spun_up)
    }
}

/// Only a fan with a tachometer that Klipper drives says anything with 0 rpm.
fn watched(fan: &Fan) -> bool {
    fan.rpm.is_some() && fan.should_turn()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sources::klipper::FanKind;

    fn fan(speed: f64, rpm: Option<f64>) -> Vec<Fan> {
        vec![Fan {
            object: "fan_generic exhaust".into(),
            label: "Exhaust".into(),
            kind: FanKind::Generic,
            speed,
            rpm,
            off_below: 0.0,
            stalled: false,
        }]
    }

    fn stalled_after(watch: &mut FanWatch, start: Instant, seconds: u64, speed: f64, rpm: Option<f64>) -> bool {
        let mut fans = fan(speed, rpm);
        watch.apply(&mut fans, start + Duration::from_secs(seconds));
        fans[0].stalled
    }

    #[test]
    fn a_fan_at_0_rpm_is_stalled_only_after_the_spin_up_time() {
        let start = Instant::now();
        let mut watch = FanWatch::default();

        assert!(!stalled_after(&mut watch, start, 0, 0.5, Some(0.0)));
        assert!(!stalled_after(&mut watch, start, 4, 0.5, Some(0.0)));
        assert!(watch.awaits_spin_up());
        assert!(stalled_after(&mut watch, start, 5, 0.5, Some(0.0)));
        assert!(!watch.awaits_spin_up());
        assert!(!stalled_after(&mut watch, start, 6, 0.5, Some(1400.0)), "turning again");
    }

    #[test]
    fn switching_off_starts_the_spin_up_time_again() {
        let start = Instant::now();
        let mut watch = FanWatch::default();
        stalled_after(&mut watch, start, 0, 0.5, Some(0.0));

        assert!(!stalled_after(&mut watch, start, 6, 0.0, Some(0.0)));
        assert!(!stalled_after(&mut watch, start, 7, 0.5, Some(0.0)));
        assert!(stalled_after(&mut watch, start, 12, 0.5, Some(0.0)));
    }

    #[test]
    fn a_fan_without_tachometer_is_never_stalled() {
        let start = Instant::now();
        let mut watch = FanWatch::default();
        stalled_after(&mut watch, start, 0, 1.0, None);

        assert!(!stalled_after(&mut watch, start, 60, 1.0, None));
    }
}
