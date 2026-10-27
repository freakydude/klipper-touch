//! Questions that need more than one source, such as how to load filament or how long a print will take. Each
//! question is answered in exactly one place, which also decides which source wins.

pub mod estimates;
pub mod filament;
pub mod pause;
pub mod spools;
