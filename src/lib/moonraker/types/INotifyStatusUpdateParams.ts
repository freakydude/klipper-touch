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

export interface INotifyStatusUpdateParams {
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
  quadGantryLevel: IQuadGantryLevel
}
