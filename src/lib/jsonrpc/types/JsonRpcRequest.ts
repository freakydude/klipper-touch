import { JsonRpcClient } from '../JsonRpcClient';
import type { IJsonRpcRequest } from './IJsonRpcRequest';

export class JsonRpcRequest implements IJsonRpcRequest {
  jsonrpc: string;
  id?: string | number | null;
  method: string;
  params?: object | object[];

  /**
   *
   * @param jsonrpc A String specifying the version of the JSON-RPC protocol. MUST be exactly "2.0".
   * @param id An identifier established by the Client that MUST contain a String, Number, or NULL value if included. If it is not included it is assumed to be a notification. The value SHOULD normally not be Null and Numbers SHOULD NOT contain fractional parts.
   * @param method A String containing the name of the method to be invoked. Method names that begin with the word rpc followed by a period character (U+002E or ASCII 46) are reserved for rpc-internal methods and extensions and MUST NOT be used for anything else.
   * @param params A Structured value that holds the parameter values to be used during the invocation of the method. This member MAY be omitted.
   */
  public constructor({ jsonrpc = '2.0', id = JsonRpcClient.generateConnectionId(), method, params }: IJsonRpcRequest) {
    this.jsonrpc = jsonrpc;
    this.id = id;
    this.method = method;
    this.params = params;
  }
}
