//! Axes and moves: `toolhead`, `motion_report` and `gcode_move`.

use super::fields::{number, number_or, text, vector};
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Axis {
    X,
    Y,
    Z,
}

impl Axis {
    pub fn index(self) -> usize {
        self as usize
    }

    pub fn letter(self) -> char {
        match self {
            Self::X => 'X',
            Self::Y => 'Y',
            Self::Z => 'Z',
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct HomedAxes {
    pub x: bool,
    pub y: bool,
    pub z: bool,
}

impl HomedAxes {
    pub fn parse(text: &str) -> Self {
        let text = text.to_ascii_lowercase();
        Self { x: text.contains('x'), y: text.contains('y'), z: text.contains('z') }
    }

    pub fn contains(self, axis: Axis) -> bool {
        match axis {
            Axis::X => self.x,
            Axis::Y => self.y,
            Axis::Z => self.z,
        }
    }

    pub fn all(self) -> bool {
        self.x && self.y && self.z
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Motion {
    pub position: [f64; 4],
    pub live_position: [f64; 4],
    pub homed: HomedAxes,
    pub axis_minimum: [f64; 4],
    pub axis_maximum: [f64; 4],
    pub max_velocity: f64,
    pub max_accel: f64,
    pub square_corner_velocity: f64,
    pub minimum_cruise_ratio: f64,
    pub live_velocity: f64,
    pub live_extruder_velocity: f64,
    pub homing_origin: [f64; 4],
    pub speed_factor: f64,
    pub extrude_factor: f64,
}

pub(super) fn read_motion(toolhead: &Value, motion_report: &Value, gcode_move: &Value) -> Motion {
    Motion {
        position: vector(toolhead, "position"),
        live_position: vector(motion_report, "live_position"),
        homed: HomedAxes::parse(text(toolhead, "homed_axes")),
        axis_minimum: vector(toolhead, "axis_minimum"),
        axis_maximum: vector(toolhead, "axis_maximum"),
        max_velocity: number(toolhead, "max_velocity"),
        max_accel: number(toolhead, "max_accel"),
        square_corner_velocity: number(toolhead, "square_corner_velocity"),
        minimum_cruise_ratio: number(toolhead, "minimum_cruise_ratio"),
        live_velocity: number(motion_report, "live_velocity"),
        live_extruder_velocity: number(motion_report, "live_extruder_velocity"),
        homing_origin: vector(gcode_move, "homing_origin"),
        speed_factor: number_or(gcode_move, "speed_factor", 1.0),
        extrude_factor: number_or(gcode_move, "extrude_factor", 1.0),
    }
}
