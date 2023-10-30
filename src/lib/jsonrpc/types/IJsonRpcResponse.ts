import type { IJsonRpcErrorObject } from './IJsonRpcErrorObject';

export interface IJsonRpcResponse {
  jsonrpc?: string; // A String specifying the version of the JSON-RPC protocol. MUST be exactly "2.0".
  id: string | number | null; // This member is REQUIRED. It MUST be the same as the value of the id member in the Request Object. If there was an error in detecting the id in the Request object (e.g. Parse error/Invalid Request), it MUST be Null.
  result?: object | null; // This member is REQUIRED on success. This member MUST NOT exist if there was an error invoking the method. The value of this member is determined by the method invoked on the Server.
  error?: IJsonRpcErrorObject | null; // This member is REQUIRED on error. This member MUST NOT exist if there was no error triggered during invocation. The value for this member MUST be an Object as defined in IJsonRpcErrorObject.
}
