import type { JsonRpcClient } from '$lib/jsonrpc/JsonRpcClient';
import type { IJsonRpcErrorResponse } from '$lib/jsonrpc/types/IJsonRpcErrorResponse';
import type { IJsonRpcRequest } from '$lib/jsonrpc/types/IJsonRpcRequest';
import type { IJsonRpcSuccessResponse } from '$lib/jsonrpc/types/IJsonRpcSuccessResponse';
import { JsonRpcRequest } from '$lib/jsonrpc/types/JsonRpcRequest';
import { DisplayStatus } from './modules/DisplayStatus';
import { Extruder } from './modules/Extruder';
import { Fan } from './modules/Fan';
import { GCodeMove } from './modules/GCodeMove';
import { HeaterBed } from './modules/HeaterBed';
import { KlipperState } from './modules/KlipperState';
import { PrintStats } from './modules/PrintStats';
import { Toolhead } from './modules/Toolhead';
import type { INotifyStatusUpdateParams } from './types/INotifyStatusUpdate';
import type { IPrinterObjects } from './types/IPrinterObjects';

export class MoonrakerClient extends EventTarget {
  _jsonRpcClient: JsonRpcClient;

  public klippyState = new KlipperState();
  public heaterBed = new HeaterBed();
  public extruder = new Extruder();
  public toolhead = new Toolhead();
  public gcodeMove = new GCodeMove();
  public fan = new Fan();
  public printStats = new PrintStats();
  public displayStatus = new DisplayStatus();

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
        const printerObjects: IPrinterObjects = {
          objects: {
            webhooks: ['state', 'state_message'],
            heater_bed: ['temperature', 'target'],
            extruder: ['temperature', 'target', 'pressure_advance'],
            toolhead: ['position', 'homed_axes', 'max_accel'],
            fan: ['speed'],
            gcode_move: ['homing_origin', 'speed', 'speed_factor', 'extrude_factor'],
            print_stats: ['filename', 'print_duration', 'filament_used', 'state', 'message', 'info'],
            display_status: ['progress']
          }
        };

        successful =
          (await this.requestIdentifyConnection()) &&
          (await this.subscribeToPrinterObjects(printerObjects)) &&
          (await this.queryPrinterObjects(printerObjects));
      }
    } catch (error) {
      console.error(error);
      successful = false;
    }

    return successful;
  }

  public async disconnect(): Promise<boolean> {
    let successful = false;

    try {
      successful = await this._jsonRpcClient.disconnect();
    } catch (error) {
      console.error(error);
      successful = false;
    }

    return successful;
  }

  private async requestIdentifyConnection(): Promise<boolean> {
    const identifyConnectionRequest = new JsonRpcRequest({
      method: 'server.connection.identify',
      params: {
        client_name: 'klipper-touch',
        version: '0.0.2',
        type: 'display',
        url: 'https://github.com/freakydude/klipper-touch'
      }
    });

    let successful = true;

    try {
      await this._jsonRpcClient.sendRequest(identifyConnectionRequest);
    } catch (error) {
      successful = false;
      console.error(error);
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
      response = (await this._jsonRpcClient.sendRequest(initialRequest)) as IJsonRpcSuccessResponse;

      this.parseNotifyStatusUpdateParams(response.result.status);
    } catch (error) {
      successful = false;
      console.error(error, response);
    }

    return successful;
  }

  private async subscribeToPrinterObjects(printerObjects: IPrinterObjects): Promise<boolean> {
    const subscribeRequest = new JsonRpcRequest({
      method: 'printer.objects.subscribe',
      params: printerObjects
    });
    let successful = true;
    try {
      const response = await this._jsonRpcClient.sendRequest(subscribeRequest);

      // TODO no good solution but works for now - error handling
      const errorResponse = response as IJsonRpcErrorResponse;
      if (errorResponse?.error) {
        successful = false;
        this.klippyState.state.set('error');
        this.klippyState.message.set(errorResponse.error.message);
        console.warn(errorResponse);
      }
    } catch (error) {
      successful = false;
      console.error(error);
    }

    return successful;
  }

  protected attachToEvents() {
    this._jsonRpcClient.addEventListener('notification', (event: Event) => {
      this.parseNotification(event as CustomEvent<IJsonRpcRequest>);
    });

    this._jsonRpcClient.isConnected.subscribe((value) => {
      // this.rpcClientIsConnectedChanged(value);
    });
  }

  private rpcClientIsConnectedChanged(value: boolean) {
    if (value === false) {
      this.klippyState.state.set('disconnected');
    }
  }

  private sleep(ms: number) {
    return new Promise((resolve) => setTimeout(resolve, ms));
  }

  private parseNotifyStatusUpdateParams(param: INotifyStatusUpdateParams) {
    this.parseWebhooks(param);
    this.parseHeaterBed(param);
    this.parseExtruder(param);
    this.parseToolhead(param);
    this.parseGcodeMove(param);
    this.parseFan(param);
    this.parsePrintStats(param);
    this.parseDisplayStatus(param);
  }

  private parseExtruder(param: INotifyStatusUpdateParams) {
    if (param.extruder?.temperature != undefined) {
      // console.log('extruder.temperature: ', param.extruder?.temperature);
      this.extruder.Temperature.set(param.extruder?.temperature);
    }
    if (param.extruder?.target != undefined) {
      // console.log('extruder.temperature: ', param.extruder?.target);
      this.extruder.Target.set(param.extruder?.target);
    }
    if (param.extruder?.pressure_advance != undefined) {
      // console.log('extruder.pressure_advance: ', param.extruder?.pressure_advance);
      this.extruder.PressureAdvance.set(param.extruder?.pressure_advance);
    }
  }

  private parseToolhead(param: INotifyStatusUpdateParams) {
    if (param.toolhead?.max_accel != undefined) {
      // console.log('toolhead.position: ', param.toolhead?.position);
      this.toolhead.MaxAcceleration.set(param.toolhead?.max_accel);
    }
    if (param.toolhead?.position != undefined) {
      // console.log('toolhead.position: ', param.toolhead?.position);
      this.toolhead.Position.set(param.toolhead?.position);
    }
    if (param.toolhead?.homed_axes != undefined) {
      // console.log('toolhead.homed_axes: ', param.toolhead?.homed_axes);
      this.toolhead.HomedAxes.set(param.toolhead?.homed_axes);
    }
  }

  private parseGcodeMove(param: INotifyStatusUpdateParams) {
    if (param.gcode_move?.homing_origin != undefined) {
      // console.log('gcode_move.homing_origin: ', param.gcode_move?.homing_origin);
      this.gcodeMove.HomeOrigin.set(param.gcode_move?.homing_origin[2]);
    }
    if (param.gcode_move?.speed != undefined) {
      // console.log('gcode_move.speed: ', param.gcode_move?.speed);
      this.gcodeMove.Speed.set(param.gcode_move?.speed);
    }
    if (param.gcode_move?.speed_factor != undefined) {
      // console.log('gcode_move.speed_factor: ', param.gcode_move?.speed_factor);
      this.gcodeMove.SpeedFactor.set(param.gcode_move?.speed_factor);
    }
    if (param.gcode_move?.extrude_factor != undefined) {
      // console.log('gcode_move.extrude_factor: ', param.gcode_move?.extrude_factor);
      this.gcodeMove.ExtrudeFactor.set(param.gcode_move?.extrude_factor);
    }
  }

  private parseFan(param: INotifyStatusUpdateParams) {
    if (param.fan?.speed != undefined) {
      console.log('fan.speed: ', param.fan?.speed);
      this.fan.Speed.set(param.fan?.speed);
    }
    if (param.fan?.rpm != undefined) {
      console.log('fan.rpm: ', param.fan?.rpm);
      this.fan.Rpm.set(param.fan?.rpm);
    }
  }

  private parsePrintStats(param: INotifyStatusUpdateParams) {
    if (param.print_stats?.filename != undefined) {
      // console.log('print_stats.filename: ', param.print_stats?.filename);
      this.printStats.Filename.set(param.print_stats?.filename.slice(0, -6)); //cut ".gcode"
    }
    if (param.print_stats?.print_duration != undefined) {
      // console.log('print_stats.print_duration: ', param.print_stats?.print_duration);
      this.printStats.PrintDuration.set(param.print_stats?.print_duration);
    }
    if (param.print_stats?.state != undefined) {
      // console.log('print_stats.state: ', param.print_stats?.state);
      this.printStats.State.set(param.print_stats?.state);
    }
    if (param.print_stats?.message != undefined) {
      // console.log('print_stats.message: ', param.print_stats?.message);
      this.printStats.Message.set(param.print_stats?.message);
    }
    if (param.print_stats?.filament_used != undefined) {
      // console.log('print_stats.filament_used: ', param.print_stats?.filament_used);
      this.printStats.FilamentUsed.set(param.print_stats?.filament_used);
    }
    if (param.print_stats?.info != undefined) {
      const info = param.print_stats?.info;
      if (info?.current_layer != undefined) {
        // console.log('print_stats.info.current_layer: ', param.print_stats?.info?.current_layer);
        this.printStats.Info.CurrentLayer.set(info?.current_layer);
      }
      if (info?.total_layer != undefined) {
        // console.log('print_stats.info.total_layer: ', param.print_stats?.info?.total_layer);
        this.printStats.Info.TotalLayer.set(info?.total_layer);
      }
    }
  }

  private parseDisplayStatus(param: INotifyStatusUpdateParams) {
    if (param.display_status?.progress != undefined) {
      // console.log('display_status.progress: ', param.display_status?.progress);
      this.displayStatus.Progress.set(param.display_status?.progress);
    }
  }

  private parseWebhooks(param: INotifyStatusUpdateParams) {
    if (param.webhooks?.state != undefined) {
      // console.log('webhooks.state: ', param.webhooks?.state);
      this.klippyState.state.set(param.webhooks?.state);
    }
    if (param.webhooks?.state_message != undefined) {
      // console.log('webhooks.state_message: ', param.webhooks?.state_message);
      this.klippyState.message.set(param.webhooks?.state_message);
    }
  }

  parseHeaterBed(param: INotifyStatusUpdateParams) {
    if (param.heater_bed?.temperature != undefined) {
      // console.log('heater_bed.temperature: ', param.heater_bed?.temperature);
      this.heaterBed.Temperature.set(param.heater_bed?.temperature);
    }
    if (param.heater_bed?.target != undefined) {
      // console.log('heater_bed.temperature: ', param.heater_bed?.target);
      this.heaterBed.Target.set(param.heater_bed?.target);
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
