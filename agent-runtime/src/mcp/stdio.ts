/**
 * MCP Stdio Transport
 *
 * Handles JSON-RPC communication over stdio with MCP servers.
 */

import { spawn, ChildProcess } from 'child_process';
import { createInterface } from 'readline';
import { EventEmitter } from 'events';
import {
  JSONRPCRequest,
  JSONRPCResponse,
  JSONRPCNotification,
  MCPServerConfig,
  MCPError,
  MCPConnectionError,
  MCPTimeoutError,
} from './types.js';
import { logger } from '../utils/logger.js';

export interface StdioTransportOptions {
  timeout?: number;
  maxBufferSize?: number;
  env?: Record<string, string>;
}

export class MCPStdioTransport extends EventEmitter {
  private process: ChildProcess | null = null;
  private rl: ReturnType<typeof createInterface> | null = null;
  private pendingRequests: Map<string | number, PendingRequest> = new Map();
  private requestId = 0;
  private connected = false;
  private readonly timeout: number;
  private readonly maxBufferSize: number;
  private readonly env: Record<string, string>;

  constructor(private readonly config: MCPServerConfig, options: StdioTransportOptions = {}) {
    super();
    this.timeout = options.timeout ?? 30000;
    this.maxBufferSize = options.maxBufferSize ?? 10 * 1024 * 1024; // 10MB
    this.env = options.env ?? config.env ?? {};
  }

  /**
   * Connect to the MCP server by spawning the process
   */
  async connect(): Promise<void> {
    if (this.connected) {
      throw new MCPError('Already connected', -32603);
    }

    logger.info('Connecting to MCP server', { name: this.config.name });

    return new Promise((resolve, reject) => {
      const timeoutId = setTimeout(() => {
        reject(new MCPTimeoutError('Connection timeout', this.timeout));
      }, this.timeout);

      let rejected = false;

      // Cleanup function to reject with error
      const rejectWithError = (error: Error) => {
        if (rejected) return;
        rejected = true;
        clearTimeout(timeoutId);
        this.cleanup();
        reject(error);
      };

      try {
        this.process = spawn(this.config.command, this.config.args || [], {
          env: { ...process.env, ...this.env },
          cwd: this.config.cwd,
          stdio: ['pipe', 'pipe', 'pipe'],
        });

        if (!this.process.stdout || !this.process.stdin) {
          rejectWithError(new MCPConnectionError('Failed to create stdio streams'));
          return;
        }

        // Set up stdio interface
        this.rl = createInterface({
          input: this.process.stdout,
          output: this.process.stdin,
          terminal: false,
        });

        // Handle incoming messages
        this.rl.on('line', (line: string) => {
          this.handleMessage(line);
        });

        // Handle process errors - reject the connect promise
        this.process.once('error', (error) => {
          logger.error('MCP server process error', { name: this.config.name, error });
          const connError = new MCPConnectionError('Process error', error);
          this.emit('error', connError);
          rejectWithError(connError);
        });

        // Handle process exit - reject if exit happens too soon
        this.process.once('exit', (code, signal) => {
          logger.info('MCP server process exited', { name: this.config.name, code, signal });
          this.connected = false;
          this.emit('disconnect', { code, signal });

          // Only reject if we haven't successfully connected yet
          if (!rejected) {
            rejectWithError(new MCPConnectionError(`Process exited with code ${code}`));
          }
        });

        // Handle stderr
        if (this.process.stderr) {
          this.process.stderr.on('data', (data) => {
            const message = data.toString().trim();
            if (message) {
              logger.debug('MCP server stderr', { name: this.config.name, message });
              this.emit('stderr', message);
            }
          });
        }

        // Wait a bit to ensure no immediate spawn errors
        // ENOENT errors typically fire within the next tick
        setTimeout(() => {
          if (rejected) return;
          clearTimeout(timeoutId);
          this.connected = true;
          this.emit('connected');
          logger.info('Connected to MCP server', { name: this.config.name });
          resolve();
        }, 50);

      } catch (error: any) {
        rejectWithError(new MCPConnectionError(`Failed to spawn process: ${error.message}`, error));
      }
    });
  }

  /**
   * Disconnect from the MCP server
   */
  async disconnect(): Promise<void> {
    if (!this.connected) {
      return;
    }

    logger.info('Disconnecting from MCP server', { name: this.config.name });

    // Try graceful shutdown
    try {
      await this.request('shutdown', undefined, 5000);
    } catch {
      // Ignore shutdown errors
    }

    this.cleanup();
  }

  /**
   * Cleanup resources
   */
  private cleanup(): void {
    if (this.rl) {
      this.rl.close();
      this.rl = null;
    }

    if (this.process) {
      this.process.kill('SIGTERM');
      setTimeout(() => {
        if (this.process && !this.process.killed) {
          this.process.kill('SIGKILL');
        }
      }, 5000).unref();
      this.process = null;
    }

    this.connected = false;

    // Reject all pending requests
    for (const [id, pending] of this.pendingRequests) {
      pending.reject(new MCPError('Connection closed', -32603));
    }
    this.pendingRequests.clear();
  }

  /**
   * Send a JSON-RPC request and wait for response
   */
  async request<T = any>(
    method: string,
    params?: any,
    timeout?: number
  ): Promise<T> {
    if (!this.connected) {
      throw new MCPError('Not connected', -32603);
    }

    const id = ++this.requestId;
    const requestTimeout = timeout ?? this.timeout;

    return new Promise<T>((resolve, reject) => {
      const timer = setTimeout(() => {
        this.pendingRequests.delete(id);
        reject(new MCPTimeoutError(`Request timeout: ${method}`, requestTimeout));
      }, requestTimeout);

      this.pendingRequests.set(id, { resolve, reject, timer, method });

      const message: JSONRPCRequest = {
        jsonrpc: '2.0',
        id,
        method,
        params,
      };

      this.send(message);
    });
  }

  /**
   * Send a JSON-RPC notification (no response expected)
   */
  notify(method: string, params?: any): void {
    if (!this.connected) {
      throw new MCPError('Not connected', -32603);
    }

    const message: JSONRPCNotification = {
      jsonrpc: '2.0',
      method,
      params,
    };

    this.send(message);
  }

  /**
   * Send a message to the server
   */
  private send(message: JSONRPCRequest | JSONRPCNotification): void {
    if (!this.process || !this.process.stdin) {
      throw new MCPError('No process available', -32603);
    }

    const json = JSON.stringify(message);
    logger.debug('Sending to MCP server', { name: this.config.name, message });

    try {
      this.process.stdin.write(json + '\n');
    } catch (error: any) {
      throw new MCPConnectionError(`Failed to send message: ${error.message}`, error);
    }
  }

  /**
   * Handle an incoming message from the server
   */
  private handleMessage(line: string): void {
    if (!line.trim()) {
      return;
    }

    logger.debug('Received from MCP server', { name: this.config.name, line });

    let message: JSONRPCResponse | JSONRPCNotification;

    try {
      message = JSON.parse(line);
    } catch (error) {
      logger.error('Failed to parse MCP message', { line, error });
      this.emit('error', new MCPError('Invalid JSON', -32700));
      return;
    }

    // Check for response
    if ('id' in message) {
      this.handleResponse(message as JSONRPCResponse);
    } else {
      this.handleNotification(message as JSONRPCNotification);
    }
  }

  /**
   * Handle a JSON-RPC response
   */
  private handleResponse(response: JSONRPCResponse): void {
    const pending = this.pendingRequests.get(response.id);

    if (!pending) {
      logger.warn('Received response for unknown request', { id: response.id });
      return;
    }

    clearTimeout(pending.timer);
    this.pendingRequests.delete(response.id);

    if (response.error) {
      pending.reject(
        new MCPError(
          response.error.message,
          response.error.code,
          response.error.data
        )
      );
    } else {
      pending.resolve(response.result);
    }
  }

  /**
   * Handle a JSON-RPC notification
   */
  private handleNotification(notification: JSONRPCNotification): void {
    logger.debug('Received notification', { method: notification.method });

    // Emit notifications as events
    switch (notification.method) {
      case 'notifications/initialized':
        this.emit('initialized');
        break;

      case 'notifications/cancelled':
        this.emit('cancelled', notification.params);
        break;

      case 'notifications/progress':
        this.emit('progress', notification.params);
        break;

      case 'notifications/message':
        this.emit('message', notification.params);
        break;

      default:
        this.emit('notification', notification);
    }
  }

  /**
   * Check if connected
   */
  isConnected(): boolean {
    return this.connected && this.process !== null;
  }

  /**
   * Get process PID
   */
  getPid(): number | null {
    return this.process?.pid ?? null;
  }
}

interface PendingRequest {
  resolve: (value: any) => void;
  reject: (error: Error) => void;
  timer: NodeJS.Timeout;
  method: string;
}
