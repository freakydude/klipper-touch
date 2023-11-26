import type { JsonRpcClient } from '$lib/jsonrpc/JsonRpcClient';
import type { IJsonRpcRequest } from '$lib/jsonrpc/types/IJsonRpcRequest';
import { JsonRpcRequest } from '$lib/jsonrpc/types/JsonRpcRequest';
import { ConfigFile } from './modules/ConfigFile';
import { DisplayStatus } from './modules/DisplayStatus';
import { Extruder } from './modules/Extruder';
import { Fan } from './modules/Fan';
import { GCodeMove } from './modules/GCodeMove';
import { HeaterBed } from './modules/HeaterBed';
import { KlipperState } from './modules/KlipperState';
import { MotionReport } from './modules/MotionReport';
import { PrintStats } from './modules/PrintStats';
import { QuadGantryLevel } from './modules/QuadGantryLevel';
import { Toolhead } from './modules/Toolhead';
import type { INotifyStatusUpdateParams } from './types/INotifyStatusUpdateParams';
import type { IPrinterObjects } from './types/IPrinterObjects';

export class MoonrakerClient extends EventTarget {
  _jsonRpcClient: JsonRpcClient;

  subscription: IPrinterObjects = {
    objects: {
      webhooks: ['state', 'state_message'],
      heater_bed: ['temperature', 'target'],
      extruder: ['temperature', 'target', 'pressure_advance', 'can_extrude'],
      toolhead: ['position', 'homed_axes', 'max_accel', 'axis_maximum', 'axis_minimum'],
      fan: ['speed'],
      gcode_move: ['homing_origin', 'speed', 'speed_factor', 'extrude_factor'],
      print_stats: ['filename', 'print_duration', 'filament_used', 'state', 'message', 'info'],
      display_status: ['progress', 'message'],
      motion_report: ['live_position', 'live_velocity', 'live_extruder_velocity'],
      configfile: ['save_config_pending'],
      quad_gantry_level: ['applied']
    }
  };

  public klippyState = new KlipperState();
  public heaterBed = new HeaterBed();
  public extruder = new Extruder();
  public toolhead = new Toolhead();
  public gcodeMove = new GCodeMove();
  public fan = new Fan();
  public printStats = new PrintStats();
  public displayStatus = new DisplayStatus();
  public motionReport = new MotionReport();
  public configFile = new ConfigFile();
  public quadGantryLevel = new QuadGantryLevel();

  public constructor(jsonRpcClient: JsonRpcClient) {
    super();
    this._jsonRpcClient = jsonRpcClient;

    this.attachToEvents();
  }

  public async connect(url: string | URL): Promise<boolean> {
    let successful = false;

    try {
      successful = await this._jsonRpcClient.connect(url);

      if (successful) {
        successful = (await this.requestIdentifyConnection()) && (await this.subscribeAndParseParams(this.subscription));
      }
    } catch (error) {
      console.error('MoonrakerClient.connect() ', error);
      successful = false;
    }

    return successful;
  }

  public async disconnect(): Promise<boolean> {
    let successful = false;

    try {
      successful = await this._jsonRpcClient.disconnect();
    } catch (error) {
      console.error('MoonrakerClient.disconnect() ', error);
      successful = false;
    }

    return successful;
  }

  private async requestIdentifyConnection(): Promise<boolean> {
    const identifyConnectionRequest = new JsonRpcRequest({
      method: 'server.connection.identify',
      params: {
        client_name: 'klipper-touch',
        version: '0.1.0',
        type: 'display',
        url: 'https://github.com/freakydude/klipper-touch'
      }
    });

    let successful = true;

    try {
      await this._jsonRpcClient.sendRequest(identifyConnectionRequest);
      console.log('MoonrakerClient.requestIdentifyConnection');
    } catch (error) {
      successful = false;
      console.error('MoonrakerClient.requestIdentifyConnection() ', error);
    }

    return successful;
  }

  private async subscribeAndParseParams(printerObjects: IPrinterObjects): Promise<boolean> {
    const subscribeRequest = new JsonRpcRequest({
      method: 'printer.objects.subscribe',
      params: printerObjects
    });
    let successful = true;
    try {
      const response = await this._jsonRpcClient.sendRequest(subscribeRequest);
      console.log('MoonrakerClient.subscribeToPrinterObjects');
      if (response.result !== undefined) {
        this.parseNotifyStatusUpdateParams(response.result.status);
      } else if (response.error) {
        successful = false;
        this.klippyState.state.set('error');
        this.klippyState.message.set(response.error.message);
        console.warn('Error on subscribeToPrinterObjects: ', response);
      }
    } catch (error) {
      successful = false;
      console.error('MoonrakerClient.subscribeToPrinterObjects() ', error);
    }

    return successful;
  }

  protected attachToEvents() {
    this._jsonRpcClient.addEventListener('notification', (event: Event) => {
      this.parseNotification(event as CustomEvent<IJsonRpcRequest>);
    });

    // this._jsonRpcClient.isConnected.subscribe((value) => {
    //   this.rpcClientIsConnectedChanged(value);
    // });
  }

  // private rpcClientIsConnectedChanged(value: boolean) {
  //   if (!value) {
  //     this.klippyState.state.set('disconnected');
  //   }
  // }

  private parseNotifyStatusUpdateParams(param: INotifyStatusUpdateParams) {
    this.parseWebhooks(param);
    this.parseHeaterBed(param);
    this.parseExtruder(param);
    this.parseToolhead(param);
    this.parseGcodeMove(param);
    this.parseFan(param);
    this.parsePrintStats(param);
    this.parseDisplayStatus(param);
    this.parseMotionReport(param);
    this.parseConfigFile(param);
    this.parseQuadGantryLevel(param);
  }

  private parseExtruder(param: INotifyStatusUpdateParams) {
    const extruder = param.extruder;
    if (extruder !== undefined) {
      if (extruder.temperature !== undefined) {
        // console.log('extruder.temperature: ', param.extruder?.temperature);
        this.extruder.Temperature.set(extruder.temperature);
      }
      if (extruder.target !== undefined) {
        // console.log('extruder.temperature: ', param.extruder?.target);
        this.extruder.Target.set(extruder.target);
      }
      if (extruder.pressure_advance !== undefined) {
        // console.log('extruder.pressure_advance: ', param.extruder?.pressure_advance);
        this.extruder.PressureAdvance.set(extruder.pressure_advance);
      }
      if (extruder.can_extrude !== undefined) {
        // console.log('extruder.can_extrude: ', param.extruder?.can_extrude);
        this.extruder.CanExtrude.set(extruder.can_extrude);
      }
    }
  }

  private parseToolhead(param: INotifyStatusUpdateParams) {
    const toolhead = param.toolhead;
    if (toolhead !== undefined) {
      if (toolhead.max_accel !== undefined) {
        // console.log('toolhead.position: ', toolhead?.position);
        this.toolhead.MaxAcceleration.set(toolhead.max_accel);
      }
      if (toolhead.position !== undefined) {
        // console.log('toolhead.position: ', toolhead?.position);
        this.toolhead.Position.set(toolhead.position);
      }
      if (toolhead.homed_axes !== undefined) {
        // console.log('toolhead.homed_axes: ', toolhead?.homed_axes);
        this.toolhead.HomedAxes.set(toolhead.homed_axes);
      }
      if (toolhead.axis_minimum !== undefined) {
        // console.log('toolhead.axis_minimum: ', toolhead?.axis_minimum);
        this.toolhead.AxisMinimum.set(toolhead.axis_minimum);
      }
      if (toolhead.axis_maximum !== undefined) {
        // console.log('toolhead.axis_maximum: ', toolhead?.axis_maximum);
        this.toolhead.AxisMaximum.set(toolhead.axis_maximum);
      }
      if (toolhead.max_velocity !== undefined) {
        // console.log('toolhead.max_velocity: ', toolhead?.max_velocity);
        this.toolhead.MaxVelocity.set(toolhead.max_velocity);
      }
      if (toolhead.square_corner_velocity !== undefined) {
        // console.log('toolhead.square_corner_velocity: ', toolhead?.square_corner_velocity);
        this.toolhead.SquareCornerVelocity.set(toolhead.square_corner_velocity);
      }
      if (toolhead.max_accel_to_decel !== undefined) {
        // console.log('toolhead.max_accel_to_decel: ', toolhead?.max_accel_to_decel);
        this.toolhead.MaxDeceleration.set(toolhead.max_accel_to_decel);
      }
    }
  }

  private parseGcodeMove(param: INotifyStatusUpdateParams) {
    const gcode_move = param.gcode_move;
    if (gcode_move !== undefined) {
      if (gcode_move.homing_origin !== undefined) {
        // console.log('gcode_move.homing_origin: ', gcode_move?.homing_origin);
        this.gcodeMove.HomeOrigin.set(gcode_move.homing_origin);
      }
      if (gcode_move.speed !== undefined) {
        // console.log('gcode_move.speed: ', gcode_move?.speed);
        this.gcodeMove.Speed.set(gcode_move.speed);
      }
      if (gcode_move.speed_factor !== undefined) {
        // console.log('gcode_move.speed_factor: ', gcode_move?.speed_factor);
        this.gcodeMove.SpeedFactor.set(gcode_move.speed_factor);
      }
      if (gcode_move.extrude_factor !== undefined) {
        // console.log('gcode_move.extrude_factor: ', gcode_move?.extrude_factor);
        this.gcodeMove.ExtrudeFactor.set(gcode_move.extrude_factor);
      }
    }
  }

  private parseFan(param: INotifyStatusUpdateParams) {
    const fan = param.fan;
    if (fan !== undefined) {
      if (fan.speed !== undefined) {
        // console.log('fan.speed: ', param.fan?.speed);
        this.fan.Speed.set(fan.speed);
      }
      if (fan.rpm !== undefined) {
        // console.log('fan.rpm: ', param.fan?.rpm);
        this.fan.Rpm.set(fan.rpm);
      }
    }
  }

  private parsePrintStats(param: INotifyStatusUpdateParams) {
    const print_stats = param.print_stats;
    if (print_stats !== undefined) {
      if (print_stats.filename !== undefined) {
        // console.log('print_stats.filename: ', print_stats.filename);
        this.printStats.Filename.set(print_stats.filename);
      }
      if (print_stats.print_duration !== undefined) {
        // console.log('print_stats.print_duration: ', print_stats.print_duration);
        this.printStats.PrintDuration.set(print_stats.print_duration);
      }
      if (print_stats.state !== undefined) {
        // console.log('print_stats.state: ', print_stats.state);
        this.printStats.State.set(print_stats.state);
      }
      if (print_stats.message !== undefined) {
        // console.log('print_stats.message: ', print_stats.message);
        this.printStats.Message.set(print_stats.message);
      }
      if (print_stats.filament_used !== undefined) {
        // console.log('print_stats.filament_used: ', print_stats.filament_used);
        this.printStats.FilamentUsed.set(print_stats.filament_used);
      }
      if (print_stats.info !== undefined) {
        const info = print_stats.info;
        if (info?.current_layer !== undefined) {
          // console.log('print_stats.info.current_layer: ', print_stats.info?.current_layer);
          this.printStats.Info.CurrentLayer.set(info?.current_layer);
        }
        if (info?.total_layer !== undefined) {
          // console.log('print_stats.info.total_layer: ', print_stats.info?.total_layer);
          this.printStats.Info.TotalLayer.set(info?.total_layer);
        }
      }
    }
  }

  private parseDisplayStatus(param: INotifyStatusUpdateParams) {
    const display_status = param.display_status;

    if (display_status !== undefined) {
      if (display_status.progress !== undefined) {
        // console.log('display_status.progress: ', display_status?.progress);
        this.displayStatus.Progress.set(display_status.progress);
      }

      if (display_status.message !== undefined) {
        // console.log('display_status.message: ', display_status?.message);
        this.displayStatus.Message.set(display_status.message);
      }
    }
  }

  private parseWebhooks(param: INotifyStatusUpdateParams) {
    const webhooks = param.webhooks;
    if (webhooks !== undefined) {
      if (webhooks.state !== undefined) {
        // console.log('webhooks.state: ', param.webhooks?.state);
        this.klippyState.state.set(webhooks.state);
      }
      if (webhooks.state_message !== undefined) {
        // console.log('webhooks.state_message: ', param.webhooks?.state_message);
        this.klippyState.message.set(webhooks.state_message);
      }
    }
  }

  private parseHeaterBed(param: INotifyStatusUpdateParams) {
    const heater_bed = param.heater_bed;
    if (heater_bed !== undefined) {
      if (heater_bed.temperature !== undefined) {
        // console.log('heater_bed.temperature: ', heater_bed?.temperature);
        this.heaterBed.Temperature.set(heater_bed.temperature);
      }
      if (heater_bed.target !== undefined) {
        // console.log('heater_bed.temperature: ', heater_bed?.target);
        this.heaterBed.Target.set(heater_bed.target);
      }
    }
  }

  private parseMotionReport(param: INotifyStatusUpdateParams) {
    const motion_report = param.motion_report;
    if (motion_report !== undefined) {
      if (motion_report.live_extruder_velocity !== undefined) {
        this.motionReport.LiveExtruderVelocity.set(motion_report.live_extruder_velocity);
      }
      if (motion_report.live_position !== undefined) {
        this.motionReport.LivePosition.set(motion_report.live_position);
      }
      if (motion_report.live_velocity !== undefined) {
        this.motionReport.LiveVelocity.set(motion_report.live_velocity);
      }
      if (motion_report.steppers !== undefined) {
        this.motionReport.Steppers.set(motion_report.steppers);
      }
      if (motion_report.trapq !== undefined) {
        this.motionReport.Trapq.set(motion_report.trapq);
      }
    }
  }

  private parseConfigFile(param: INotifyStatusUpdateParams) {
    const configfile = param.configfile;
    if (configfile !== undefined) {
      if (configfile.config !== undefined) {
        this.configFile.Config.set(configfile.config);
      }
      if (configfile.settings !== undefined) {
        this.configFile.Settings.set(configfile.settings);
      }
      if (configfile.save_config_pending !== undefined) {
        this.configFile.SaveConfigPending.set(configfile.save_config_pending);
      }
    }
  }

  private parseQuadGantryLevel(param: INotifyStatusUpdateParams) {
    const quadGantryLevel = param.quadGantryLevel;
    if (quadGantryLevel !== undefined) {
      if (quadGantryLevel.applied !== undefined) {
        this.quadGantryLevel.Applied.set(quadGantryLevel.applied);
      }
    }
  }

  private async parseNotification(event: CustomEvent<IJsonRpcRequest>): Promise<void> {
    const notification = event.detail;
    switch (notification.method) {
      case 'notify_status_update':
        // console.log('update', notification.params);
        if (Array.isArray(notification.params) && notification.params.length > 0) {
          this.parseNotifyStatusUpdateParams(notification.params[0]);
        }
        break;
      case 'notify_klippy_ready':
        this.klippyState.state.set('ready');
        console.log('notify_klippy: ready');
        await this.subscribeAndParseParams(this.subscription);
        break;
      case 'notify_klippy_disconnected':
        this.klippyState.state.set('disconnected');
        console.log('notify_klippy: disconnected');
        break;
      case 'notify_klippy_error':
        this.klippyState.state.set('error');
        console.log('notify_klippy: error');
        break;
      case 'notify_klippy_startup':
        this.klippyState.state.set('startup');
        console.log('notify_klippy: startup');
        break;
      case 'notify_klippy_shutdown':
        this.klippyState.state.set('shutdown');
        console.log('notify_klippy: shutdown');
        break;
      case 'notify_proc_stat_update':
        // TODO parse process stats
        break;
      case 'notify_gcode_response':
        // TODO parse gcode response
        break;
      default:
        console.log('Unknown notification: ', notification);
        break;
    }
  }
}
