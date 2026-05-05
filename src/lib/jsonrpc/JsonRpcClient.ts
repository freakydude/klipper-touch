import { writable } from 'svelte/store';
import type { IJsonRpcRequest } from './types/IJsonRpcRequest';
import type { IJsonRpcResponse } from './types/IJsonRpcResponse';

export class JsonRpcClient extends EventTarget {
  private _ws?: WebSocket;
  private static _id: number = 0;
  private requestTimeout = 30 * 1000;

  public isConnected = writable(false);

  public constructor() {
    super();
  }

  private isSocketOpen(): boolean {
    return this._ws?.readyState === WebSocket.OPEN;
  }

  public connect(url: string | URL): Promise<boolean> {
    const result: Promise<boolean> = new Promise<boolean>((resolve, reject) => {
      try {
        if (this.isSocketOpen()) {
          console.log('JsonRpcClient.connect() WebSocket already open');
          resolve(true);
          return;
        }

        if (!this._ws) {
          let isSettled = false;
          const resolveOnce = (value: boolean) => {
            if (!isSettled) {
              isSettled = true;
              resolve(value);
            }
          };

          this.isConnected.set(false);
          console.log('JsonRpcClient.connect() Create new WebSocket');
          this._ws = new WebSocket(url);

          this._ws.onopen = (event: Event) => {
            console.log('JsonRpcClient.WebSocket.onopen ', event);
            this.isConnected.set(true);
            resolveOnce(true);
          };

          this._ws.onclose = (event: CloseEvent) => {
            this.isConnected.set(false);
            this._ws = undefined;
            console.log('JsonRpcClient.WebSocket.onclose ', event);
            resolveOnce(false);
          };

          this._ws.onerror = (event: Event) => {
            this.isConnected.set(false);
            this._ws = undefined;
            console.warn('JsonRpcClient.WebSocket.onerror ', event);
            resolveOnce(false);
          };

          this._ws.onmessage = (event: MessageEvent) => {
            const request: IJsonRpcRequest | IJsonRpcRequest[] = JSON.parse(event.data);

            // console.log('connect ws.onmessage: single', request);
            if (Array.isArray(request)) {
              // batch request - send notification for every notification inside
              console.log('JsonRpcClient.WebSocket.onmessage received batch request', request);
              for (const singleRequest of request) {
                if (!singleRequest.id) {
                    // console.log('connect ws.onmessage: notification', singleRequest);
                  this.dispatchEvent(
                      new CustomEvent<IJsonRpcRequest>('notification', {
                        detail: singleRequest
                    })
                  );
                }
              }
            } else {
              // single request
              if (!request.id) {
                // console.log('connect ws.onmessage: received singleRequest', request);
                this.dispatchEvent(
                  new CustomEvent<IJsonRpcRequest>('notification', {
                    detail: request
                  })
                );
              }
            }
          };
        } else {
          console.log('JsonRpcClient.connect() WebSocket is initializing');
          resolve(false);
        }
      } catch (error) {
        console.error('JsonRpcClient.connect() ', error);
        reject(new Error(`Websocket could not be initialized: ${String(error)}`));
      }
    });

    return result;
  }

  public disconnect(): Promise<boolean> {
    const result: Promise<boolean> = new Promise<boolean>((resolve, reject) => {
      try {
        if (this._ws) {
          this._ws.close();
          resolve(true);
        } else {
          console.warn('JsonRpcClient.disconnect() WS ws not OPEN');
          resolve(false);
        }
      } catch (error) {
        console.error('JsonRpcClient.disconnect() ', error);
        reject(new Error(`Websocket could not be disconnected: ${String(error)}`));
      }
    });

    return result;
  }

  public sendRequest<TResult = object>(request: IJsonRpcRequest): Promise<IJsonRpcResponse<TResult>> {
    let promise: Promise<IJsonRpcResponse<TResult>>;
    const socket = this._ws;

    if (!socket || socket.readyState !== WebSocket.OPEN) {
      const notConnected = 'Websocket is not connected, send request failed';
      console.error('JsonRpcClient.sendRequest() ', notConnected);
      promise = Promise.reject(notConnected);
    } else {
      promise = new Promise<IJsonRpcResponse<TResult>>((resolve, reject) => {
        const timeout = setTimeout(() => {
          socket.removeEventListener('message', parser);

          console.warn('JsonRpcClient.sendRequest() Timeout send request: ', request);
          reject('Websocket Timeout send request');
        }, this.requestTimeout);

        const parser = (event: MessageEvent) => {
          const response: IJsonRpcResponse<TResult> = JSON.parse(event.data);

          if (request.id == response.id) {
            clearTimeout(timeout);
            socket.removeEventListener('message', parser);

            console.log('Websocket got response for request id:', response.id, 'response:', response);

            resolve(response);
          }
          // else {
          //   console.log('parser some message - request: ', request, ' response:', response);
          // }
        };
        socket.addEventListener('message', parser);
      });

      try {
        // console.log("request", request)
        socket.send(JSON.stringify(request));
      } catch (error) {
        console.log(error);
        promise = Promise.reject(error);
      }
    }

    return promise;
  }

  public sendBatchRequest<TResult = object>(requests: IJsonRpcRequest[]): Promise<IJsonRpcResponse<TResult>[]> {
    let promise: Promise<IJsonRpcResponse<TResult>[]>;
    const socket = this._ws;
    const expectedRequestIds = requests
      .map((request) => request.id)
      .filter((id): id is string | number => id !== undefined && id !== null)
      .map((id) => String(id));

    if (!socket || socket.readyState !== WebSocket.OPEN) {
      const notConnected = 'Websocket is not connected, send batch request failed';
      console.log(notConnected);
      promise = Promise.reject(notConnected);
    } else if (!expectedRequestIds.length) {
      promise = Promise.reject('Batch request failed: no request ids provided');
    } else {
      promise = new Promise<IJsonRpcResponse<TResult>[]>((resolve, reject) => {
        const timeout = setTimeout(() => {
          socket.removeEventListener('message', parser);

          console.warn('Websocket Timeout send batch request: ', requests);
          reject('Websocket Timeout send batch request');
        }, this.requestTimeout);

        const parser = (event: MessageEvent) => {
            const parsedEventData: IJsonRpcResponse<TResult> | IJsonRpcResponse<TResult>[] = JSON.parse(event.data);

          // TODO if request is a invalid json -> response is a single error json. code don't care about this right now
            if (Array.isArray(parsedEventData)) {
              console.log('Websocket received batch responses', parsedEventData);
              const responsesWithId = parsedEventData.filter(
                (response): response is IJsonRpcResponse<TResult> & { id: string | number } =>
                  response.id !== undefined && response.id !== null
              );
              const responseIds = responsesWithId.map((response) => String(response.id));
              const hasExactIdMatch =
                responseIds.length === expectedRequestIds.length &&
                expectedRequestIds.every((expectedId) => responseIds.includes(expectedId));

              if (hasExactIdMatch) {
              clearTimeout(timeout);
                socket.removeEventListener('message', parser);
              console.log('Websocket got response for request id:', responsesWithId[0].id, ' Responses:', responsesWithId);
              resolve(responsesWithId);
            }
            // else {
              //   console.log('parser some message - requests:', requests, ' responses:', parsedEventData);
            // }
          }
        };
        socket.addEventListener('message', parser);
      });

      try {
        // console.log("request", request)
        socket.send(JSON.stringify(requests));
      } catch (error) {
        console.log(error);
        promise = Promise.reject(error);
      }
    }

    return promise;
  }

  public static generateConnectionId(): string {
    this._id++;

    return this._id.toString();
  }
}
