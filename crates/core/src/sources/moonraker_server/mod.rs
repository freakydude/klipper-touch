//! What Moonraker itself provides beside the Klipper objects: its components (files, job history, power, updates,
//! temperature and console stores, host information, WLED strips).

pub mod files;
pub mod gcode_store;
pub mod history;
pub mod machine;
pub mod power;
pub mod server_info;
pub mod temperature_store;
pub mod update_manager;
pub mod wled;

pub use server_info::ServerComponents;
