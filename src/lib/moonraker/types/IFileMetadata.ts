import type { IThumbnail } from './IThumbnail';

export interface IFileMetadata {
  print_start_time: null;
  job_id: null;
  size: number;
  modified: number;
  slicer: string;
  slicer_version: string;
  layer_height: number;
  first_layer_height: number;
  object_height: number;
  filament_total: number;
  estimated_time: number;
  thumbnails: IThumbnail[];
  first_layer_bed_temp: number;
  first_layer_extr_temp: number;
  gcode_start_byte: number;
  gcode_end_byte: number;
  filename: string;
}
