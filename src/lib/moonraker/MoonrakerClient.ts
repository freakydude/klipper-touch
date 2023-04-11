import type { JsonRpcClient } from '$lib/jsonrpc/JsonRpcClient';
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
            extruder: ['temperature', 'target'],
            toolhead: ['position', 'homed_axes'],
            fan: ['speed'],
            gcode_move: ['homing_origin'],
            print_stats: ['filename', 'print_duration', 'state', 'message'],
            display_status: ['progress']
          }
        };

        await this.requestIdentifyConnection();
        await this.subscribeToPrinterObjects(printerObjects);
        await this.queryPrinterObjects(printerObjects);
      }
    } catch (error) {
      console.log(error);
    }

    return successful;
  }

  public async disconnect(): Promise<boolean> {
    let successful = false;

    try {
      successful = await this._jsonRpcClient.disconnect();
    } catch (error) {
      console.log(error);
    }

    return successful;
  }

  private async requestIdentifyConnection() {
    const identifyConnectionRequest = new JsonRpcRequest({
      method: 'server.connection.identify',
      params: {
        client_name: 'klipper-touch',
        version: '0.0.2',
        type: 'display',
        url: 'https://github.com/freakydude/klipper-touch'
      }
    });

    try {
      await this._jsonRpcClient.sendRequest(identifyConnectionRequest);
    } catch (error) {
      console.log(error);
    }
  }

  private async queryPrinterObjects(printerObjects: IPrinterObjects) {
    const initialRequest = new JsonRpcRequest({
      method: 'printer.objects.query',
      params: printerObjects
    });

    try {
      const response = (await this._jsonRpcClient.sendRequest(initialRequest)) as IJsonRpcSuccessResponse;
      this.parseNotifyStatusUpdateParams(response.result.status);
    } catch (error) {
      console.log(error);
    }
  }

  private async subscribeToPrinterObjects(printerObjects: IPrinterObjects) {
    const subscribeRequest = new JsonRpcRequest({
      method: 'printer.objects.subscribe',
      params: printerObjects
    });

    try {
      await this._jsonRpcClient.sendRequest(subscribeRequest);
    } catch (error) {
      console.log(error);
    }
  }

  protected attachToEvents() {
    this._jsonRpcClient.addEventListener('notification', (event: Event) => {
      this.parseNotification(event as CustomEvent<IJsonRpcRequest>);
    });

    this._jsonRpcClient.isConnected.subscribe((value) => {
      this.rpcClientIsConnectedChanged(value);
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
    if (param.webhooks?.state != undefined) {
      // console.log('webhooks.state: ', firstObject.webhooks?.state);
      this.klippyState.state.set(param.webhooks?.state);
    }
    if (param.webhooks?.state_message != undefined) {
      // console.log('webhooks.state_message: ', firstObject.webhooks?.state_message);
      this.klippyState.message.set(param.webhooks?.state_message);
    }
    if (param.heater_bed?.temperature != undefined) {
      // console.log('heater_bed.temperature: ', firstObject.heater_bed?.temperature);
      this.heaterBed.Temperature.set(param.heater_bed?.temperature);
    }
    if (param.heater_bed?.target != undefined) {
      // console.log('heater_bed.temperature: ', firstObject.heater_bed?.target);
      this.heaterBed.Target.set(param.heater_bed?.target);
    }
    if (param.extruder?.temperature != undefined) {
      // console.log('extruder.temperature: ', firstObject.extruder?.temperature);
      this.extruder.Temperature.set(param.extruder?.temperature);
    }
    if (param.extruder?.target != undefined) {
      // console.log('extruder.temperature: ', firstObject.extruder?.target);
      this.extruder.Target.set(param.extruder?.target);
    }
    if (param.toolhead?.position != undefined) {
      // console.log('toolhead.position: ', firstObject.toolhead?.position);
      this.toolhead.Position.set(param.toolhead?.position);
    }
    if (param.toolhead?.homed_axes != undefined) {
      // console.log('toolhead.homed_axes: ', firstObject.toolhead?.homed_axes);
      this.toolhead.HomedAxes.set(param.toolhead?.homed_axes);
    }
    if (param.gcode_move?.homing_origin != undefined) {
      // console.log('gcode_move.homing_origin: ', firstObject.gcode_move?.homing_origin);
      this.gcodeMove.HomeOrigin.set(param.gcode_move?.homing_origin[2]);
    }
    if (param.fan?.speed != undefined) {
      // console.log('fan.speed: ', firstObject.fan?.speed);
      this.fan.Speed.set(param.fan?.speed);
    }
    if (param.print_stats?.filename != undefined) {
      // console.log('print_stats.filename: ', firstObject.print_stats?.filename);
      this.printStats.Filename.set(param.print_stats?.filename.slice(0, -6)); //cut ".gcode"
    }
    if (param.print_stats?.print_duration != undefined) {
      // console.log('print_stats.print_duration: ', firstObject.print_stats?.print_duration);
      this.printStats.PrintDuration.set(param.print_stats?.print_duration);
    }
    if (param.print_stats?.state != undefined) {
      // console.log('print_stats.state: ', firstObject.print_stats?.state);
      this.printStats.State.set(param.print_stats?.state);
    }
    if (param.print_stats?.message != undefined) {
      // console.log('print_stats.message: ', firstObject.print_stats?.message);
      this.printStats.Message.set(param.print_stats?.message);
    }
    if (param.display_status?.progress != undefined) {
      // console.log('display_status.progress: ', firstObject.display_status?.progress);
      this.displayStatus.Progress.set(param.display_status?.progress);
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
        break;
      case 'notify_klippy_disconnected':
        this.klippyState.state.set('disconnected');
        break;
      case 'notify_klippy_error':
        this.klippyState.state.set('error');
        break;
      case 'notify_klippy_startup':
        this.klippyState.state.set('startup');
        break;
      case 'notify_klippy_shutdown':
        this.klippyState.state.set('shutdown');
        break;
      case 'notify_proc_stat_update':
        // TODO parse process stats
        break;
      default:
        console.log('Unknown notification: ', notification);
        break;
    }
  }
}
