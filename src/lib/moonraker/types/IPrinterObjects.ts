export interface IPrinterObjectsParams {
  webhooks: ('state' | 'state_message')[];
  heater_bed: ('temperature' | 'target')[];
  extruder: ('temperature' | 'target' | 'pressure_advance' | 'can_extrude')[];
  toolhead: ('position' | 'homed_axes' | 'max_accel' | 'axis_minimum' | 'axis_maximum')[];
  fan: ('speed' | 'rpm')[];
  gcode_move: ('homing_origin' | 'speed' | 'speed_factor' | 'extrude_factor')[];
  print_stats: ('filename' | 'total_duration' | 'print_duration' | 'filament_used' | 'state' | 'message' | 'info')[];
  display_status: ('progress' | 'message')[];
  motion_report: ('live_position' | 'live_velocity' | 'live_extruder_velocity')[];
  configfile: ('config' | 'settings' | 'save_config_pending')[];
  quad_gantry_level: 'applied'[];
}

export interface IPrinterStatsInfo {
  info: ('total_layer' | 'current_layer')[];
}

export interface IPrinterObjects {
  objects: IPrinterObjectsParams;
}
