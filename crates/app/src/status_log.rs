//! Messages of the status bar and its flyout: errors and informations from Klipper and from the app.
//!
//! Pure logic without Slint; the presenter puts the result on screen.

use std::collections::VecDeque;
use std::time::Duration;

const LIMIT: usize = 50;
/// Klipper reports a rejected command twice: as RPC error and as `!!` response. A response this close to the
/// failure and repeating its reason is the same event.
const DUPLICATE_WINDOW_SECONDS: f64 = 2.0;
/// An information stays in the status bar for a base time plus the time to read it.
const INFO_BASE: Duration = Duration::from_secs(5);
/// Relaxed reading is about 13 to 15 characters per second.
const INFO_PER_CHARACTER: Duration = Duration::from_millis(70);
const MAX_INFO_DISPLAY: Duration = Duration::from_secs(15);

/// Layout of a message row in the flyout, in logical pixels: the text over the full width, the time small below it.
pub const ROW_GAP: f32 = 6.0;
const LINE_HEIGHT: f32 = 22.0;
const ROW_PADDING: f32 = 20.0;
const TIME_LINE_HEIGHT: f32 = 16.0;
/// Roughly what fits into the width of the flyout at the normal font size.
const CHARACTERS_PER_LINE: usize = 48;
const MAX_LINES: usize = 10;

#[derive(Debug, Clone, PartialEq)]
pub struct Message {
    pub id: u64,
    /// The whole text; a multi-line message stays one entry.
    pub text: String,
    pub is_error: bool,
    /// Unix time in seconds.
    pub time: f64,
}

impl Message {
    /// The first line, which the status bar shows.
    pub fn headline(&self) -> &str {
        self.text.lines().next().unwrap_or_default()
    }
}

#[derive(Debug, Default)]
pub struct StatusLog {
    messages: VecDeque<Message>,
    next_id: u64,
    /// The message the status bar shows.
    badge: Option<u64>,
    /// Errors with a smaller id were seen in the flyout.
    seen_before: u64,
    /// Reason of the last rejected command, to recognise its duplicate response.
    last_failure: Option<(String, f64)>,
}

impl StatusLog {
    /// Adds a message and makes it the one the status bar shows. Returns its id.
    pub fn add(&mut self, text: &str, is_error: bool, now: f64) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.messages.push_back(Message { id, text: text.to_owned(), is_error, time: now });
        if self.messages.len() > LIMIT {
            self.messages.pop_front();
        }
        self.badge = Some(id);
        id
    }

    /// A command of the app was rejected. `shown` goes to the list, `reason` is what Klipper repeats in its response.
    pub fn add_command_failure(&mut self, shown: &str, reason: &str, now: f64) -> u64 {
        self.last_failure = Some((reason.to_owned(), now));
        self.add(shown, true, now)
    }

    /// A `!!` or `//` response of Klipper. Returns `None` for an error that repeats the failure just reported.
    pub fn add_response(&mut self, text: &str, is_error: bool, now: f64) -> Option<u64> {
        if is_error && self.repeats_last_failure(text, now) {
            return None;
        }
        Some(self.add(text, is_error, now))
    }

    fn repeats_last_failure(&self, text: &str, now: f64) -> bool {
        self.last_failure
            .as_ref()
            .is_some_and(|(reason, time)| now - time <= DUPLICATE_WINDOW_SECONDS && text.contains(reason.as_str()))
    }

    /// The message the status bar shows, if any.
    pub fn badge(&self) -> Option<&Message> {
        let id = self.badge?;
        self.messages.iter().find(|message| message.id == id)
    }

    /// All messages, newest first.
    pub fn messages(&self) -> impl Iterator<Item = &Message> {
        self.messages.iter().rev()
    }

    /// The flyout was opened: the status bar stops showing the message, the list keeps it.
    pub fn mark_seen(&mut self) {
        self.badge = None;
        self.seen_before = self.next_id;
    }

    /// An information ran out of display time. Hides it only while it is still the one shown; an error that
    /// nobody has seen yet comes back, because errors stay until they are seen.
    pub fn hide_badge(&mut self, id: u64) {
        if self.badge == Some(id) {
            self.badge = self.newest_unseen_error();
        }
    }

    fn newest_unseen_error(&self) -> Option<u64> {
        self.messages
            .iter()
            .rev()
            .find(|message| message.is_error && message.id >= self.seen_before)
            .map(|message| message.id)
    }

    pub fn clear(&mut self) {
        self.messages.clear();
        self.badge = None;
        self.seen_before = self.next_id;
    }
}

/// Height of a message row: one line per wrapped line of the text, but never below the minimum.
pub fn row_height(text: &str) -> f32 {
    let lines: usize = text.lines().map(|line| line.chars().count().div_ceil(CHARACTERS_PER_LINE).max(1)).sum();
    lines.clamp(1, MAX_LINES) as f32 * LINE_HEIGHT + ROW_PADDING + TIME_LINE_HEIGHT
}

/// How long an information stays in the status bar.
pub fn info_display_time(text: &str) -> Duration {
    let characters = u32::try_from(text.chars().count()).unwrap_or(u32::MAX);
    (INFO_BASE + INFO_PER_CHARACTER * characters).min(MAX_INFO_DISPLAY)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn newest_message_is_shown_and_listed_first() {
        let mut log = StatusLog::default();
        log.add("first", true, 1.0);
        log.add("second", false, 2.0);

        assert_eq!(log.badge().map(Message::headline), Some("second"));
        let listed: Vec<&str> = log.messages().map(|message| message.text.as_str()).collect();
        assert_eq!(listed, ["second", "first"]);
    }

    #[test]
    fn marking_seen_hides_the_badge_but_keeps_the_list() {
        let mut log = StatusLog::default();
        log.add("error", true, 1.0);

        log.mark_seen();

        assert!(log.badge().is_none());
        assert_eq!(log.messages().count(), 1);
    }

    #[test]
    fn hiding_an_old_information_keeps_a_newer_one() {
        let mut log = StatusLog::default();
        let old = log.add("old info", false, 1.0);
        log.add("error", true, 2.0);

        log.hide_badge(old);

        assert_eq!(log.badge().map(Message::headline), Some("error"));
    }

    #[test]
    fn response_repeating_a_failure_is_skipped() {
        let mut log = StatusLog::default();
        log.add_command_failure("G1 X10: Must home axis first", "Must home axis first", 10.0);

        assert!(log.add_response("Must home axis first: 0 0", true, 10.5).is_none());
        assert_eq!(log.messages().count(), 1);
    }

    #[test]
    fn repeat_after_the_window_is_a_new_message() {
        let mut log = StatusLog::default();
        log.add_command_failure("G1 X10: Must home axis first", "Must home axis first", 10.0);

        assert!(log.add_response("Must home axis first", true, 20.0).is_some());
    }

    #[test]
    fn informations_are_never_skipped_as_duplicates() {
        let mut log = StatusLog::default();
        log.add_command_failure("x", "Must home", 10.0);

        assert!(log.add_response("Must home", false, 10.1).is_some());
    }

    #[test]
    fn list_is_limited_to_the_newest_messages() {
        let mut log = StatusLog::default();
        for index in 0..60 {
            log.add(&format!("message {index}"), true, index as f64);
        }

        assert_eq!(log.messages().count(), LIMIT);
        assert_eq!(log.messages().last().map(|message| message.text.as_str()), Some("message 10"));
    }

    #[test]
    fn multi_line_message_is_one_entry_with_its_first_line_in_the_status() {
        let mut log = StatusLog::default();
        log.add("Available commands:\nG0, G1\nG28", false, 1.0);

        assert_eq!(log.messages().count(), 1);
        assert_eq!(log.badge().map(Message::headline), Some("Available commands:"));
    }

    #[test]
    fn information_time_grows_with_length_up_to_a_limit() {
        assert_eq!(info_display_time(""), Duration::from_secs(5));
        assert_eq!(info_display_time(&"x".repeat(25)), Duration::from_millis(6750));
        assert_eq!(info_display_time(&"x".repeat(1000)), Duration::from_secs(15));
    }

    #[test]
    fn row_height_grows_with_wrapped_lines_up_to_a_limit() {
        let frame = ROW_PADDING + TIME_LINE_HEIGHT;
        assert_eq!(row_height("short"), 22.0 + frame);
        assert_eq!(row_height("line one\nline two\nline three"), 3.0 * 22.0 + frame);
        assert_eq!(row_height(&"x".repeat(100)), 3.0 * 22.0 + frame);
        assert_eq!(row_height(&"x\n".repeat(30)), 10.0 * 22.0 + frame);
    }

    #[test]
    fn clear_empties_list_and_badge() {
        let mut log = StatusLog::default();
        log.add("error", true, 1.0);

        log.clear();

        assert!(log.badge().is_none());
        assert_eq!(log.messages().count(), 0);
    }

    #[test]
    fn an_unseen_error_returns_when_a_later_information_runs_out() {
        let mut log = StatusLog::default();
        log.add("fault", true, 1.0);
        let info = log.add("note", false, 2.0);
        assert_eq!(log.badge().map(Message::headline), Some("note"));

        log.hide_badge(info);

        assert_eq!(log.badge().map(Message::headline), Some("fault"));
    }

    #[test]
    fn a_seen_error_does_not_return() {
        let mut log = StatusLog::default();
        log.add("fault", true, 1.0);
        log.mark_seen();
        let info = log.add("note", false, 2.0);

        log.hide_badge(info);

        assert!(log.badge().is_none());
    }

    #[test]
    fn a_new_error_is_unseen_even_after_an_earlier_one_was_seen() {
        let mut log = StatusLog::default();
        log.add("old fault", true, 1.0);
        log.mark_seen();
        log.add("new fault", true, 2.0);
        let info = log.add("note", false, 3.0);

        log.hide_badge(info);

        assert_eq!(log.badge().map(Message::headline), Some("new fault"));
    }
}
