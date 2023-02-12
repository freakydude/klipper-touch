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
  // public klippyStateMessage = writable<string>('');
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
  public printStateFilename = writable('');
  public printStateProgress = writable(0.0);

  private async parseNotification(event: CustomEvent<IJsonRpcRequest>): Promise<void> {
    const notification = event.detail;
    switch (notification.method) {
      case 'notify_status_update':
        // console.log('update', notification.params);
        if (Array.isArray(notification.params) && notification.params.length > 0) {
          if (notification.params[0].heater_bed?.temperature != undefined) {
            // console.log('heater_bed.temperature: ', notification.params[0].heater_bed?.temperature);
            this.heaterBedCurrentTemperature.set(notification.params[0].heater_bed?.temperature);
          }
          if (notification.params[0].heater_bed?.target != undefined) {
            // console.log('heater_bed.temperature: ', notification.params[0].heater_bed?.target);
            this.heaterBedTargetTemperature.set(notification.params[0].heater_bed?.target);
          }
          if (notification.params[0].extruder?.temperature != undefined) {
            // console.log('extruder.temperature: ', notification.params[0].extruder?.temperature);
            this.extruderCurrentTemperature.set(notification.params[0].extruder?.temperature);
          }
          if (notification.params[0].extruder?.target != undefined) {
            // console.log('extruder.temperature: ', notification.params[0].extruder?.target);
            this.extruderTargetTemperature.set(notification.params[0].extruder?.target);
          }
          if (notification.params[0].toolhead?.position != undefined) {
            // console.log('toolhead.position: ', notification.params[0].toolhead?.position);
            this.toolheadPosition.set(notification.params[0].toolhead?.position);
          }
          if (notification.params[0].toolhead?.homed_axes != undefined) {
            // console.log('toolhead.homed_axes: ', notification.params[0].toolhead?.homed_axes);
            this.homedAxes.set(notification.params[0].toolhead?.homed_axes);
          }
          if (notification.params[0].gcode_move?.homing_origin != undefined) {
            // console.log('gcode_move.homing_origin: ', notification.params[0].gcode_move?.homing_origin);
            this.gcodeZOffset.set(notification.params[0].gcode_move?.homing_origin[2]);
          }
          if (notification.params[0].fan?.speed != undefined) {
            // console.log('fan.speed: ', notification.params[0].fan?.speed);
            this.fanSpeed.set(notification.params[0].fan?.speed);
          }
          if (notification.params[0].print_stats?.filename != undefined) {
            // console.log('print_stats.filename: ', notification.params[0].print_stats?.filename);
            this.printStateFilename.set(notification.params[0].print_stats?.filename);
          }
          if (notification.params[0].print_stats?.state != undefined) {
            // console.log('print_stats.state: ', notification.params[0].print_stats?.state);
            this.printState.set(notification.params[0].print_stats?.state);
          }
          if (notification.params[0].print_stats?.message != undefined) {
            // console.log('print_stats.message: ', notification.params[0].print_stats?.message);
            this.printStateErrorMessage.set(notification.params[0].print_stats?.message);
          }
          if (notification.params[0].virtual_sdcard?.progress != undefined) {
            // console.log('virtual_sdcard.progress: ', notification.params[0].virtual_sdcard?.progress);
            this.printStateProgress.set(notification.params[0].virtual_sdcard?.progress);
          }
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
