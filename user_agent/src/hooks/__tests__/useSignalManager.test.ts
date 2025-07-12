import { SignalManagerClient, Message } from '../../lib/signal-manager-client';

// Mock WebSocket
class MockWebSocket {
  public static instances: MockWebSocket[] = [];
  public static lastInstance: MockWebSocket | null = null;
  public readyState = 1;
  public sentMessages: Array<ArrayBuffer> = [];
  public onopen: (() => void) | null = null;
  public onclose: ((event: any) => void) | null = null;
  public onmessage: ((event: any) => void) | null = null;
  public onerror: ((event: any) => void) | null = null;
  constructor(public url: string) {
    MockWebSocket.instances.push(this);
    MockWebSocket.lastInstance = this;
  }
  send(data: ArrayBuffer) {
    this.sentMessages.push(data);
  }
  close(code?: number, reason?: string) {
    this.readyState = 3;
    if (this.onclose) {
      this.onclose({ code, reason });
    }
  }
}

global.WebSocket = MockWebSocket as any;
(WebSocket as any).OPEN = 1;

describe('SignalManagerClient', () => {
  const config = {
    url: 'localhost',
    port: 8080,
    clientId: 'test-client',
    authToken: 'test-token',
    version: '1.0.0',
    heartbeatInterval: 10,
    timeout: 10,
    reconnectAttempts: 1,
    reconnectDelay: 1,
  };

  let client: SignalManagerClient;
  let logSpy: jest.SpyInstance;

  beforeEach(() => {
    jest.useFakeTimers();
    MockWebSocket.instances = [];
    MockWebSocket.lastInstance = null;
    logSpy = jest.spyOn(console, 'log').mockImplementation(() => {});
    client = new SignalManagerClient(config);
  });

  afterEach(() => {
    logSpy.mockRestore();
    jest.useRealTimers();
  });

  it('should send register message and log on connect', async () => {
    await client.connect();
    // Simulate WebSocket open
    const ws = MockWebSocket.lastInstance!;
    ws.onopen && ws.onopen();
    await Promise.resolve(); // flush microtasks
    // Check that a register message was sent
    expect(ws.sentMessages.length).toBeGreaterThan(0);
    // Check log for register
    expect(logSpy).toHaveBeenCalledWith(
      expect.stringContaining('[REGISTER] Sent register for client_id=test-client')
    );
  });

  it('should send unregister message and log on disconnect', async () => {
    await client.connect();
    const ws = MockWebSocket.lastInstance!;
    ws.onopen && ws.onopen();
    // Clear previous logs
    logSpy.mockClear();
    client.disconnect();
    jest.runAllTimers(); // flush setTimeout for unregister
    // Check that an unregister message was sent
    expect(ws.sentMessages.length).toBeGreaterThan(1);
    // Check log for unregister
    expect(logSpy).toHaveBeenCalledWith(
      expect.stringContaining('[UNREGISTER] Sent unregister for client_id=test-client')
    );
  });
}); 