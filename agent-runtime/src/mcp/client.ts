/**
 * MCP Client - Model Context Protocol client with enhanced functionality
 */

import { logger } from '../utils/logger.js';
import { RateLimiter, createBrowserRateLimiter } from '../utils/rate-limiter.js';
import { BROWSER_TOOLS, getBrowserMCPConfig } from './browser.js';

export interface MCPServerConfig {
  name: string;
  command: string;
  args?: string[];
  env?: Record<string, string>;
}

export interface Tool {
  name: string;
  description: string;
  inputSchema: any;
}

interface MCPServerConnection {
  config: MCPServerConfig;
  process: any | null;
  ready: boolean;
}

export class MCPClient {
  private servers: Map<string, MCPServerConnection> = new Map();
  private tools: Map<string, Tool> = new Map();
  private rateLimiters: Map<string, RateLimiter> = new Map();

  async initialize(configs: MCPServerConfig[]): Promise<void> {
    logger.info('Initializing MCP client', { serverCount: configs.length });

    // Register built-in browser tools
    this.registerBrowserTools();

    for (const config of configs) {
      await this.registerServer(config);
    }
  }

  /**
   * Register browser tools (built-in)
   */
  private registerBrowserTools(): void {
    for (const tool of BROWSER_TOOLS) {
      this.tools.set(tool.name, tool);
    }
    this.rateLimiters.set('browser', createBrowserRateLimiter());
    logger.info('Registered browser tools', { count: BROWSER_TOOLS.length });
  }

  /**
   * Register an MCP server
   */
  private async registerServer(config: MCPServerConfig): Promise<void> {
    const connection: MCPServerConnection = {
      config,
      process: null,
      ready: false,
    };

    this.servers.set(config.name, connection);
    logger.debug('Registered MCP server', { name: config.name });

    // Note: In a full implementation, we would spawn the MCP server process
    // and communicate via stdio. For now, we register the server config.
  }

  /**
   * List all available tools
   */
  async listTools(): Promise<Tool[]> {
    return Array.from(this.tools.values());
  }

  /**
   * Call a tool with arguments
   */
  async callTool(name: string, args: any): Promise<any> {
    logger.debug('Calling tool', { name, args });

    // Check if tool exists
    const tool = this.tools.get(name);
    if (!tool) {
      throw new Error(`Tool not found: ${name}`);
    }

    // Check rate limit for browser tools
    if (name.startsWith('browser_')) {
      const limiter = this.rateLimiters.get('browser');
      if (limiter && !limiter.tryConsume()) {
        throw new Error(
          `Rate limit exceeded for browser tools. Retry after ${limiter.getTimeUntilAvailable()}ms`
        );
      }
    }

    // Parse tool name to get server and actual tool
    const [serverName, toolName] = this.parseToolName(name);

    // For browser tools, return mock response for now
    if (serverName === 'browser') {
      return this.handleBrowserTool(toolName, args);
    }

    // Route to appropriate MCP server
    const connection = this.servers.get(serverName);
    if (!connection) {
      throw new Error(`MCP server not found: ${serverName}`);
    }

    // TODO: Implement actual MCP protocol communication
    throw new Error(`Tool execution not yet implemented for: ${name}`);
  }

  /**
   * Parse tool name to get server and tool
   */
  private parseToolName(name: string): [string, string] {
    const parts = name.split('_');
    if (parts.length >= 2) {
      return [parts[0], name];
    }
    return ['unknown', name];
  }

  /**
   * Handle browser tool calls (mock implementation)
   */
  private handleBrowserTool(toolName: string, args: any): any {
    logger.info('Handling browser tool', { toolName, args });

    switch (toolName) {
      case 'browser_navigate':
        return { status: 'success', url: args.url };

      case 'browser_screenshot':
        return {
          status: 'success',
          format: 'png',
          // In real implementation, would return base64 image
          message: 'Screenshot captured',
        };

      case 'browser_click':
        return {
          status: 'success',
          selector: args.selector,
          clicked: true,
        };

      case 'browser_type':
        return {
          status: 'success',
          selector: args.selector,
          typed: args.text,
        };

      case 'browser_extract_dom':
        return {
          status: 'success',
          content: '<!-- DOM content would be here -->',
          truncated: false,
        };

      case 'browser_scroll':
        return {
          status: 'success',
          direction: args.direction,
        };

      case 'browser_wait':
        return {
          status: 'success',
          waited: true,
        };

      case 'browser_close':
        return { status: 'success', message: 'Browser closed' };

      default:
        throw new Error(`Unknown browser tool: ${toolName}`);
    }
  }

  /**
   * Get rate limiter status
   */
  getRateLimitStatus(name: string): { tokens: number; maxTokens: number } | null {
    const limiter = this.rateLimiters.get(name);
    if (!limiter) return null;
    return {
      tokens: limiter.getTokens(),
      maxTokens: 100, // From BROWSER_RATE_LIMIT
    };
  }

  /**
   * Shutdown MCP client
   */
  async shutdown(): Promise<void> {
    logger.info('Shutting down MCP client');

    // Stop all servers
    for (const [name, connection] of this.servers) {
      if (connection.process) {
        // TODO: Kill process
        logger.debug('Stopping MCP server', { name });
      }
    }

    this.servers.clear();
    this.tools.clear();
    this.rateLimiters.clear();
  }
}
