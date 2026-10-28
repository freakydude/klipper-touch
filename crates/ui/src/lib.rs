//! freakymate-ui: the Slint UI (`ui/*.slint`) compiled to Rust, without app logic.
//!
//! A crate of its own, so changes in `freakymate-core` or the app code do not recompile the generated UI code.

slint::include_modules!();
