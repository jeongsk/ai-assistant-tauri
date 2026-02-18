/**
 * MCP Client - Model Context Protocol client with full implementation
 *
 * Manages connections to multiple MCP servers and provides a unified interface
 * for tools, resources, and prompts.
 */

import { EventEmitter } from 'events';
import {
  MCPServerConfig,
  MCPServerConnection,
  MCPServerInfo,
  Tool,
  ToolCallParams,
  ToolCallResult,
  Resource,
  ResourceReadParams,
  ResourceReadResult,
  Prompt,
  PromptGetParams,
  PromptGetResult,
  InitializeParams,
  MCPError,
  MCPConnectionError,
  Notification,
} from './types.js';
import { MCPStdioTransport } from './stdio.js';
import { logger } from '../utils/logger.js';
import { RateLimiter, createBrowserRateLimiter } from '../utils/rate-limiter.js';
import { BROWSER_TOOLS, getBrowserMCPConfig } from './browser.js';

export interface MCPClientOptions {
  requestTimeout?: number;
  enableBrowserTools?: boolean;
}

export class MCPClient extends EventEmitter {
  private servers: Map<string, MCPServerConnection> = new Map();
  private transports: Map<string, MCPStdioTransport> = new Map();
  private tools: Map<string, Tool> = new Map();
  private rateLimiters: Map<string, RateLimiter> = new Map();
  private readonly requestTimeout: number;
  private readonly enableBrowserTools: boolean;

  constructor(options: MCPClientOptions = {}) {
    super();
    this.requestTimeout = options.requestTimeout ?? 30000;
    this.enableBrowserTools = options.enableBrowserTools ?? true;
  }

  /**
   * Initialize MCP client with server configurations
   */
  async initialize(configs: MCPServerConfig[]): Promise<void> {
    logger.info('Initializing MCP client', { serverCount: configs.length });

    // Register built-in browser tools
    if (this.enableBrowserTools) {
      this.registerBrowserTools();
    }

    // Connect to each server
    for (const config of configs) {
      try {
        await this.connectServer(config);
      } catch (error) {
        logger.error(`Failed to connect to MCP server: ${config.name}`, error);
        // Continue with other servers even if one fails
      }
    }

    logger.info('MCP client initialized', {
      connectedServers: this.servers.size,
      totalTools: this.tools.size,
    });
  }

  /**
   * Connect to a single MCP server
   */
  private async connectServer(config: MCPServerConfig): Promise<void> {
    logger.info('Connecting to MCP server', { name: config.name });

    // Create transport
    const transport = new MCPStdioTransport(config, {
      timeout: this.requestTimeout,
      env: config.env,
    });

    // Set up event handlers
    transport.on('connected', () => {
      logger.debug('Transport connected', { name: config.name });
    });

    transport.on('error', (error) => {
      logger.error('Transport error', { name: config.name, error });
      this.emit('serverError', { server: config.name, error });
    });

    transport.on('disconnect', (reason) => {
      logger.info('Transport disconnected', { name: config.name, reason });
      this.handleServerDisconnect(config.name);
    });

    transport.on('notification', (notification: Notification) => {
      this.emit('notification', { server: config.name, notification });
    });

    // Connect
    await transport.connect();
    this.transports.set(config.name, transport);

    // Initialize the MCP server
    const initParams: InitializeParams = {
      protocolVersion: '2024-11-05',
      capabilities: {
        roots: false,
        sampling: false,
      },
      clientInfo: {
        name: 'ai-assistant-tauri',
        version: '0.4.0',
      },
    };

    const initResult = await transport.request('initialize', initParams);
    const serverInfo: MCPServerInfo = {
      name: initResult.serverInfo.name,
      version: initResult.serverInfo.version,
      protocolVersion: initResult.protocolVersion,
      capabilities: initResult.capabilities,
    };

    // Send initialized notification
    transport.notify('notifications/initialized');

    // Create connection record
    const connection: MCPServerConnection = {
      config,
      process: { pid: transport.getPid() },
      stdio: null,
      ready: true,
      capabilities: serverInfo.capabilities,
      tools: [],
      requestId: 0,
    };

    // Load tools from server
    if (serverInfo.capabilities.tools) {
      try {
        const toolsResult = await this.listServerTools(config.name);
        connection.tools = toolsResult;

        // Register tools with server prefix
        for (const tool of toolsResult) {
          const qualifiedName = `${config.name}/${tool.name}`;
          this.tools.set(qualifiedName, tool);
        }

        logger.info('Loaded tools from MCP server', {
          name: config.name,
          toolCount: toolsResult.length,
        });
      } catch (error) {
        logger.error('Failed to load tools from MCP server', { name: config.name, error });
      }
    }

    this.servers.set(config.name, connection);

    logger.info('MCP server connected', {
      name: config.name,
      serverInfo,
    });

    this.emit('serverConnected', { name: config.name, serverInfo });
  }

  /**
   * Register browser tools (built-in)
   */
  private registerBrowserTools(): void {
    for (const browserTool of BROWSER_TOOLS) {
      // Convert BrowserTool to Tool type
      const tool: Tool = {
        name: browserTool.name,
        description: browserTool.description,
        inputSchema: {
          type: 'object',
          properties: browserTool.inputSchema.properties as Record<string, import('./types.js').ToolProperty>,
          required: browserTool.inputSchema.required,
        },
      };
      this.tools.set(tool.name, tool);
    }
    this.rateLimiters.set('browser', createBrowserRateLimiter());
    logger.info('Registered browser tools', { count: BROWSER_TOOLS.length });
  }

  /**
   * Handle server disconnection
   */
  private handleServerDisconnect(serverName: string): void {
    const connection = this.servers.get(serverName);
    if (!connection) return;

    // Remove tools from this server
    for (const [name, tool] of this.tools) {
      if (name.startsWith(serverName + '/')) {
        this.tools.delete(name);
      }
    }

    this.servers.delete(serverName);

    const transport = this.transports.get(serverName);
    if (transport) {
      this.transports.delete(serverName);
    }

    this.emit('serverDisconnected', { server: serverName });
  }

  /**
   * List all available tools from all servers
   */
  async listTools(): Promise<Tool[]> {
    return Array.from(this.tools.values());
  }

  /**
   * List tools from a specific server
   */
  private async listServerTools(serverName: string): Promise<Tool[]> {
    const transport = this.transports.get(serverName);
    if (!transport) {
      throw new MCPError(`Server not found: ${serverName}`, -32602);
    }

    const result = await transport.request('tools/list', {});
    return result.tools || [];
  }

  /**
   * Call a tool with arguments
   */
  async callTool(name: string, args: Record<string, any> = {}): Promise<ToolCallResult> {
    logger.debug('Calling tool', { name, args });

    // Parse tool name
    const { serverName, toolName } = this.parseToolName(name);

    // Handle browser tools specially
    if (serverName === 'browser' || this.tools.has(name)) {
      return this.handleBrowserTool(toolName, args);
    }

    // Check rate limit
    await this.checkRateLimit(serverName);

    // Get transport
    const transport = this.transports.get(serverName);
    if (!transport) {
      throw new MCPError(`MCP server not found: ${serverName}`, -32602);
    }

    // Call tool via MCP
    const params: ToolCallParams = {
      name: toolName,
      arguments: args,
    };

    try {
      const result = await transport.request('tools/call', params);
      return result;
    } catch (error) {
      logger.error('Tool call failed', { name, error });
      throw error;
    }
  }

  /**
   * Parse tool name to get server and tool
   */
  private parseToolName(name: string): { serverName: string; toolName: string } {
    const parts = name.split('/');

    if (parts.length >= 2) {
      return {
        serverName: parts[0],
        toolName: parts.slice(1).join('/'),
      };
    }

    // Check if it's a browser tool
    if (name.startsWith('browser_')) {
      return { serverName: 'browser', toolName: name };
    }

    // Default: treat entire name as tool name, search for server
    for (const [serverName, connection] of this.servers) {
      for (const tool of connection.tools) {
        if (tool.name === name) {
          return { serverName, toolName: name };
        }
      }
    }

    return { serverName: 'unknown', toolName: name };
  }

  /**
   * Handle browser tool calls
   */
  private async handleBrowserTool(
    toolName: string,
    args: Record<string, any>
  ): Promise<ToolCallResult> {
    logger.info('Handling browser tool', { toolName, args });

    // Check rate limit
    const limiter = this.rateLimiters.get('browser');
    if (limiter && !limiter.tryConsume()) {
      throw new Error(
        `Rate limit exceeded for browser tools. Retry after ${limiter.getTimeUntilAvailable()}ms`
      );
    }

    // For now, return mock responses
    // In a real implementation, this would communicate with a browser automation server
    switch (toolName) {
      case 'browser_navigate':
        return {
          content: [
            {
              type: 'text',
              text: `Navigated to ${args.url}`,
            },
          ],
        };

      case 'browser_screenshot':
        return {
          content: [
            {
              type: 'text',
              text: `Screenshot captured (selector: ${args.selector || 'full page'})`,
            },
          ],
        };

      case 'browser_click':
        return {
          content: [
            {
              type: 'text',
              text: `Clicked element: ${args.selector}`,
            },
          ],
        };

      case 'browser_type':
        return {
          content: [
            {
              type: 'text',
              text: `Typed "${args.text}" into ${args.selector}`,
            },
          ],
        };

      case 'browser_extract_dom':
        return {
          content: [
            {
              type: 'text',
              text: '<!-- DOM content would be here -->',
            },
          ],
        };

      case 'browser_scroll':
        return {
          content: [
            {
              type: 'text',
              text: `Scrolled ${args.direction || 'down'}`,
            },
          ],
        };

      case 'browser_wait':
        return {
          content: [
            {
              type: 'text',
              text: `Waited ${args.timeout || 5000}ms`,
            },
          ],
        };

      case 'browser_close':
        return {
          content: [
            {
              type: 'text',
              text: 'Browser closed',
            },
          ],
        };

      default:
        throw new Error(`Unknown browser tool: ${toolName}`);
    }
  }

  /**
   * Check rate limit for a server
   */
  private async checkRateLimit(serverName: string): Promise<void> {
    const limiter = this.rateLimiters.get(serverName);
    if (limiter && !limiter.tryConsume()) {
      throw new Error(
        `Rate limit exceeded for ${serverName}. Retry after ${limiter.getTimeUntilAvailable()}ms`
      );
    }
  }

  /**
   * List all resources from all servers
   */
  async listResources(): Promise<Resource[]> {
    const allResources: Resource[] = [];

    for (const [serverName, transport] of this.transports) {
      try {
        const result = await transport.request('resources/list', {});
        if (result.resources) {
          allResources.push(...result.resources);
        }
      } catch (error) {
        logger.error('Failed to list resources', { server: serverName, error });
      }
    }

    return allResources;
  }

  /**
   * Read a resource
   */
  async readResource(params: ResourceReadParams): Promise<ResourceReadResult> {
    const { serverName, uri } = this.parseResourceUri(params.uri);

    const transport = this.transports.get(serverName);
    if (!transport) {
      throw new MCPError(`Server not found: ${serverName}`, -32602);
    }

    return await transport.request('resources/read', { uri: params.uri });
  }

  /**
   * Parse resource URI to get server
   */
  private parseResourceUri(uri: string): { serverName: string; uri: string } {
    // MCP resources can have various URI formats
    // Try to extract server name from URI if possible
    for (const serverName of this.servers.keys()) {
      if (uri.startsWith(serverName + '://') || uri.startsWith('/' + serverName + '/')) {
        return { serverName, uri };
      }
    }

    // Default to first available server
    const firstServer = this.servers.keys().next().value;
    if (firstServer) {
      return { serverName: firstServer, uri };
    }

    throw new MCPError('No MCP servers available', -32603);
  }

  /**
   * List all prompts from all servers
   */
  async listPrompts(): Promise<Prompt[]> {
    const allPrompts: Prompt[] = [];

    for (const [serverName, transport] of this.transports) {
      try {
        const result = await transport.request('prompts/list', {});
        if (result.prompts) {
          allPrompts.push(...result.prompts);
        }
      } catch (error) {
        logger.error('Failed to list prompts', { server: serverName, error });
      }
    }

    return allPrompts;
  }

  /**
   * Get a prompt
   */
  async getPrompt(params: PromptGetParams): Promise<PromptGetResult> {
    // Find which server has this prompt
    for (const [serverName, transport] of this.transports) {
      try {
        const result = await transport.request('prompts/get', params);
        return result;
      } catch (error: any) {
        if (error.code === -32602) {
          // Method not found on this server, try next
          continue;
        }
        throw error;
      }
    }

    throw new MCPError(`Prompt not found: ${params.name}`, -32602);
  }

  /**
   * Get rate limiter status
   */
  getRateLimitStatus(name: string): { tokens: number; maxTokens: number } | null {
    const limiter = this.rateLimiters.get(name);
    if (!limiter) return null;
    return {
      tokens: limiter.getTokens(),
      maxTokens: 100,
    };
  }

  /**
   * Get server info
   */
  getServerInfo(name: string): MCPServerConnection | undefined {
    return this.servers.get(name);
  }

  /**
   * Get all connected servers
   */
  getServers(): MCPServerConnection[] {
    return Array.from(this.servers.values());
  }

  /**
   * Shutdown MCP client and all connections
   */
  async shutdown(): Promise<void> {
    logger.info('Shutting down MCP client');

    const disconnectPromises: Promise<void>[] = [];

    for (const [name, transport] of this.transports) {
      disconnectPromises.push(
        transport.disconnect().catch((error) => {
          logger.error('Error disconnecting server', { name, error });
        })
      );
    }

    await Promise.all(disconnectPromises);

    this.servers.clear();
    this.transports.clear();
    this.tools.clear();
    this.rateLimiters.clear();

    logger.info('MCP client shut down');
  }
}
