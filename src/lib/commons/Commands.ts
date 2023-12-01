import { client } from '$lib/base.svelte';
import type { JsonRpcClient } from '$lib/jsonrpc/JsonRpcClient';
import { JsonRpcRequest } from '$lib/jsonrpc/types/JsonRpcRequest';
import type { IFile } from '$lib/moonraker/types/IFile';

export class Commands {
  private _jsonRpcClient: JsonRpcClient;
  private speedZ = 20;
  private speedXY = 50;

  public constructor(jsonRpcClient: JsonRpcClient) {
    // super();
    this._jsonRpcClient = jsonRpcClient;
  }

  public async listFiles(): Promise<IFile[]> {
    const listFilesRequest = new JsonRpcRequest({
      method: 'server.files.list',
      params: {}
    });

    const files: Promise<IFile[]> = new Promise<IFile[]>((resolve, reject) => {
      const response = client.sendRequest(listFilesRequest);
      response.then((response) => {
        if (response.result !== undefined) {
          resolve(response.result as IFile[]);
        } else {
          reject(response.error);
        }
      });
    });

    return files;
  }

  public async extrude(distance: number, speed: number) {
    const request = new JsonRpcRequest({
      method: 'printer.gcode.script',
      params: {
        script: 'G91\nG1 E' + distance + ' F' + speed * 60
      }
    });
    await this._jsonRpcClient.sendRequest(request);
  }

  public async changeOffset(relativeSteps: number) {
    const request = new JsonRpcRequest({
      method: 'printer.gcode.script',
      params: {
        script: 'SET_GCODE_OFFSET Z_ADJUST=' + relativeSteps + ' MOVE=1'
      }
    });
    await this._jsonRpcClient.sendRequest(request);
  }

  public async resetOffset() {
    const request = new JsonRpcRequest({
      method: 'printer.gcode.script',
      params: {
        script: 'SET_GCODE_OFFSET Z=0 MOVE=1 MOVE_SPEED=' + this.speedZ
      }
    });
    await this._jsonRpcClient.sendRequest(request);
  }

  public async saveConfig() {
    const request = new JsonRpcRequest({
      method: 'printer.gcode.script',
      params: {
        script: 'SAVE_CONFIG'
      }
    });
    await this._jsonRpcClient.sendRequest(request);
  }

  public async levelQuadGantry() {
    const request = new JsonRpcRequest({
      method: 'printer.gcode.script',
      params: {
        script: 'QUAD_GANTRY_LEVEL'
      }
    });
    await this._jsonRpcClient.sendRequest(request);
  }

  public async homeXY() {
    const request = new JsonRpcRequest({
      method: 'printer.gcode.script',
      params: {
        script: 'G28 X Y'
      }
    });
    await this._jsonRpcClient.sendRequest(request);
  }

  public async homeZ(isHomedXY: boolean) {
    let homeRequest;

    if (!isHomedXY) {
      homeRequest = new JsonRpcRequest({
        method: 'printer.gcode.script',
        params: {
          script: 'G28'
        }
      });
    } else {
      homeRequest = new JsonRpcRequest({
        method: 'printer.gcode.script',
        params: {
          script: 'G28 Z'
        }
      });
    }
    await this._jsonRpcClient.sendRequest(homeRequest);
  }

  public async setNozzleTemperature(temperature: number) {
    const request = new JsonRpcRequest({
      method: 'printer.gcode.script',
      params: {
        script: 'M104 S' + temperature
      }
    });
    await this._jsonRpcClient.sendRequest(request);
  }

  public async setBedTemperature(temperature: number) {
    const request = new JsonRpcRequest({
      method: 'printer.gcode.script',
      params: {
        script: 'M140 S' + temperature
      }
    });
    await this._jsonRpcClient.sendRequest(request);
  }

  public async moveAbsolute(x: number | undefined, y: number | undefined, z: number | undefined) {
    const xPart = x === undefined ? '' : ' X' + x;
    const yPart = y === undefined ? '' : ' Y' + y;
    const zPart = z === undefined ? '' : ' Z' + z;

    const moveDownRequest = new JsonRpcRequest({
      method: 'printer.gcode.script',
      params: {
        script: 'G90\nG0' + xPart + yPart + zPart + ' F' + this.speedZ * 60
      }
    });
    await this._jsonRpcClient.sendRequest(moveDownRequest);
  }

  public async startPrint(filename: string) {
    const resumeRequest = new JsonRpcRequest({
      method: 'printer.print.start',
      params: {
        filename: filename // selectedFile + '.gcode'
      }
    });
    await this._jsonRpcClient.sendRequest(resumeRequest);
  }

  public async resumePrint() {
    const resumeRequest = new JsonRpcRequest({
      method: 'printer.print.resume',
      params: {}
    });
    await this._jsonRpcClient.sendRequest(resumeRequest);
  }

  public async pausePrint() {
    const pauseRequest = new JsonRpcRequest({
      method: 'printer.print.pause',
      params: {}
    });
    await this._jsonRpcClient.sendRequest(pauseRequest);
  }

  public async cancelPrint() {
    const cancelRequest = new JsonRpcRequest({
      method: 'printer.print.cancel',
      params: {}
    });
    await this._jsonRpcClient.sendRequest(cancelRequest);
  }

  public async disableSteppers() {
    const request = new JsonRpcRequest({
      method: 'printer.gcode.script',
      params: {
        script: 'M84'
      }
    });
    await this._jsonRpcClient.sendRequest(request);
  }

  public async printerRestart() {
    const stopRequest = new JsonRpcRequest({
      method: 'printer.restart',
      params: {}
    });
    await this._jsonRpcClient.sendRequest(stopRequest);
  }

  public async firmwareRestart() {
    const stopRequest = new JsonRpcRequest({
      method: 'printer.firmware_restart',
      params: {}
    });
    await this._jsonRpcClient.sendRequest(stopRequest);
  }

  public async emergencyStop() {
    const emergencyStopRequest = new JsonRpcRequest({
      method: 'printer.emergency_stop',
      params: {}
    });

    await this._jsonRpcClient.sendRequest(emergencyStopRequest);
  }
}
