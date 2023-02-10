import { JsonRpcClient, JsonRpcRequest, type IJsonRpcErrorResponse, type IJsonRpcRequest, type IJsonRpcSuccessResponse } from '$lib/JsonRpcClient';
import { writable, type Readable } from 'svelte/store';

export type KlippyState = 'ready' | 'error' | 'shutdown' | 'startup' | 'disconnected';

export class MoonrakerRpcClient extends EventTarget {
  _jsonRpcClient: JsonRpcClient;
  _isConnected = writable(false);
  _klippyState = writable<KlippyState>('disconnected');

  public constructor(jsonRpcClient: JsonRpcClient) {
    super();
    this._jsonRpcClient = jsonRpcClient;

    this.attachToEvents();
  }

  public get isConnected(): Readable<boolean> {
    return this._isConnected as Readable<boolean>;
  }

  public get klippyState(): Readable<KlippyState> {
    return this._klippyState as Readable<KlippyState>;
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
      let serverInfoRequest = new JsonRpcRequest({ method: 'server.info' });

      try {
        let response = (await this._jsonRpcClient.sendRequest(serverInfoRequest)) as IJsonRpcSuccessResponse;

        let state: KlippyState = response.result.klippy_state;
        this._klippyState.set(state);
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

  public heaterBedTargetTemperature = writable(0.0);
  public heaterBedCurrentTemperature = writable(0.0);
  public extruderCurrentTemperature = writable(0.0);
  public extruderTargetTemperature = writable(0.0);
  public toolheadPosition = writable([0, 0, 0, 0]);
  public gcodeZOffset = writable(0.0);
  public homedAxes = writable('');

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
        }
        break;
      case 'notify_klippy_ready':
        this._klippyState.set('ready');
        break;
      case 'notify_klippy_disconnected':
        this._klippyState.set('disconnected');
        break;
      case 'notify_klippy_error':
        this._klippyState.set('error');
        break;
      case 'notify_klippy_startup':
        this._klippyState.set('startup');
        break;
      case 'notify_klippy_shutdown':
        this._klippyState.set('shutdown');
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
