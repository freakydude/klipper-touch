//! What Moonraker itself provides beside the Klipper objects: its components (files, job history, power, updates,
//! temperature and console stores, host information).

pub mod files;
pub mod gcode_store;
pub mod machine;
pub mod power;
pub mod server_info;
pub mod temperature_store;
pub mod update_manager;

pub use server_info::ServerComponents;
