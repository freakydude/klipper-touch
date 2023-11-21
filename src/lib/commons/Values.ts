import { client } from '$lib/base.svelte';
import { JsonRpcRequest } from '$lib/jsonrpc/types/JsonRpcRequest';
import type { MoonrakerClient } from '$lib/moonraker/MoonrakerClient';
import type { IFileMetadata } from '$lib/moonraker/types/IFileMetadata';
import type { IThumbnail } from '$lib/moonraker/types/IThumbnail';
import { readable, type Readable } from 'svelte/store';

export class Values {
  private _moonrakerClient: MoonrakerClient;

  public clockFormatter: Intl.DateTimeFormat;
  public clock: Readable<Date>;

  public constructor(moonrakerClient: MoonrakerClient) {
    // super();
    this._moonrakerClient = moonrakerClient;
    this.clockFormatter = this.createClockFormatter();
    this.clock = this.createClock();
  }

  public async getFileMetadata(filename: string): Promise<IFileMetadata | null> {
    const requestMetadata = new JsonRpcRequest({
      method: 'server.files.metadata',
      params: {
        filename: filename
      }
    });
    const response = await client.sendRequest(requestMetadata);
    let metadata: IFileMetadata | null = null;

    if (response.error === undefined) {
      metadata = response.result as IFileMetadata;
    } else {
      console.warn('getFileMetadata.response.error: ', response.error);
    }

    return metadata;
  }

  public async getLargestAbsoluteThumbnailPath(thumbnails: IThumbnail[]): Promise<string> {
    let path = '';

    if (Array.isArray(thumbnails) && thumbnails.length > 0) {
      const thumbnail = thumbnails.sort((n1, n2) => n2.width - n1.width)[0];
      path = 'http://127.0.0.1' + '/server/files/gcodes/' + thumbnail.relative_path;
    } else {
      path = '';
    }

    return path;
  }

  private createClockFormatter(): Intl.DateTimeFormat {
    return new Intl.DateTimeFormat('de', {
      hour12: false,
      hour: '2-digit',
      minute: '2-digit'
    });
  }

  private createClock(): Readable<Date> {
    return readable(new Date(), function start(set) {
      const interval = setInterval(() => {
        set(new Date());
      }, 1000);

      return function stop() {
        clearInterval(interval);
      };
    });
  }
}
