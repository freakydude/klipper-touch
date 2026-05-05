import type { Writable } from 'svelte/store';
import type { IConfigFile } from './IConfigFile';
import type { IExtruder } from './IExtruder';
import type { IFan } from './IFan';
import type { IGCodeMove } from './IGCodeMove';
import type { IHeaterBed } from './IHeaterBed';
import type { IMotionReport } from './IMotionReport';
import type { IPrintStats } from './IPrintStats';
import type { IProgress } from './IProgress';
import type { IQuadGantryLevel } from './IQuadGantryLevel';
import type { IToolhead } from './IToolhead';
import type { IWebhooks } from './IWebhooks';

export interface IPrinterObjectDataMap {
  webhooks: IWebhooks;
  heater_bed: IHeaterBed;
  extruder: IExtruder;
  toolhead: IToolhead;
  fan: IFan;
  gcode_move: IGCodeMove;
  print_stats: IPrintStats;
  display_status: IProgress;
  motion_report: IMotionReport;
  configfile: IConfigFile;
  quad_gantry_level: IQuadGantryLevel;
}

export type TPrinterObjectName = Extract<keyof IPrinterObjectDataMap, string>;

export type TPrinterObjectField<T extends TPrinterObjectName> = Extract<keyof IPrinterObjectDataMap[T], string>;

export type TWebhooksField = TPrinterObjectField<'webhooks'>;
export type THeaterBedField = TPrinterObjectField<'heater_bed'>;
export type TExtruderField = TPrinterObjectField<'extruder'>;
export type TToolheadField = TPrinterObjectField<'toolhead'>;
export type TFanField = TPrinterObjectField<'fan'>;
export type TGCodeMoveField = TPrinterObjectField<'gcode_move'>;
export type TPrintStatsField = TPrinterObjectField<'print_stats'>;
export type TDisplayStatusField = TPrinterObjectField<'display_status'>;
export type TMotionReportField = TPrinterObjectField<'motion_report'>;
export type TConfigFileField = TPrinterObjectField<'configfile'>;
export type TQuadGantryLevelField = TPrinterObjectField<'quad_gantry_level'>;

export type IPrinterObjectsParams = {
  [K in TPrinterObjectName]: TPrinterObjectField<K>[] | null;
};

export type IPrinterObjectsSelection = {
  [K in TPrinterObjectName]?: readonly TPrinterObjectField<K>[];
};

export function definePrinterObjects<T extends IPrinterObjectsSelection>(selection: T): Partial<IPrinterObjectsParams> {
  return selection as Partial<IPrinterObjectsParams>;
}

export interface IPrinterObjectStores {
  webhooks: {
    state: Writable<IWebhooks['state']>;
    state_message: Writable<IWebhooks['state_message']>;
  };
  heater_bed: {
    temperature: Writable<IHeaterBed['temperature']>;
    target: Writable<IHeaterBed['target']>;
  };
  extruder: {
    temperature: Writable<IExtruder['temperature']>;
    target: Writable<IExtruder['target']>;
    pressure_advance: Writable<IExtruder['pressure_advance']>;
    can_extrude: Writable<IExtruder['can_extrude']>;
  };
  toolhead: {
    position: Writable<IToolhead['position']>;
    homed_axes: Writable<IToolhead['homed_axes']>;
    max_accel: Writable<IToolhead['max_accel']>;
    axis_minimum: Writable<IToolhead['axis_minimum']>;
    axis_maximum: Writable<IToolhead['axis_maximum']>;
    max_velocity: Writable<IToolhead['max_velocity']>;
    square_corner_velocity: Writable<IToolhead['square_corner_velocity']>;
    minimum_cruise_ratio: Writable<IToolhead['minimum_cruise_ratio']>;
  };
  fan: {
    speed: Writable<IFan['speed']>;
    rpm: Writable<IFan['rpm']>;
  };
  gcode_move: {
    homing_origin: Writable<IGCodeMove['homing_origin']>;
    speed: Writable<IGCodeMove['speed']>;
    speed_factor: Writable<IGCodeMove['speed_factor']>;
    extrude_factor: Writable<IGCodeMove['extrude_factor']>;
  };
  print_stats: {
    filename: Writable<IPrintStats['filename']>;
    total_duration: Writable<IPrintStats['total_duration']>;
    print_duration: Writable<IPrintStats['print_duration']>;
    filament_used: Writable<IPrintStats['filament_used']>;
    state: Writable<IPrintStats['state']>;
    message: Writable<IPrintStats['message']>;
    info: {
      current_layer: Writable<number | null>;
      total_layer: Writable<number | null>;
    };
  };
  display_status: {
    progress: Writable<IProgress['progress']>;
    message: Writable<IProgress['message']>;
  };
  motion_report: {
    live_position: Writable<IMotionReport['live_position']>;
    live_velocity: Writable<IMotionReport['live_velocity']>;
    live_extruder_velocity: Writable<IMotionReport['live_extruder_velocity']>;
    steppers: Writable<IMotionReport['steppers']>;
    trapq: Writable<IMotionReport['trapq']>;
  };
  configfile: {
    save_config_pending: Writable<IConfigFile['save_config_pending']>;
  };
  quad_gantry_level: {
    applied: Writable<IQuadGantryLevel['applied']>;
  };
}

export interface IPrinterObjects {
  objects: IPrinterObjectsParams;
}
