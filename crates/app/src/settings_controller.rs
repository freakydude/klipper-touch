//! Connects `UiSettings` with the settings file and applies changes immediately.

use crate::presenter::Presenter;
use crate::{AppWindow, PresetItem, UiSettings};
use klipper_touch_core::settings::{Settings, SettingsStore, TemperaturePreset};
use klipper_touch_core::{Endpoint, PrinterService};
use slint::{ComponentHandle, Model, ModelRc, VecModel};
use std::cell::RefCell;
use std::rc::Rc;

pub struct SettingsController {
    window: slint::Weak<AppWindow>,
    store: SettingsStore,
    /// File contents; an address from the command line is not saved.
    settings: RefCell<Settings>,
    service: PrinterService,
    presenter: Rc<Presenter>,
    presets: Rc<VecModel<PresetItem>>,
}

impl SettingsController {
    pub fn bind(
        window: &AppWindow,
        store: SettingsStore,
        settings: Settings,
        endpoint: &Endpoint,
        service: PrinterService,
        presenter: Rc<Presenter>,
    ) -> Rc<Self> {
        let controller = Rc::new(Self {
            window: window.as_weak(),
            store,
            settings: RefCell::new(settings),
            service,
            presenter,
            presets: Rc::default(),
        });
        controller.show(window, endpoint);
        controller.connect_callbacks(window);
        controller
    }

    pub fn preset(&self, index: usize) -> Option<TemperaturePreset> {
        self.settings.borrow().temperature_presets.get(index).cloned()
    }

    pub fn fullscreen(&self) -> bool {
        self.settings.borrow().fullscreen
    }

    fn show(&self, window: &AppWindow, endpoint: &Endpoint) {
        let ui = window.global::<UiSettings>();
        let settings = self.settings.borrow();
        ui.set_moonraker(endpoint.display().into());
        ui.set_fullscreen(settings.fullscreen);
        ui.set_move_step(settings.steps.move_mm as f32);
        ui.set_z_offset_step(settings.steps.z_offset_mm as f32);
        ui.set_temperature_step(settings.steps.temperature as f32);
        ui.set_extrude_length(settings.steps.extrude_mm as f32);
        ui.set_extrude_speed(settings.steps.extrude_speed as f32);
        ui.set_settings_file(self.store.path().display().to_string().into());
        ui.set_version(env!("CARGO_PKG_VERSION").into());
        self.presets.set_vec(settings.temperature_presets.iter().map(preset_item).collect::<Vec<_>>());
        ui.set_presets(ModelRc::from(self.presets.clone()));
    }

    fn connect_callbacks(self: &Rc<Self>, window: &AppWindow) {
        let ui = window.global::<UiSettings>();
        let controller = Rc::downgrade(self);
        ui.on_steps_changed(move || {
            if let Some(controller) = controller.upgrade() {
                controller.store_steps();
            }
        });
        let controller = Rc::downgrade(self);
        ui.on_moonraker_entered(move |address| {
            if let Some(controller) = controller.upgrade() {
                controller.change_moonraker(&address);
            }
        });
        let controller = Rc::downgrade(self);
        ui.on_fullscreen_toggled(move |fullscreen| {
            if let Some(controller) = controller.upgrade() {
                controller.change_fullscreen(fullscreen);
            }
        });
        let controller = Rc::downgrade(self);
        ui.on_preset_changed(move |index, extruder, bed| {
            if let Some(controller) = controller.upgrade() {
                controller.change_preset(index, f64::from(extruder), f64::from(bed));
            }
        });
    }

    fn store_steps(&self) {
        let Some(window) = self.window.upgrade() else { return };
        let ui = window.global::<UiSettings>();
        self.update(|settings| {
            settings.steps.move_mm = f64::from(ui.get_move_step());
            settings.steps.z_offset_mm = f64::from(ui.get_z_offset_step());
            settings.steps.temperature = f64::from(ui.get_temperature_step());
            settings.steps.extrude_mm = f64::from(ui.get_extrude_length());
            settings.steps.extrude_speed = f64::from(ui.get_extrude_speed());
        });
    }

    fn change_moonraker(&self, address: &str) {
        match Endpoint::parse(address) {
            Ok(endpoint) => {
                let display = endpoint.display();
                self.update(|settings| settings.moonraker.clone_from(&display));
                if let Some(window) = self.window.upgrade() {
                    window.global::<UiSettings>().set_moonraker(display.as_str().into());
                }
                self.service.connect_to(endpoint);
                self.presenter.show_notice(&format!("Connecting to {display}"), false);
            }
            Err(error) => self.presenter.show_notice(&error.to_string(), true),
        }
    }

    fn change_fullscreen(&self, fullscreen: bool) {
        if let Some(window) = self.window.upgrade() {
            window.window().set_fullscreen(fullscreen);
        }
        self.update(|settings| settings.fullscreen = fullscreen);
    }

    fn change_preset(&self, index: i32, extruder: f64, bed: f64) {
        let Ok(index) = usize::try_from(index) else { return };
        self.update(|settings| {
            if let Some(preset) = settings.temperature_presets.get_mut(index) {
                preset.extruder = extruder.max(0.0);
                preset.bed = bed.max(0.0);
            }
        });
        if let Some(preset) = self.preset(index) {
            if index < self.presets.row_count() {
                self.presets.set_row_data(index, preset_item(&preset));
            }
        }
    }

    fn update(&self, change: impl FnOnce(&mut Settings)) {
        let mut settings = self.settings.borrow_mut();
        change(&mut settings);
        if let Err(error) = self.store.save(&settings) {
            log::warn!("{error}");
            self.presenter.show_notice(&error.to_string(), true);
        }
    }
}

fn preset_item(preset: &TemperaturePreset) -> PresetItem {
    PresetItem { name: preset.name.as_str().into(), extruder: preset.extruder as f32, bed: preset.bed as f32 }
}
