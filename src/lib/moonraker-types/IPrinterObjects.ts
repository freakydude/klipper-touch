export interface IPrinterObjectsParams {
  heater_bed: ('temperature' | 'target')[];
  extruder: ('temperature' | 'target')[];
  toolhead: ('position' | 'homed_axes')[];
  fan: 'speed'[];
  gcode_move: 'homing_origin'[];
  print_stats: ('filename' | 'state' | 'message')[];
  display_status: 'progress'[];
}

export interface IPrinterObjects {
  objects: IPrinterObjectsParams;
}
