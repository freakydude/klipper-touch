export interface IPrinterObjectsParams {
  webhooks: ('state' | 'state_message')[];
  heater_bed: ('temperature' | 'target')[];
  extruder: ('temperature' | 'target')[];
  toolhead: ('position' | 'homed_axes')[];
  fan: 'speed'[];
  gcode_move: 'homing_origin'[];
  print_stats: ('filename' | 'total_duration' | 'print_duration' | 'filament_used' | 'state' | 'message')[];
  display_status: ('progress' | 'message')[];
}

export interface IPrinterObjects {
  objects: IPrinterObjectsParams;
}
