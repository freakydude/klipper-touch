import type { IThumbnail } from './IThumbnail';

export interface IFileMetadata {
  thumbnails: IThumbnail[];
  estimated_time: number; // 12771
  filament_name: string; // "Geeetech PLA N0.4"
  filament_total: number; // 19084
  filament_type: string; // "PLA"
  filament_weight_total: number; // 56.96
  filename: string; // "swx2-led-light v8_prusa_PLA_L0.2_F0.97_S80.gcode"
  first_layer_bed_temp: number; // 65
  first_layer_extr_temp: number; // 210
  first_layer_height: number; // 0.2
  gcode_end_byte: number; //6482519
  gcode_start_byte: number; // 67056
  job_id: string; //  "000088"
  layer_count: number; // 85
  layer_height: number; // 0.2
  modified: number; // 1701163957.4757094
  nozzle_diameter: number; // 0.4
  object_height: number; // 17
  print_start_time: number; // 1701269472.5509927
  size: number; // 6494120
  slicer: string; //  "PrusaSlicer"
  slicer_version: string; // "2.7.0+win64"
  uuid: string; //  "7cfa8f87-13e4-4578-9ac7-e79296a2b279"
}
