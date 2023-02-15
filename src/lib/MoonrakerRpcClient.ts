import { JsonRpcClient, JsonRpcRequest, type IJsonRpcErrorResponse, type IJsonRpcRequest, type IJsonRpcSuccessResponse } from '$lib/JsonRpcClient';
import { writable, type Readable } from 'svelte/store';

export type KlippyState = 'ready' | 'error' | 'shutdown' | 'startup' | 'disconnected';
export type PrintState = 'standby' | 'printing' | 'paused' | 'complete' | 'cancelled' | 'error';

export class MoonrakerRpcClient extends EventTarget {
  _jsonRpcClient: JsonRpcClient;
  _isConnected = writable(false);

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
        version: '0.0.1',
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

        const state: KlippyState = response.result.klippy_state;
        this.klippyState.set(state);
      } catch (error) {
        console.log(error);
      }

      const subscribeRequest = new JsonRpcRequest({
        method: 'printer.objects.subscribe',
        params: {
          objects: {
            heater_bed: ['temperature', 'target'],
            extruder: ['temperature', 'target'],
            toolhead: ['position', 'homed_axes'],
            fan: ['speed'],
            gcode_move: ['homing_origin'],
            print_stats: ['filename', 'state', 'message'],
            display_status: ['progress']
          }
        }
      });

      try {
        const response = (await this._jsonRpcClient.sendRequest(subscribeRequest)) as IJsonRpcSuccessResponse;
        // console.log(response);
      } catch (error) {
        console.log(error);
      }

      const initialRequest = new JsonRpcRequest({
        method: 'printer.objects.query',
        params: {
          objects: {
            heater_bed: ['temperature', 'target'],
            extruder: ['temperature', 'target'],
            toolhead: ['position', 'homed_axes'],
            fan: ['speed'],
            gcode_move: ['homing_origin'],
            print_stats: ['filename', 'state', 'message'],
            display_status: ['progress']
          }
        }
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

  public klippyState = writable<KlippyState>('disconnected');
  public heaterBedTargetTemperature = writable(0.0);
  public heaterBedCurrentTemperature = writable(0.0);
  public extruderCurrentTemperature = writable(0.0);
  public extruderTargetTemperature = writable(0.0);
  public toolheadPosition = writable([0, 0, 0, 0]);
  public gcodeZOffset = writable(0.0);
  public fanSpeed = writable(0.0);
  public homedAxes = writable('');
  public printState = writable<PrintState>('standby');
  public printStateErrorMessage = writable('');
  public currentPrintedFile = writable('');
  public printStateProgress = writable(0.0);

  private parseState(params: any) {
    if (params.heater_bed?.temperature != undefined) {
      // console.log('heater_bed.temperature: ', params.heater_bed?.temperature);
      this.heaterBedCurrentTemperature.set(params.heater_bed?.temperature);
    }
    if (params.heater_bed?.target != undefined) {
      // console.log('heater_bed.temperature: ', params.heater_bed?.target);
      this.heaterBedTargetTemperature.set(params.heater_bed?.target);
    }
    if (params.extruder?.temperature != undefined) {
      // console.log('extruder.temperature: ', params.extruder?.temperature);
      this.extruderCurrentTemperature.set(params.extruder?.temperature);
    }
    if (params.extruder?.target != undefined) {
      // console.log('extruder.temperature: ', params.extruder?.target);
      this.extruderTargetTemperature.set(params.extruder?.target);
    }
    if (params.toolhead?.position != undefined) {
      // console.log('toolhead.position: ', params.toolhead?.position);
      this.toolheadPosition.set(params.toolhead?.position);
    }
    if (params.toolhead?.homed_axes != undefined) {
      // console.log('toolhead.homed_axes: ', params.toolhead?.homed_axes);
      this.homedAxes.set(params.toolhead?.homed_axes);
    }
    if (params.gcode_move?.homing_origin != undefined) {
      // console.log('gcode_move.homing_origin: ', params.gcode_move?.homing_origin);
      this.gcodeZOffset.set(params.gcode_move?.homing_origin[2]);
    }
    if (params.fan?.speed != undefined) {
      // console.log('fan.speed: ', params.fan?.speed);
      this.fanSpeed.set(params.fan?.speed);
    }
    if (params.print_stats?.filename != undefined) {
      // console.log('print_stats.filename: ', params.print_stats?.filename);
      this.currentPrintedFile.set(params.print_stats?.filename.slice(0, -6)); //cut ".gcode"
    }
    if (params.print_stats?.state != undefined) {
      // console.log('print_stats.state: ', params.print_stats?.state);
      this.printState.set(params.print_stats?.state);
    }
    if (params.print_stats?.message != undefined) {
      // console.log('print_stats.message: ', params.print_stats?.message);
      this.printStateErrorMessage.set(params.print_stats?.message);
    }
    if (params.display_status?.progress != undefined) {
      // console.log('display_status.progress: ', params.display_status?.progress);
      this.printStateProgress.set(params.display_status?.progress);
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
