import type { JsonRpcClient } from '$lib/jsonrpc/JsonRpcClient';
import type { IJsonRpcRequest } from '$lib/jsonrpc/types/IJsonRpcRequest';
import { JsonRpcRequest } from '$lib/jsonrpc/types/JsonRpcRequest';
import { writable } from 'svelte/store';
import type { INotifyStatusUpdateParams } from './types/INotifyStatusUpdateParams';
import type { IPrinterObjectStores, IPrinterObjectsParams } from './types/IPrinterObjects';

interface IPrinterObjectsSubscribeResult {
  status: INotifyStatusUpdateParams;
}

export class MoonrakerClient extends EventTarget {
  private readonly _jsonRpcClient: JsonRpcClient;

  private fieldRefCounts = new Map<string, number>();
  private appliedSubscriptionHash = '';

  public webhooks: IPrinterObjectStores['webhooks'] = {
    state: writable('disconnected'),
    state_message: writable('')
  };

  public heater_bed: IPrinterObjectStores['heater_bed'] = {
    temperature: writable(0.0),
    target: writable(0.0)
  };

  public extruder: IPrinterObjectStores['extruder'] = {
    temperature: writable(0.0),
    target: writable(0.0),
    pressure_advance: writable(0.0),
    can_extrude: writable(false)
  };

  public toolhead: IPrinterObjectStores['toolhead'] = {
    position: writable([0, 0, 0, 0]),
    homed_axes: writable(''),
    max_accel: writable(0.0),
    axis_minimum: writable([0, 0, 0, 0]),
    axis_maximum: writable([0, 0, 0, 0]),
    max_velocity: writable(0.0),
    square_corner_velocity: writable(0.0),
    minimum_cruise_ratio: writable(0.0)
  };

  public fan: IPrinterObjectStores['fan'] = {
    speed: writable(0.0),
    rpm: writable<number | null>(null)
  };

  public gcode_move: IPrinterObjectStores['gcode_move'] = {
    homing_origin: writable([0.0, 0.0, 0.0, 0.0]),
    speed: writable(0.0),
    speed_factor: writable(0.0),
    extrude_factor: writable(0.0)
  };

  public print_stats: IPrinterObjectStores['print_stats'] = {
    filename: writable(''),
    total_duration: writable(0.0),
    print_duration: writable(0.0),
    filament_used: writable(0.0),
    state: writable('standby'),
    message: writable(''),
    info: {
      current_layer: writable<number | null>(null),
      total_layer: writable<number | null>(null)
    }
  };

  public display_status: IPrinterObjectStores['display_status'] = {
    progress: writable(0.0),
    message: writable('')
  };

  public motion_report: IPrinterObjectStores['motion_report'] = {
    live_position: writable([0.0, 0.0, 0.0, 0.0]),
    live_velocity: writable(0.0),
    live_extruder_velocity: writable(0.0),
    steppers: writable<string[]>([]),
    trapq: writable<string[]>([])
  };

  public configfile: IPrinterObjectStores['configfile'] = {
    save_config_pending: writable(false)
  };

  public quad_gantry_level: IPrinterObjectStores['quad_gantry_level'] = {
    applied: writable(false)
  };

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
        this.fieldRefCounts.clear();
        this.appliedSubscriptionHash = '';
        successful = (await this.requestIdentifyConnection()) && (await this.subscribe({ webhooks: ['state', 'state_message'] }));
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

  public async subscribe(spec: Partial<IPrinterObjectsParams>): Promise<boolean> {
    this.updateRefCounts(spec, 1);
    return this.applyCurrentSubscription();
  }

  public async unsubscribe(spec: Partial<IPrinterObjectsParams>): Promise<boolean> {
    this.updateRefCounts(spec, -1);
    return this.applyCurrentSubscription();
  }

  private updateRefCounts(spec: Partial<IPrinterObjectsParams>, delta: number): void {
    for (const [objectName, fields] of Object.entries(spec) as [string, string[] | null | undefined][]) {
      if (!Array.isArray(fields)) {
        continue;
      }

      for (const field of fields) {
        const key = `${objectName}.${field}`;
        const currentCount = this.fieldRefCounts.get(key) ?? 0;
        const newCount = currentCount + delta;

        if (newCount <= 0) {
          this.fieldRefCounts.delete(key);
        } else {
          this.fieldRefCounts.set(key, newCount);
        }
      }
    }
  }

  private buildCurrentSubscription(): Partial<IPrinterObjectsParams> {
    const subscription: Record<string, string[]> = {};

    for (const key of this.fieldRefCounts.keys()) {
      const splitIndex = key.indexOf('.');
      if (splitIndex <= 0 || splitIndex >= key.length - 1) {
        continue;
      }

      const objectName = key.slice(0, splitIndex);
      const field = key.slice(splitIndex + 1);

      if (!subscription[objectName]) {
        subscription[objectName] = [];
      }

      subscription[objectName].push(field);
    }

    for (const fields of Object.values(subscription)) {
      fields.sort();
    }

    return subscription as Partial<IPrinterObjectsParams>;
  }

  private createCurrentSubscriptionHash(): string {
    return [...this.fieldRefCounts.keys()].sort().join('|');
  }

  private async applyCurrentSubscription(): Promise<boolean> {
    const nextHash = this.createCurrentSubscriptionHash();
    if (nextHash === this.appliedSubscriptionHash) {
      return true;
    }

    const successful = await this.subscribeAndParseParams({ objects: this.buildCurrentSubscription() });
    if (successful) {
      this.appliedSubscriptionHash = nextHash;
    }

    return successful;
  }

  private async subscribeAndParseParams(printerObjects: { objects: Partial<IPrinterObjectsParams> }): Promise<boolean> {
    const subscribeRequest = new JsonRpcRequest({
      method: 'printer.objects.subscribe',
      params: printerObjects
    });
    let successful = true;
    try {
      const response = await this._jsonRpcClient.sendRequest<IPrinterObjectsSubscribeResult>(subscribeRequest);
      console.log('MoonrakerClient.subscribeToPrinterObjects');
      if (response.result !== undefined) {
        this.parseNotifyStatusUpdateParams(response.result.status);
      } else if (response.error) {
        successful = false;
        this.webhooks.state.set('error');
        this.webhooks.state_message.set(response.error.message);
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
  }

  private applyFieldTable(table: Array<readonly [any, { set: (value: any) => void }]>) {
    for (const [value, target] of table) {
      if (value !== undefined) {
        target.set(value);
      }
    }
  }

  private parseNotifyStatusUpdateParams(param: INotifyStatusUpdateParams) {
    const parsers = [
      this.parseWebhooks,
      this.parseHeaterBed,
      this.parseExtruder,
      this.parseToolhead,
      this.parseGcodeMove,
      this.parseFan,
      this.parsePrintStats,
      this.parseDisplayStatus,
      this.parseMotionReport,
      this.parseConfigFile,
      this.parseQuadGantryLevel
    ];

    for (const parser of parsers) {
      parser.call(this, param);
    }
  }

  private parseExtruder(param: INotifyStatusUpdateParams) {
    const extruder = param.extruder;
    if (extruder === undefined) {
      return;
    }

    this.applyFieldTable([
      [extruder.temperature, this.extruder.temperature],
      [extruder.target, this.extruder.target],
      [extruder.pressure_advance, this.extruder.pressure_advance],
      [extruder.can_extrude, this.extruder.can_extrude]
    ]);
  }

  private parseToolhead(param: INotifyStatusUpdateParams) {
    const toolhead = param.toolhead;
    if (toolhead === undefined) {
      return;
    }

    this.applyFieldTable([
      [toolhead.max_accel, this.toolhead.max_accel],
      [toolhead.position, this.toolhead.position],
      [toolhead.homed_axes, this.toolhead.homed_axes],
      [toolhead.axis_minimum, this.toolhead.axis_minimum],
      [toolhead.axis_maximum, this.toolhead.axis_maximum],
      [toolhead.max_velocity, this.toolhead.max_velocity],
      [toolhead.square_corner_velocity, this.toolhead.square_corner_velocity],
      [toolhead.minimum_cruise_ratio, this.toolhead.minimum_cruise_ratio]
    ]);
  }

  private parseGcodeMove(param: INotifyStatusUpdateParams) {
    const gcode_move = param.gcode_move;
    if (gcode_move === undefined) {
      return;
    }

    this.applyFieldTable([
      [gcode_move.homing_origin, this.gcode_move.homing_origin],
      [gcode_move.speed, this.gcode_move.speed],
      [gcode_move.speed_factor, this.gcode_move.speed_factor],
      [gcode_move.extrude_factor, this.gcode_move.extrude_factor]
    ]);
  }

  private parseFan(param: INotifyStatusUpdateParams) {
    const fan = param.fan;
    if (fan === undefined) {
      return;
    }

    this.applyFieldTable([
      [fan.speed, this.fan.speed],
      [fan.rpm, this.fan.rpm]
    ]);
  }

  private parsePrintStats(param: INotifyStatusUpdateParams) {
    const print_stats = param.print_stats;
    if (print_stats === undefined) {
      return;
    }

    this.applyFieldTable([
      [print_stats.filename, this.print_stats.filename],
      [print_stats.total_duration, this.print_stats.total_duration],
      [print_stats.print_duration, this.print_stats.print_duration],
      [print_stats.state, this.print_stats.state],
      [print_stats.message, this.print_stats.message],
      [print_stats.filament_used, this.print_stats.filament_used]
    ]);

    const info = print_stats.info;
    if (info !== undefined) {
      this.applyFieldTable([
          [info.current_layer, this.print_stats.info.current_layer],
          [info.total_layer, this.print_stats.info.total_layer]
      ]);
    }
  }

  private parseDisplayStatus(param: INotifyStatusUpdateParams) {
    const display_status = param.display_status;
    if (display_status === undefined) {
      return;
    }

    this.applyFieldTable([[display_status.progress, this.display_status.progress]]);

    if (display_status.message !== undefined) {
      if (display_status.message !== null) {
        this.display_status.message.set(display_status.message);
      } else {
        this.display_status.message.set('');
      }
    }
  }

  private parseWebhooks(param: INotifyStatusUpdateParams) {
    const webhooks = param.webhooks;
    if (webhooks === undefined) {
      return;
    }

    this.applyFieldTable([
      [webhooks.state, this.webhooks.state],
      [webhooks.state_message, this.webhooks.state_message]
    ]);
  }

  private parseHeaterBed(param: INotifyStatusUpdateParams) {
    const heater_bed = param.heater_bed;
    if (heater_bed === undefined) {
      return;
    }

    this.applyFieldTable([
      [heater_bed.temperature, this.heater_bed.temperature],
      [heater_bed.target, this.heater_bed.target]
    ]);
  }

  private parseMotionReport(param: INotifyStatusUpdateParams) {
    const motion_report = param.motion_report;
    if (motion_report === undefined) {
      return;
    }

    this.applyFieldTable([
      [motion_report.live_extruder_velocity, this.motion_report.live_extruder_velocity],
      [motion_report.live_position, this.motion_report.live_position],
      [motion_report.live_velocity, this.motion_report.live_velocity],
      [motion_report.steppers, this.motion_report.steppers],
      [motion_report.trapq, this.motion_report.trapq]
    ]);
  }

  private parseConfigFile(param: INotifyStatusUpdateParams) {
    const configfile = param.configfile;
    if (configfile === undefined) {
      return;
    }

    this.applyFieldTable([[configfile.save_config_pending, this.configfile.save_config_pending]]);
  }

  private parseQuadGantryLevel(param: INotifyStatusUpdateParams) {
    const quadGantryLevel = param.quad_gantry_level;
    if (quadGantryLevel === undefined) {
      return;
    }

    this.applyFieldTable([[quadGantryLevel.applied, this.quad_gantry_level.applied]]);
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
        this.webhooks.state.set('ready');
        console.log('notify_klippy: ready');
        this.appliedSubscriptionHash = ''; // Moonraker loses all subscriptions on Klippy restart
        await this.applyCurrentSubscription();
        break;
      case 'notify_klippy_disconnected':
        this.webhooks.state.set('disconnected');
        console.log('notify_klippy: disconnected');
        break;
      case 'notify_klippy_error':
        this.webhooks.state.set('error');
        console.log('notify_klippy: error');
        break;
      case 'notify_klippy_startup':
        this.webhooks.state.set('startup');
        console.log('notify_klippy: startup');
        break;
      case 'notify_klippy_shutdown':
        this.webhooks.state.set('shutdown');
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
