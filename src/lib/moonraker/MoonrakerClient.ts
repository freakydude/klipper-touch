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
      configfile: ['save_config_pending']
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

  public constructor(jsonRpcClient: JsonRpcClient) {
    super();
    this._jsonRpcClient = jsonRpcClient;

    this.attachToEvents();
  }

  public async connect(): Promise<boolean> {
    let successful = false;

    try {
      successful = await this._jsonRpcClient.connect();

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

  private async queryPrinterObjects(printerObjects: IPrinterObjects): Promise<boolean> {
    const initialRequest = new JsonRpcRequest({
      method: 'printer.objects.query',
      params: printerObjects
    });

    let successful = true;

    let response;
    try {
      response = await this._jsonRpcClient.sendRequest(initialRequest);
      console.log('MoonrakerClient.queryPrinterObjects');
      if (response.result) {
        this.parseNotifyStatusUpdateParams(response.result.status);
      }
    } catch (error) {
      successful = false;
      console.error('MoonrakerClient.queryPrinterObjects() ', error, response);
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
      if (response.result) {
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
  }

  private parseExtruder(param: INotifyStatusUpdateParams) {
    if (param.extruder?.temperature !== undefined) {
      // console.log('extruder.temperature: ', param.extruder?.temperature);
      this.extruder.Temperature.set(param.extruder?.temperature);
    }
    if (param.extruder?.target !== undefined) {
      // console.log('extruder.temperature: ', param.extruder?.target);
      this.extruder.Target.set(param.extruder?.target);
    }
    if (param.extruder?.pressure_advance !== undefined) {
      // console.log('extruder.pressure_advance: ', param.extruder?.pressure_advance);
      this.extruder.PressureAdvance.set(param.extruder?.pressure_advance);
    }
    if (param.extruder?.can_extrude !== undefined) {
      // console.log('extruder.can_extrude: ', param.extruder?.can_extrude);
      this.extruder.CanExtrude.set(param.extruder?.can_extrude);
    }
  }

  private parseToolhead(param: INotifyStatusUpdateParams) {
    if (param.toolhead?.max_accel !== undefined) {
      // console.log('toolhead.position: ', param.toolhead?.position);
      this.toolhead.MaxAcceleration.set(param.toolhead?.max_accel);
    }
    if (param.toolhead?.position !== undefined) {
      // console.log('toolhead.position: ', param.toolhead?.position);
      this.toolhead.Position.set(param.toolhead?.position);
    }
    if (param.toolhead?.homed_axes !== undefined) {
      // console.log('toolhead.homed_axes: ', param.toolhead?.homed_axes);
      this.toolhead.HomedAxes.set(param.toolhead?.homed_axes);
    }
    if (param.toolhead?.axis_minimum !== undefined) {
      // console.log('toolhead.axis_minimum: ', param.toolhead?.axis_minimum);
      this.toolhead.AxisMinimum.set(param.toolhead?.axis_minimum);
    }
    if (param.toolhead?.axis_maximum !== undefined) {
      // console.log('toolhead.axis_maximum: ', param.toolhead?.axis_maximum);
      this.toolhead.AxisMaximum.set(param.toolhead?.axis_maximum);
    }
    if (param.toolhead?.max_velocity !== undefined) {
      // console.log('toolhead.max_velocity: ', param.toolhead?.max_velocity);
      this.toolhead.MaxVelocity.set(param.toolhead?.max_velocity);
    }
    if (param.toolhead?.square_corner_velocity !== undefined) {
      // console.log('toolhead.square_corner_velocity: ', param.toolhead?.square_corner_velocity);
      this.toolhead.SquareCornerVelocity.set(param.toolhead?.square_corner_velocity);
    }
    if (param.toolhead?.max_accel_to_decel !== undefined) {
      // console.log('toolhead.max_accel_to_decel: ', param.toolhead?.max_accel_to_decel);
      this.toolhead.MaxDeceleration.set(param.toolhead?.max_accel_to_decel);
    }
  }

  private parseGcodeMove(param: INotifyStatusUpdateParams) {
    if (param.gcode_move?.homing_origin !== undefined) {
      // console.log('gcode_move.homing_origin: ', param.gcode_move?.homing_origin);
      this.gcodeMove.HomeOrigin.set(param.gcode_move?.homing_origin);
    }
    if (param.gcode_move?.speed !== undefined) {
      // console.log('gcode_move.speed: ', param.gcode_move?.speed);
      this.gcodeMove.Speed.set(param.gcode_move?.speed);
    }
    if (param.gcode_move?.speed_factor !== undefined) {
      // console.log('gcode_move.speed_factor: ', param.gcode_move?.speed_factor);
      this.gcodeMove.SpeedFactor.set(param.gcode_move?.speed_factor);
    }
    if (param.gcode_move?.extrude_factor !== undefined) {
      // console.log('gcode_move.extrude_factor: ', param.gcode_move?.extrude_factor);
      this.gcodeMove.ExtrudeFactor.set(param.gcode_move?.extrude_factor);
    }
  }

  private parseFan(param: INotifyStatusUpdateParams) {
    if (param.fan?.speed !== undefined) {
      // console.log('fan.speed: ', param.fan?.speed);
      this.fan.Speed.set(param.fan?.speed);
    }
    if (param.fan?.rpm !== undefined) {
      // console.log('fan.rpm: ', param.fan?.rpm);
      this.fan.Rpm.set(param.fan?.rpm);
    }
  }

  private parsePrintStats(param: INotifyStatusUpdateParams) {
    if (param.print_stats?.filename !== undefined) {
      // console.log('print_stats.filename: ', param.print_stats?.filename);
      this.printStats.Filename.set(param.print_stats?.filename.slice(0, -6)); //cut ".gcode"
    }
    if (param.print_stats?.print_duration !== undefined) {
      // console.log('print_stats.print_duration: ', param.print_stats?.print_duration);
      this.printStats.PrintDuration.set(param.print_stats?.print_duration);
    }
    if (param.print_stats?.state !== undefined) {
      // console.log('print_stats.state: ', param.print_stats?.state);
      this.printStats.State.set(param.print_stats?.state);
    }
    if (param.print_stats?.message !== undefined) {
      // console.log('print_stats.message: ', param.print_stats?.message);
      this.printStats.Message.set(param.print_stats?.message);
    }
    if (param.print_stats?.filament_used !== undefined) {
      // console.log('print_stats.filament_used: ', param.print_stats?.filament_used);
      this.printStats.FilamentUsed.set(param.print_stats?.filament_used);
    }
    if (param.print_stats?.info !== undefined) {
      const info = param.print_stats?.info;
      if (info?.current_layer !== undefined) {
        // console.log('print_stats.info.current_layer: ', param.print_stats?.info?.current_layer);
        this.printStats.Info.CurrentLayer.set(info?.current_layer);
      }
      if (info?.total_layer !== undefined) {
        // console.log('print_stats.info.total_layer: ', param.print_stats?.info?.total_layer);
        this.printStats.Info.TotalLayer.set(info?.total_layer);
      }
    }
  }

  private parseDisplayStatus(param: INotifyStatusUpdateParams) {
    if (param.display_status?.progress !== undefined) {
      // console.log('display_status.progress: ', param.display_status?.progress);
      this.displayStatus.Progress.set(param.display_status?.progress);
    }
    if (param.display_status?.message !== undefined) {
      // console.log('display_status.message: ', param.display_status?.message);
      this.displayStatus.Message.set(param.display_status?.message);
    }
  }

  private parseWebhooks(param: INotifyStatusUpdateParams) {
    if (param.webhooks?.state !== undefined) {
      // console.log('webhooks.state: ', param.webhooks?.state);
      this.klippyState.state.set(param.webhooks?.state);
    }
    if (param.webhooks?.state_message !== undefined) {
      // console.log('webhooks.state_message: ', param.webhooks?.state_message);
      this.klippyState.message.set(param.webhooks?.state_message);
    }
  }

  private parseHeaterBed(param: INotifyStatusUpdateParams) {
    if (param.heater_bed?.temperature !== undefined) {
      // console.log('heater_bed.temperature: ', param.heater_bed?.temperature);
      this.heaterBed.Temperature.set(param.heater_bed?.temperature);
    }
    if (param.heater_bed?.target !== undefined) {
      // console.log('heater_bed.temperature: ', param.heater_bed?.target);
      this.heaterBed.Target.set(param.heater_bed?.target);
    }
  }

  private parseMotionReport(param: INotifyStatusUpdateParams) {
    if (param.motion_report?.live_extruder_velocity !== undefined) {
      this.motionReport.LiveExtruderVelocity.set(param.motion_report?.live_extruder_velocity);
    }
    if (param.motion_report?.live_position !== undefined) {
      this.motionReport.LivePosition.set(param.motion_report?.live_position);
    }
    if (param.motion_report?.live_velocity !== undefined) {
      this.motionReport.LiveVelocity.set(param.motion_report?.live_velocity);
    }
    if (param.motion_report?.steppers !== undefined) {
      this.motionReport.Steppers.set(param.motion_report?.steppers);
    }
    if (param.motion_report?.trapq !== undefined) {
      this.motionReport.Trapq.set(param.motion_report?.trapq);
    }
  }

  private parseConfigFile(param: INotifyStatusUpdateParams) {
    if (param.configfile?.config !== undefined) {
      this.configFile.Config.set(param.configfile?.config);
    }
    if (param.configfile?.settings !== undefined) {
      this.configFile.Settings.set(param.configfile?.settings);
    }
    if (param.configfile?.save_config_pending !== undefined) {
      this.configFile.SaveConfigPending.set(param.configfile?.save_config_pending);
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
