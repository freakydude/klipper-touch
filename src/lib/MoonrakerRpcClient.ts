import { JsonRpcClient,JsonRpcRequest,type IJsonRpcRequest,type IJsonRpcSuccessResponse } from '$lib/JsonRpcClient';
import { writable } from 'svelte/store';
import { HeaterBed } from './HeaterBed';
import { KlipperState } from './KlipperState';
import type { INotifyStatusUpdateParams } from './moonraker-types/INotifyStatusUpdate';
import type { IPrinterObjects } from './moonraker-types/IPrinterObjects';
import type { TPrintState } from './moonraker-types/TPrintState';

export class Extruder {
  public Temperature = writable(0.0);
  public Target = writable(0.0);
}

export class MoonrakerRpcClient extends EventTarget {
  _jsonRpcClient: JsonRpcClient;

  public klippyState = new KlipperState();
  public heaterBed = new HeaterBed();
  public extruder = new Extruder()
  public toolheadPosition = writable([0, 0, 0, 0]);
  public gcodeMoveHomeOrigin = writable(0.0);
  public fanSpeed = writable(0.0);
  public toolheadHomedAxes = writable('');
  public printStatsState = writable<TPrintState>('standby');
  public printStatsMessage = writable('');
  public printStatsFilename = writable('');
  public printStatsPrintDuration = writable(0.0);
  public displayStatusProgress = writable(0.0);

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
      this.klippyState.klippyState.set('disconnected');
    }
  }

  private sleep(ms: number) {
    return new Promise((resolve) => setTimeout(resolve, ms));
  }

  private parseNotifyStatusUpdateParams(param: INotifyStatusUpdateParams) {
    if (param.webhooks?.state != undefined) {
      // console.log('webhooks.state: ', firstObject.webhooks?.state);
      this.klippyState.klippyState.set(param.webhooks?.state);
    }
    if (param.webhooks?.state_message != undefined) {
      // console.log('webhooks.state_message: ', firstObject.webhooks?.state_message);
      this.klippyState.klippyStateMessage.set(param.webhooks?.state_message);
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
      this.toolheadPosition.set(param.toolhead?.position);
    }
    if (param.toolhead?.homed_axes != undefined) {
      // console.log('toolhead.homed_axes: ', firstObject.toolhead?.homed_axes);
      this.toolheadHomedAxes.set(param.toolhead?.homed_axes);
    }
    if (param.gcode_move?.homing_origin != undefined) {
      // console.log('gcode_move.homing_origin: ', firstObject.gcode_move?.homing_origin);
      this.gcodeMoveHomeOrigin.set(param.gcode_move?.homing_origin[2]);
    }
    if (param.fan?.speed != undefined) {
      // console.log('fan.speed: ', firstObject.fan?.speed);
      this.fanSpeed.set(param.fan?.speed);
    }
    if (param.print_stats?.filename != undefined) {
      // console.log('print_stats.filename: ', firstObject.print_stats?.filename);
      this.printStatsFilename.set(param.print_stats?.filename.slice(0, -6)); //cut ".gcode"
    }
    if (param.print_stats?.print_duration != undefined) {
      // console.log('print_stats.print_duration: ', firstObject.print_stats?.print_duration);
      this.printStatsPrintDuration.set(param.print_stats?.print_duration);
    }
    if (param.print_stats?.state != undefined) {
      // console.log('print_stats.state: ', firstObject.print_stats?.state);
      this.printStatsState.set(param.print_stats?.state);
    }
    if (param.print_stats?.message != undefined) {
      // console.log('print_stats.message: ', firstObject.print_stats?.message);
      this.printStatsMessage.set(param.print_stats?.message);
    }
    if (param.display_status?.progress != undefined) {
      // console.log('display_status.progress: ', firstObject.display_status?.progress);
      this.displayStatusProgress.set(param.display_status?.progress);
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
        this.klippyState.klippyState.set('ready');
        break;
      case 'notify_klippy_disconnected':
        this.klippyState.klippyState.set('disconnected');
        break;
      case 'notify_klippy_error':
        this.klippyState.klippyState.set('error');
        break;
      case 'notify_klippy_startup':
        this.klippyState.klippyState.set('startup');
        break;
      case 'notify_klippy_shutdown':
        this.klippyState.klippyState.set('shutdown');
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
