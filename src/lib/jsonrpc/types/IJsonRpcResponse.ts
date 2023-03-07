export interface IJsonRpcResponse {
  jsonrpc?: string; // A String specifying the version of the JSON-RPC protocol. MUST be exactly "2.0".
  id: string | number | null; // This member is REQUIRED. It MUST be the same as the value of the id member in the Request Object. If there was an error in detecting the id in the Request object (e.g. Parse error/Invalid Request), it MUST be Null.
}
