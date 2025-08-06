import { Socket } from 'net';
import { TcpCommand, TcpResponse, ConnectionError } from '../types';

/**
 * TCP Client for communicating with SniperForge Enterprise server
 */
export class TCPClient {
  private socket?: Socket;
  private host: string;
  private port: number;
  private connected = false;
  private pendingRequests = new Map<string, {
    resolve: (value: any) => void;
    reject: (error: Error) => void;
    timeout: ReturnType<typeof setTimeout>;
  }>();
  private requestIdCounter = 0;

  constructor(host = 'localhost', port = 8888) {
    this.host = host;
    this.port = port;
  }

  /**
   * Connect to the SniperForge TCP server
   */
  async connect(): Promise<void> {
    return new Promise((resolve, reject) => {
      try {
        this.socket = new Socket();

        this.socket.connect(this.port, this.host, () => {
          this.connected = true;
          this.setupEventHandlers();
          console.log(`✅ Connected to SniperForge at ${this.host}:${this.port}`);
          resolve();
        });

        this.socket.on('error', (error: Error) => {
          this.connected = false;
          console.error('❌ TCP Connection error:', error);
          reject(new ConnectionError(`Failed to connect to ${this.host}:${this.port}`, { error }));
        });

        this.socket.on('close', () => {
          this.connected = false;
          console.log('🔌 TCP Connection closed');
          this.cleanupPendingRequests();
        });

      } catch (error) {
        reject(new ConnectionError('Failed to create TCP socket', { error }));
      }
    });
  }

  /**
   * Setup event handlers for the socket
   */
  private setupEventHandlers(): void {
    if (!this.socket) return;

    this.socket.on('data', (data: Buffer) => {
      try {
        const responseText = data.toString('utf8');
        
        // Handle multiple responses in one data packet
        const responses = responseText.split('\n').filter((line: string) => line.trim());
        
        for (const responseJson of responses) {
          try {
            const response: TcpResponse = JSON.parse(responseJson);
            this.handleResponse(response);
          } catch (parseError) {
            console.error('❌ Failed to parse response:', responseJson, parseError);
          }
        }
      } catch (error) {
        console.error('❌ Error handling TCP data:', error);
      }
    });
  }

  /**
   * Handle incoming responses from the server
   */
  private handleResponse(response: TcpResponse): void {
    const requestId = response.request_id;
    
    if (requestId && this.pendingRequests.has(requestId)) {
      const request = this.pendingRequests.get(requestId)!;
      
      clearTimeout(request.timeout);
      this.pendingRequests.delete(requestId);
      
      if (response.error) {
        request.reject(new Error(response.error));
      } else {
        request.resolve(response);
      }
    } else {
      // Handle responses without request ID (events, broadcasts)
      console.log('📨 Received broadcast:', response);
    }
  }

  /**
   * Send a command to the server and wait for response
   */
  async sendCommand<T>(command: TcpCommand, timeout = 30000): Promise<TcpResponse<T>> {
    if (!this.connected || !this.socket) {
      throw new ConnectionError('Not connected to SniperForge server');
    }

    return new Promise((resolve, reject) => {
      const requestId = this.generateRequestId();
      const commandWithId = { ...command, request_id: requestId };
      
      // Setup timeout
      const timeoutHandle = setTimeout(() => {
        this.pendingRequests.delete(requestId);
        reject(new Error(`Command timeout after ${timeout}ms`));
      }, timeout);

      // Store pending request
      this.pendingRequests.set(requestId, {
        resolve,
        reject,
        timeout: timeoutHandle
      });

      try {
        const jsonCommand = JSON.stringify(commandWithId) + '\n';
        this.socket!.write(jsonCommand, 'utf8');
      } catch (error) {
        clearTimeout(timeoutHandle);
        this.pendingRequests.delete(requestId);
        reject(new ConnectionError('Failed to send command', { error }));
      }
    });
  }

  /**
   * Send a command without waiting for response (fire and forget)
   */
  sendCommandAsync(command: TcpCommand): void {
    if (!this.connected || !this.socket) {
      throw new ConnectionError('Not connected to SniperForge server');
    }

    try {
      const jsonCommand = JSON.stringify(command) + '\n';
      this.socket.write(jsonCommand, 'utf8');
    } catch (error) {
      throw new ConnectionError('Failed to send async command', { error });
    }
  }

  /**
   * Disconnect from the server
   */
  disconnect(): void {
    if (this.socket) {
      this.socket.destroy();
      this.socket = undefined;
    }
    this.connected = false;
    this.cleanupPendingRequests();
  }

  /**
   * Check if connected to the server
   */
  isConnected(): boolean {
    return this.connected;
  }

  /**
   * Ping the server to check connectivity
   */
  async ping(): Promise<boolean> {
    try {
      const response = await this.sendCommand({ command: 'Ping' }, 5000);
      return response.type === 'Pong';
    } catch {
      return false;
    }
  }

  /**
   * Generate a unique request ID
   */
  private generateRequestId(): string {
    return `req_${++this.requestIdCounter}_${Date.now()}`;
  }

  /**
   * Cleanup all pending requests
   */
  private cleanupPendingRequests(): void {
    for (const [requestId, request] of this.pendingRequests) {
      clearTimeout(request.timeout);
      request.reject(new ConnectionError('Connection closed'));
    }
    this.pendingRequests.clear();
  }

  /**
   * Get current connection status
   */
  getConnectionStatus(): {
    connected: boolean;
    host: string;
    port: number;
    pendingRequests: number;
  } {
    return {
      connected: this.connected,
      host: this.host,
      port: this.port,
      pendingRequests: this.pendingRequests.size
    };
  }
}
