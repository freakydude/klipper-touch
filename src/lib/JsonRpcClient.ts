export interface IJsonRpcRequest {
  jsonrpc: string;
  method: string;
  params?: any[] | object;
  id?: string | number | null;
}

export class JsonRpcRequest implements IJsonRpcRequest {
  jsonrpc: string;
  id?: string | number | null;
  method: string;
  params?: object | any[];

  /**
   *
   * @param jsonrpc A String specifying the version of the JSON-RPC protocol. MUST be exactly "2.0".
   * @param id An identifier established by the Client that MUST contain a String, Number, or NULL value if included. If it is not included it is assumed to be a notification. The value SHOULD normally not be Null and Numbers SHOULD NOT contain fractional parts.
   * @param method A String containing the name of the method to be invoked. Method names that begin with the word rpc followed by a period character (U+002E or ASCII 46) are reserved for rpc-internal methods and extensions and MUST NOT be used for anything else.
   * @param params A Structured value that holds the parameter values to be used during the invocation of the method. This member MAY be omitted.
   */
  public constructor(method: string, id?: string | number | null, params?: any[] | object) {
    this.jsonrpc = '2.0';
    this.id = id;
    this.method = method;
    this.params = params;
  }
}

export interface IJsonRpcResponse {
  jsonrpc: string;
  id?: string | number | null;
}

export interface IJsonRpcSuccessResponse extends IJsonRpcResponse {
  result: any;
}

export class JsonRpcSuccessResponse implements IJsonRpcSuccessResponse {
  result: any;
  jsonrpc: string;
  id?: string | number | null;

  /**
   *
   * @param jsonrpc A String specifying the version of the JSON-RPC protocol. MUST be exactly "2.0".
   * @param id This member is REQUIRED. It MUST be the same as the value of the id member in the Request Object. If there was an error in detecting the id in the Request object (e.g. Parse error/Invalid Request), it MUST be Null.
   * @param result This member is REQUIRED on success. This member MUST NOT exist if there was an error invoking the method. The value of this member is determined by the method invoked on the Server.
   */
  constructor(result: any, id?: string | number | null) {
    this.jsonrpc = '2.0';
    this.id = id;
    this.result = result;
  }
}

export interface IJsonRpcErrorObject {
  code: number;
  message: string;
  data?: object;
}

export class JsonRpcErrorObject implements IJsonRpcErrorObject {
  code: number;
  message: string;
  data?: object;

  /**
   *
   * @param code A Number that indicates the error type that occurred. This MUST be an integer.
   * @param message A String providing a short description of the error. The message SHOULD be limited to a concise single sentence.
   * @param data A Primitive or Structured value that contains additional information about the error. This may be omitted. The value of this member is defined by the Server (e.g. detailed error information, nested errors etc.).
   */
  public constructor(code: number, message: string, data?: object) {
    this.code = code;
    this.message = message;
    this.data = data;
  }
}

export interface IJsonRpcErrorResponse extends IJsonRpcResponse {
  error: IJsonRpcErrorObject;
}

export class JsonRpcErrorResponse implements IJsonRpcErrorResponse {
  error: IJsonRpcErrorObject;
  jsonrpc: string;
  id?: string | number | null;

  /**
   *
   * @param error This member is REQUIRED on error. This member MUST NOT exist if there was no error triggered during invocation. The value for this member MUST be an Object as defined in section 5.1.
   * @param jsonrpc A String specifying the version of the JSON-RPC protocol. MUST be exactly "2.0".
   * @param id This member is REQUIRED. It MUST be the same as the value of the id member in the Request Object. If there was an error in detecting the id in the Request object (e.g. Parse error/Invalid Request), it MUST be Null.
   */
  public constructor(error: IJsonRpcErrorObject, id?: string | number | null) {
    this.id = id;
    this.jsonrpc = '2.0';
    this.error = error;
  }
}

export class JsonRpcClient extends EventTarget {
  private isConnected: Boolean = false;
  private ws: WebSocket;

  public constructor(url: string | URL) {
    super();
    this.ws = new WebSocket(url);
  }

  public async connect(): Promise<boolean> {
    this.isConnected = false;

    let result: Promise<boolean> = new Promise<boolean>((resolve, reject) => {
      this.ws.onopen = (event: Event) => {
        console.log('connect ws.onopen', event);
        this.isConnected = true;
        resolve(true);
      };

      this.ws.onclose = (event: CloseEvent) => {
        this.isConnected = false;
        console.log('connect ws.onclose', event);
        reject(event);
      };

      this.ws.onerror = (event: Event) => {
        this.isConnected = false;
        console.log('connect ws.onerror', event);
        reject(event);
      };

      this.ws.onmessage = (event: MessageEvent) => {
        const response: IJsonRpcErrorResponse | IJsonRpcSuccessResponse = JSON.parse(event.data);

        if (!response.id) {
          //console.log('connect ws.onmessage: notification', response);

          this.dispatchEvent(new CustomEvent<IJsonRpcErrorResponse | IJsonRpcSuccessResponse>('notification', { detail: response }));
        }
      };
    });

    return result;
  }

  public async disconnect(): Promise<boolean> {
    let result: Promise<boolean> = new Promise<boolean>((resolve, reject) => {
      if (this.isConnected) {
        this.ws.close();
        resolve(true);
      } else {
        reject('WebSocket was not connected');
      }
    });

    return result;
  }

  public async sendMessage(message: IJsonRpcRequest): Promise<IJsonRpcSuccessResponse | IJsonRpcErrorResponse> {
    let promise: Promise<IJsonRpcSuccessResponse | IJsonRpcErrorResponse>;

    if (!this.isConnected) {
      console.log('sendMessage ws not connected');
      promise = Promise.reject('Reject: ws not connected');
    } else if (this.ws.readyState <= 1) {
      console.log('sendMessage ws.readyState <=1', message);

      promise = new Promise<IJsonRpcSuccessResponse | IJsonRpcErrorResponse>((resolve, reject) => {
        let timeout = setTimeout(() => {
          this.ws.removeEventListener('message', parser);

          console.log('sendMessage - timeout');
          reject('sendMessage - timeout');
        }, 30 * 1000);

        let parser = (event: MessageEvent) => {
          const data: IJsonRpcSuccessResponse | IJsonRpcErrorResponse = JSON.parse(event.data);

          if (message.id == data.id) {
            this.ws.removeEventListener('message', parser);
            clearTimeout(timeout);

            console.log('parser response for id:', data.id, 'data:', data);

            resolve(data);
          } else {
            console.log('parser some message: ', message, ' data:', data);
          }
        };
        this.ws.addEventListener('message', parser);
      });

      this.ws.send(JSON.stringify(message));
    } else {
      console.log('ws not ready reject');
      promise = Promise.reject('ws not ready');
    }

    return promise;
  }

  public generateConnectionId(): string {
    return Math.floor(Math.random() * 1000000).toString();
  }
}
