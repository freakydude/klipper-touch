//! Fits the layout, which is made for 800x480, to the real display: until the pages respond to the screen size,
//! every other resolution shows the same layout scaled up or down.

use slint::platform::WindowEvent;
use slint::{LogicalSize, Window};

/// The size the pages are laid out for: the BTT HDMI5.
pub const DESIGN_WIDTH: f32 = 800.0;
pub const DESIGN_HEIGHT: f32 = 480.0;

/// The largest factor at which the design size still fits into the display. The window then is as large as the
/// design size on one axis and larger on the other, where the pages simply have more room. `None` without a size.
pub fn fit_scale(width: u32, height: u32) -> Option<f32> {
    (width > 0 && height > 0).then(|| (width as f32 / DESIGN_WIDTH).min(height as f32 / DESIGN_HEIGHT))
}

/// Scales the window to its current size. Does nothing at the design size, so the primary display is not touched.
pub fn apply_fit(window: &Window) {
    let size = window.size();
    let Some(scale) = fit_scale(size.width, size.height) else { return };
    if (scale - window.scale_factor()).abs() < 0.001 {
        return;
    }
    let logical = LogicalSize::new(size.width as f32 / scale, size.height as f32 / scale);
    for event in [WindowEvent::ScaleFactorChanged { scale_factor: scale }, WindowEvent::Resized { size: logical }] {
        if let Err(error) = window.dispatch_event_with_result(event) {
            log::warn!("cannot fit the display ({}x{}): {error}", size.width, size.height);
            return;
        }
    }
    log::info!("display {}x{} fitted with scale factor {scale:.3}", size.width, size.height);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_design_size_needs_no_scaling() {
        assert_eq!(fit_scale(800, 480), Some(1.0));
    }

    #[test]
    fn other_displays_are_scaled_by_the_axis_that_limits() {
        // 7" 1024x600: the height limits (1.25 against 1.28).
        assert_eq!(fit_scale(1024, 600), Some(1.25));
        // 4.3" 480x272: the height limits.
        assert!((fit_scale(480, 272).unwrap() - 0.566_67).abs() < 1e-4);
        assert_eq!(fit_scale(640, 400).map(|scale| (scale * 100.0).round()), Some(80.0));
        assert_eq!(fit_scale(1280, 720), Some(1.5));
    }

    #[test]
    fn the_scaled_window_is_never_smaller_than_the_design_size() {
        for (width, height) in [(1024, 600), (480, 272), (640, 400), (1920, 1080), (800, 400), (500, 480)] {
            let scale = fit_scale(width, height).unwrap();
            assert!(width as f32 / scale >= DESIGN_WIDTH - 0.01, "{width}x{height}");
            assert!(height as f32 / scale >= DESIGN_HEIGHT - 0.01, "{width}x{height}");
        }
    }

    #[test]
    fn an_empty_window_has_no_scale() {
        assert_eq!(fit_scale(0, 480), None);
        assert_eq!(fit_scale(800, 0), None);
    }
}
