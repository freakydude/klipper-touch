import { client } from '$lib/base.svelte';
import { JsonRpcRequest } from '$lib/jsonrpc/types/JsonRpcRequest';
import type { MoonrakerClient } from '$lib/moonraker/MoonrakerClient';
import type { IFileMetadata } from '$lib/moonraker/types/IFileMetadata';
import type { IThumbnail } from '$lib/moonraker/types/IThumbnail';
import { readable, writable, type Readable } from 'svelte/store';

export class Values {
  private _moonrakerClient: MoonrakerClient;

  public clockFormatter: Intl.DateTimeFormat;
  public clock: Readable<Date>;
  public stepsMove = writable(50);
  public stepsTemp = writable(20);
  public stepsBaby = writable(0.05);
  public stepsExtrusion = writable(10);
  public stepsExtrusionSpeed = writable(3);
  public fileMetadata = writable<IFileMetadata | null>(null);
  public largestAbsoluteThumbnailPath = writable('');

  public constructor(moonrakerClient: MoonrakerClient) {
    // super();
    this._moonrakerClient = moonrakerClient;
    this.clockFormatter = this.createClockFormatter();
    this.clock = this.createClock();
  }

  public async getFileMetadata(relativeFilename: string): Promise<IFileMetadata | null> {
    const requestMetadata = new JsonRpcRequest({
      method: 'server.files.metadata',
      params: {
        filename: relativeFilename
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

  public async getThumbnails(relativeFilename: string): Promise<IThumbnail[] | null> {
    const request = new JsonRpcRequest({
      method: 'server.files.thumbnails',
      params: {
        filename: relativeFilename
      }
    });

    const response = await client.sendRequest(request);
    let thumbnails: IThumbnail[] | null = null;

    if (response.error === undefined) {
      thumbnails = response.result as IThumbnail[];
    } else {
      console.warn('getThumbnails.response.error: ', response.error);
    }

    return thumbnails;
  }

  public async getLargestAbsoluteThumbnailPath(moonrakerApi: URL, thumbnails: IThumbnail[]): Promise<string> {
    let path = '';

    if (Array.isArray(thumbnails) && thumbnails.length > 0) {
      const thumbnail = thumbnails.sort((n1, n2) => n2.width - n1.width)[0];
      path = moonrakerApi + 'server/files/gcodes/' + thumbnail.relative_path;
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
