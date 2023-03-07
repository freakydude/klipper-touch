import type { IJsonRpcRequest } from './IJsonRpcRequest';

export class JsonRpcNotification implements IJsonRpcRequest {
  jsonrpc: string;
  method: string;
  params?: object | object[];

  /**
   *
   * @param jsonrpc A String specifying the version of the JSON-RPC protocol. MUST be exactly "2.0".
   * @param method A String containing the name of the method to be invoked. Method names that begin with the word rpc followed by a period character (U+002E or ASCII 46) are reserved for rpc-internal methods and extensions and MUST NOT be used for anything else.
   * @param params A Structured value that holds the parameter values to be used during the invocation of the method. This member MAY be omitted.
   */
  public constructor({ jsonrpc = '2.0', method, params }: IJsonRpcRequest) {
    this.jsonrpc = jsonrpc;
    this.method = method;
    this.params = params;
  }
}
