export interface IJsonRpcRequest {
  jsonrpc?: string;
  method: string;
  params?: object | object[];
  id?: string | number | null;
}
