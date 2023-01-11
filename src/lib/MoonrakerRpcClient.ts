import type { IJsonRpcErrorResponse, IJsonRpcSuccessResponse, JsonRpcClient, JsonRpcRequest } from './JsonRpcClient';

export class MoonrakerRpcClient {
  jsonRpcClient: JsonRpcClient;

  public constructor(jsonRpcClient: JsonRpcClient) {
    this.jsonRpcClient = jsonRpcClient;
  }

  public async requestIdentifyConnection(): Promise<IJsonRpcSuccessResponse | IJsonRpcErrorResponse> {
    const identifyConnectionRequest: JsonRpcRequest = {
      jsonrpc: '2.0',
      method: 'server.connection.identify',
      params: {
        client_name: 'klipper-touch',
        version: '0.0.1',
        type: 'display',
        url: 'https://github.com/freakydude/klipper-touch'
      },
      id: this.jsonRpcClient.generateConnectionId()
    };

    let result = await this.jsonRpcClient.sendMessage(identifyConnectionRequest);
    // console.log('requestIdentifyConnection - isConnected', result);

    return result;
  }
}
