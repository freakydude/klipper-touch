export interface IPrinterObjectsParams {
  webhooks: ('state' | 'state_message')[];
  heater_bed: ('temperature' | 'target')[];
  extruder: ('temperature' | 'target' | 'pressure_advance')[];
  toolhead: ('position' | 'homed_axes' | 'max_accel')[];
  fan: ('speed' | 'rpm')[];
  gcode_move: ('homing_origin' | 'speed' | 'speed_factor' | 'extrude_factor')[];
  print_stats: ('filename' | 'total_duration' | 'print_duration' | 'filament_used' | 'state' | 'message' | 'info')[];
  display_status: ('progress' | 'message')[];
}

export interface IPrinterStatsInfo {
  info: ('total_layer' | 'current_layer')[];
}

export interface IPrinterObjects {
  objects: IPrinterObjectsParams;
}
