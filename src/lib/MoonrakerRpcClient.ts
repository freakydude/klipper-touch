import { JsonRpcClient, JsonRpcRequest, type IJsonRpcErrorResponse, type IJsonRpcRequest, type IJsonRpcSuccessResponse } from '$lib/JsonRpcClient';
import { writable, type Readable } from 'svelte/store';
import type { INotifyStatusUpdateParams } from './moonraker-types/INotifyStatusUpdate';
import type { IPrinterObjects } from './moonraker-types/IPrinterObjects';
import type { TKlippyState } from './moonraker-types/TKlippyState';
import type { TPrintState } from './moonraker-types/TPrintState';

export class MoonrakerRpcClient extends EventTarget {
  _jsonRpcClient: JsonRpcClient;
  _isConnected = writable(false);

  public klippyState = writable<TKlippyState>('disconnected');
  public heaterBedTarget = writable(0.0);
  public heaterBedTemperature = writable(0.0);
  public extruderTemperature = writable(0.0);
  public extruderTarget = writable(0.0);
  public toolheadPosition = writable([0, 0, 0, 0]);
  public gcodeMoveHomeOrigin = writable(0.0);
  public fanSpeed = writable(0.0);
  public toolheadHomedAxes = writable('');
  public printStatsState = writable<TPrintState>('standby');
  public printStatsMessage = writable('');
  public printStatsFilename = writable('');
  public displayStatusProgress = writable(0.0);

  public constructor(jsonRpcClient: JsonRpcClient) {
    super();
    this._jsonRpcClient = jsonRpcClient;

    this.attachToEvents();
  }

  public get isConnected(): Readable<boolean> {
    return this._isConnected as Readable<boolean>;
  }

  public async requestIdentifyConnection(): Promise<IJsonRpcSuccessResponse | IJsonRpcErrorResponse> {
    const identifyConnectionRequest = new JsonRpcRequest({
      method: 'server.connection.identify',
      params: {
        client_name: 'klipper-touch',
        version: '0.0.2',
        type: 'display',
        url: 'https://github.com/freakydude/klipper-touch'
      }
    });

    const result = await this._jsonRpcClient.sendRequest(identifyConnectionRequest);
    // console.log('requestIdentifyConnection - isConnected', result);

    return result;
  }

  public async connect(): Promise<boolean> {
    let successful = false;

    try {
      successful = await this._jsonRpcClient.connect();
    } catch (error) {
      console.log(error);
    }

    if (successful) {
      const serverInfoRequest = new JsonRpcRequest({ method: 'server.info' });

      try {
        const response = (await this._jsonRpcClient.sendRequest(serverInfoRequest)) as IJsonRpcSuccessResponse;

        const state: TKlippyState = response.result.klippy_state;
        this.klippyState.set(state);
      } catch (error) {
        console.log(error);
      }

      const printerObjects: IPrinterObjects = {
        objects: {
          heater_bed: ['temperature', 'target'],
          extruder: ['temperature', 'target'],
          toolhead: ['position', 'homed_axes'],
          fan: ['speed'],
          gcode_move: ['homing_origin'],
          print_stats: ['filename', 'state', 'message'],
          display_status: ['progress']
        }
      };

      const subscribeRequest = new JsonRpcRequest({
        method: 'printer.objects.subscribe',
        params: printerObjects
      });

      try {
        await this._jsonRpcClient.sendRequest(subscribeRequest);
        // console.log(response);
      } catch (error) {
        console.log(error);
      }

      const initialRequest = new JsonRpcRequest({
        method: 'printer.objects.query',
        params: printerObjects
      });

      try {
        const response = (await this._jsonRpcClient.sendRequest(initialRequest)) as IJsonRpcSuccessResponse;
        // console.log(response);
        this.parseState(response.result.status);
      } catch (error) {
        console.log(error);
      }
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

  protected attachToEvents() {
    this._jsonRpcClient.addEventListener('notification', (event: Event) => {
      this.parseNotification(event as CustomEvent<IJsonRpcRequest>);
    });

    this._jsonRpcClient.isConnected.subscribe((value) => {
      this._isConnected.set(value);
    });
  }

  private sleep(ms: number) {
    return new Promise((resolve) => setTimeout(resolve, ms));
  }

  private parseState(objectsParamsRoot: INotifyStatusUpdateParams) {
    const firstObject = objectsParamsRoot;

    if (firstObject.heater_bed?.temperature != undefined) {
      // console.log('heater_bed.temperature: ', firstObject.heater_bed?.temperature);
      this.heaterBedTemperature.set(firstObject.heater_bed?.temperature);
    }
    if (firstObject.heater_bed?.target != undefined) {
      // console.log('heater_bed.temperature: ', firstObject.heater_bed?.target);
      this.heaterBedTarget.set(firstObject.heater_bed?.target);
    }
    if (firstObject.extruder?.temperature != undefined) {
      // console.log('extruder.temperature: ', firstObject.extruder?.temperature);
      this.extruderTemperature.set(firstObject.extruder?.temperature);
    }
    if (firstObject.extruder?.target != undefined) {
      // console.log('extruder.temperature: ', firstObject.extruder?.target);
      this.extruderTarget.set(firstObject.extruder?.target);
    }
    if (firstObject.toolhead?.position != undefined) {
      // console.log('toolhead.position: ', firstObject.toolhead?.position);
      this.toolheadPosition.set(firstObject.toolhead?.position);
    }
    if (firstObject.toolhead?.homed_axes != undefined) {
      // console.log('toolhead.homed_axes: ', firstObject.toolhead?.homed_axes);
      this.toolheadHomedAxes.set(firstObject.toolhead?.homed_axes);
    }
    if (firstObject.gcode_move?.homing_origin != undefined) {
      // console.log('gcode_move.homing_origin: ', firstObject.gcode_move?.homing_origin);
      this.gcodeMoveHomeOrigin.set(firstObject.gcode_move?.homing_origin[2]);
    }
    if (firstObject.fan?.speed != undefined) {
      // console.log('fan.speed: ', firstObject.fan?.speed);
      this.fanSpeed.set(firstObject.fan?.speed);
    }
    if (firstObject.print_stats?.filename != undefined) {
      // console.log('print_stats.filename: ', firstObject.print_stats?.filename);
      this.printStatsFilename.set(firstObject.print_stats?.filename.slice(0, -6)); //cut ".gcode"
    }
    if (firstObject.print_stats?.state != undefined) {
      // console.log('print_stats.state: ', firstObject.print_stats?.state);
      this.printStatsState.set(firstObject.print_stats?.state);
    }
    if (firstObject.print_stats?.message != undefined) {
      // console.log('print_stats.message: ', firstObject.print_stats?.message);
      this.printStatsMessage.set(firstObject.print_stats?.message);
    }
    if (firstObject.display_status?.progress != undefined) {
      // console.log('display_status.progress: ', firstObject.display_status?.progress);
      this.displayStatusProgress.set(firstObject.display_status?.progress);
    }
  }

  private async parseNotification(event: CustomEvent<IJsonRpcRequest>): Promise<void> {
    const notification = event.detail;
    switch (notification.method) {
      case 'notify_status_update':
        // console.log('update', notification.params);
        if (Array.isArray(notification.params) && notification.params.length > 0) {
          this.parseState(notification.params[0]);
        }
        break;
      case 'notify_klippy_ready':
        this.klippyState.set('ready');
        break;
      case 'notify_klippy_disconnected':
        this.klippyState.set('disconnected');
        break;
      case 'notify_klippy_error':
        this.klippyState.set('error');
        break;
      case 'notify_klippy_startup':
        this.klippyState.set('startup');
        break;
      case 'notify_klippy_shutdown':
        this.klippyState.set('shutdown');
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
