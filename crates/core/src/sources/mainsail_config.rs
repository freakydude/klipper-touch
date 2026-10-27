//! Macros of mainsail-config (`client.cfg`, optional) that FreakyMate calls. Macros are black boxes: they are called
//! by name with their documented parameters, and nothing is assumed about what they do inside.

use super::klipper::Setup;

/// Pauses at the next layer change: `SET_PAUSE_NEXT_LAYER [ENABLE=[0|1]] [MACRO=<name>]`.
pub const PAUSE_NEXT_LAYER_MACRO: &str = "SET_PAUSE_NEXT_LAYER";
/// Pauses when a layer is reached: `SET_PAUSE_AT_LAYER [ENABLE=[0|1]] [LAYER=<number>] [MACRO=<name>]`.
pub const PAUSE_AT_LAYER_MACRO: &str = "SET_PAUSE_AT_LAYER";

/// Which of the layer pause macros exist.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LayerPauseMacros {
    pub next_layer: bool,
    pub at_layer: bool,
}

impl LayerPauseMacros {
    /// `None` if neither macro exists.
    pub fn detect(klipper: &Setup) -> Option<Self> {
        let next_layer = klipper.has_macro(PAUSE_NEXT_LAYER_MACRO);
        let at_layer = klipper.has_macro(PAUSE_AT_LAYER_MACRO);
        (next_layer || at_layer).then_some(Self { next_layer, at_layer })
    }

    /// The call that plans a pause at the next layer; `None` without that macro.
    pub fn pause_next_layer(self) -> Option<String> {
        self.next_layer.then(|| format!("{PAUSE_NEXT_LAYER_MACRO} ENABLE=1"))
    }

    /// The call that plans a pause at `layer`; `None` without that macro.
    pub fn pause_at_layer(self, layer: u32) -> Option<String> {
        self.at_layer.then(|| format!("{PAUSE_AT_LAYER_MACRO} ENABLE=1 LAYER={layer}"))
    }

    /// The calls that take back a planned pause, whichever macro planned it.
    pub fn clear(self) -> Vec<String> {
        [(self.next_layer, PAUSE_NEXT_LAYER_MACRO), (self.at_layer, PAUSE_AT_LAYER_MACRO)]
            .into_iter()
            .filter(|(exists, _)| *exists)
            .map(|(_, name)| format!("{name} ENABLE=0"))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn detect(macros: &[&str]) -> Option<LayerPauseMacros> {
        LayerPauseMacros::detect(&Setup::with_macros(macros))
    }

    #[test]
    fn each_macro_is_detected_on_its_own() {
        assert_eq!(detect(&[]), None);
        assert_eq!(detect(&["set_pause_at_layer"]), Some(LayerPauseMacros { next_layer: false, at_layer: true }));
        assert_eq!(
            detect(&[PAUSE_NEXT_LAYER_MACRO, PAUSE_AT_LAYER_MACRO]),
            Some(LayerPauseMacros { next_layer: true, at_layer: true })
        );
    }

    #[test]
    fn only_existing_macros_are_called_with_documented_parameters() {
        let at_layer = LayerPauseMacros { next_layer: false, at_layer: true };

        assert_eq!(at_layer.pause_next_layer(), None);
        assert_eq!(at_layer.pause_at_layer(42).as_deref(), Some("SET_PAUSE_AT_LAYER ENABLE=1 LAYER=42"));
        assert_eq!(at_layer.clear(), ["SET_PAUSE_AT_LAYER ENABLE=0"]);
    }
}
