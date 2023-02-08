import { JsonRpcClient, JsonRpcRequest, type IJsonRpcErrorResponse, type IJsonRpcRequest, type IJsonRpcSuccessResponse } from '$lib/JsonRpcClient';
import { writable, type Readable } from 'svelte/store';

export class MoonrakerRpcClient extends EventTarget {
  _jsonRpcClient: JsonRpcClient;
  _isReady = writable(false);
  _isConnecting = false;

  public constructor(jsonRpcClient: JsonRpcClient) {
    super();
    this._jsonRpcClient = jsonRpcClient;

    this.attachToEvents();
  }

  public get isReady(): Readable<boolean> {
    return this._isReady as Readable<boolean>;
  }

  public async requestIdentifyConnection(): Promise<IJsonRpcSuccessResponse | IJsonRpcErrorResponse> {
    const identifyConnectionRequest: JsonRpcRequest = {
      jsonrpc: '2.0',
      method: 'server.connection.identify',
      params: {
        client_name: 'klipper-touch',
        version: '0.0.1',
        type: 'display',
        url: 'https://github.com/freakydude/klipper-touch'
      },
      id: JsonRpcClient.generateConnectionId()
    };

    const result = await this._jsonRpcClient.sendRequest(identifyConnectionRequest);
    // console.log('requestIdentifyConnection - isConnected', result);

    return result;
  }

  public async connect() {
    this._isConnecting = true;
    while (this._isConnecting) {
      this._isReady.set(false);
      try {
        if (await this._jsonRpcClient.connect()) {
          let klippyState = '';
          while (true) {
            const serverInfoRequest = new JsonRpcRequest({ method: 'server.info', id: JsonRpcClient.generateConnectionId() });

            try {
              const serverInfoResponse = (await this._jsonRpcClient.sendRequest(serverInfoRequest)) as IJsonRpcSuccessResponse;
              klippyState = serverInfoResponse.result.klippy_state;
              console.log('klippy_state: ', klippyState);

              if (klippyState == 'ready') {
                this._isConnecting = false;
                break;
              } else {
                await this.sleep(1000 * 5);
              }
            } catch (error) {
              console.log('MoonrakerRpcClient connect error: ', error);
            }
          }
          if (klippyState == 'ready') {
            this._isReady.set(true);

            break;
          }
        }
      } catch (error) {
        console.log('MoonrakerRpcClient connect error: ', error);
      }
      await this.sleep(1000 * 5);
    }

    this._isReady.set(true);
  }

  public async disconnect() {
    this._isConnecting = false;
    try {
      await this._jsonRpcClient.disconnect();
    } catch (error) {
      console.log('MoonrakerRpcClient disconnect error: ', error);
    }
    this._isReady.set(false);
  }

  protected attachToEvents() {
    this._jsonRpcClient.addEventListener('notification', (event: Event) => {
      this.parseNotification(event as CustomEvent<IJsonRpcRequest>);
    });

    this._jsonRpcClient.addEventListener('isConnected', (event: Event) => {
      if ((event as CustomEvent<boolean>).detail == false) {
        this.reconnect();
        this._isReady.set(false);
      }
    });
  }

  protected async reconnect(): Promise<void> {
    if (this._isConnecting == false) {
      await this.connect();
    }
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

    // console.log('parseNotification: ', notification.method);

    switch (notification.method) {
      case 'notify_klippy_ready':
        this._isReady.set(true);
        break;
      case 'notify_status_update':
        // console.log('update', notification.params);
        if (Array.isArray(notification.params) && notification.params.length > 0) {
          if (notification.params[0].heater_bed?.temperature != undefined) {
            // console.log('heater_bed.temperature', notification.params[0].heater_bed?.temperature);
            this.heaterBedCurrentTemperature.set(notification.params[0].heater_bed?.temperature);
          }
          if (notification.params[0].heater_bed?.target != undefined) {
            // console.log('heater_bed.temperature', notification.params[0].heater_bed?.target);
            this.heaterBedTargetTemperature.set(notification.params[0].heater_bed?.target);
          }
          if (notification.params[0].extruder?.temperature != undefined) {
            // console.log('extruder.temperature', notification.params[0].extruder?.temperature);
            this.extruderCurrentTemperature.set(notification.params[0].extruder?.temperature);
          }
          if (notification.params[0].extruder?.target != undefined) {
            // console.log('extruder.temperature', notification.params[0].extruder?.target);
            this.extruderTargetTemperature.set(notification.params[0].extruder?.target);
          }
          if (notification.params[0].toolhead?.position != undefined) {
            // console.log('toolhead.position', notification.params[0].toolhead?.position);
            this.toolheadPosition.set(notification.params[0].toolhead?.position);
          }
          if (notification.params[0].toolhead?.homed_axes != undefined) {
            // console.log('toolhead.homed_axes', notification.params[0].toolhead?.homed_axes);
            this.homedAxes.set(notification.params[0].toolhead?.homed_axes);
          }
          if (notification.params[0].gcode_move?.homing_origin != undefined) {
            // console.log('gcode_move.homing_origin', notification.params[0].gcode_move?.homing_origin);
            this.gcodeZOffset.set(notification.params[0].gcode_move?.homing_origin[2]);
          }
        }
        break;
      case 'notify_klippy_disconnected ':
        console.log('notify_klippy_disconnected: ');

        try {
          await this.disconnect();
        } catch (error) {
          console.log(error);
        }
        try {
          await this.connect();
        } catch (error) {
          console.log(error);
        }

        break;
    }
  }
}
