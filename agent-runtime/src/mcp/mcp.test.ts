/**
 * MCP Client Tests
 */

import { describe, it, expect, beforeEach, afterEach } from '@jest/globals';
import { MCPClient } from './client.js';
import { MCPStdioTransport } from './stdio.js';
import type { MCPServerConfig, Tool } from './types.js';

describe('MCPStdioTransport', () => {
  describe('constructor', () => {
    it('should create a transport with default options', () => {
      const config: MCPServerConfig = {
        name: 'test-server',
        command: 'echo',
      };

      const transport = new MCPStdioTransport(config);

      expect(transport.isConnected()).toBe(false);
    });

    it('should create a transport with custom options', () => {
      const config: MCPServerConfig = {
        name: 'test-server',
        command: 'echo',
      };

      const transport = new MCPStdioTransport(config, {
        timeout: 60000,
        env: { TEST_VAR: 'test' },
      });

      expect(transport.isConnected()).toBe(false);
    });
  });

  describe('connection', () => {
    let transport: MCPStdioTransport;
    const echoConfig: MCPServerConfig = {
      name: 'echo-server',
      command: 'node',
      args: ['-e', 'require("readline").createInterface({input:process.stdin,output:process.stdout,terminal:false}).on("line",(line)=>{if(line.trim()){try{const msg=JSON.parse(line);console.log(JSON.stringify({jsonrpc:"2.0",id:msg.id,result:{}}))}catch{}}})'],
    };

    beforeEach(() => {
      transport = new MCPStdioTransport(echoConfig, { timeout: 5000 });
    });

    afterEach(async () => {
      if (transport.isConnected()) {
        await transport.disconnect();
      }
    });

    it('should connect to an echo server', async () => {
      await transport.connect();

      expect(transport.isConnected()).toBe(true);
      expect(transport.getPid()).toBeGreaterThan(0);
    });

    it('should disconnect from server', async () => {
      await transport.connect();
      expect(transport.isConnected()).toBe(true);

      await transport.disconnect();

      expect(transport.isConnected()).toBe(false);
    });

    it('should handle connection errors gracefully', async () => {
      const badConfig: MCPServerConfig = {
        name: 'bad-server',
        command: 'nonexistent-command-that-does-not-exist',
      };

      const badTransport = new MCPStdioTransport(badConfig, { timeout: 5000 });

      // The error event will be emitted, even if the promise resolves
      // This is because spawn ENOENT fires asynchronously after our delay
      const errorPromise = new Promise<void>((resolve) => {
        badTransport.once('error', () => resolve());
      });

      // Try to connect - may succeed or fail depending on timing
      try {
        await badTransport.connect();
      } catch {
        // Connection failed as expected
      }

      // Wait for error event
      await errorPromise;

      // Transport should not be connected
      expect(badTransport.isConnected()).toBe(false);
    });
  });

  describe('requests', () => {
    let transport: MCPStdioTransport;

    // Simple MCP mock server
    const mockServerConfig: MCPServerConfig = {
      name: 'mock-server',
      command: 'node',
      args: ['-e', `
        const rl = require('readline').createInterface({
          input: process.stdin,
          output: process.stdout,
          terminal: false
        });

        rl.on('line', (line) => {
          if (!line.trim()) return;

          try {
            const msg = JSON.parse(line);

            if (msg.method === 'initialize') {
              console.log(JSON.stringify({
                jsonrpc: '2.0',
                id: msg.id,
                result: {
                  protocolVersion: '2024-11-05',
                  capabilities: { tools: {} },
                  serverInfo: { name: 'mock-server', version: '1.0.0' }
                }
              }));
            } else if (msg.method === 'tools/list') {
              console.log(JSON.stringify({
                jsonrpc: '2.0',
                id: msg.id,
                result: {
                  tools: [
                    {
                      name: 'test_tool',
                      description: 'A test tool',
                      inputSchema: { type: 'object', properties: {} }
                    }
                  ]
                }
              }));
            } else if (msg.method === 'shutdown') {
              console.log(JSON.stringify({
                jsonrpc: '2.0',
                id: msg.id,
                result: {}
              }));
              process.exit(0);
            }
          } catch (e) {
            console.error('Error:', e.message);
          }
        });
      `],
    };

    beforeEach(async () => {
      transport = new MCPStdioTransport(mockServerConfig, { timeout: 5000 });
      await transport.connect();
    });

    afterEach(async () => {
      if (transport.isConnected()) {
        await transport.disconnect();
      }
    });

    it('should send initialize request', async () => {
      const result = await transport.request('initialize', {
        protocolVersion: '2024-11-05',
        capabilities: {},
        clientInfo: { name: 'test-client', version: '1.0.0' },
      });

      expect(result).toBeDefined();
      expect(result.protocolVersion).toBe('2024-11-05');
      expect(result.serverInfo.name).toBe('mock-server');
    });

    it('should list tools', async () => {
      const result = await transport.request('tools/list', {});

      expect(result).toBeDefined();
      expect(result.tools).toBeInstanceOf(Array);
      expect(result.tools.length).toBeGreaterThan(0);
    });
  });
});

describe('MCPClient', () => {
  describe('constructor', () => {
    it('should create a client with default options', () => {
      const client = new MCPClient();

      expect(client.getServers()).toHaveLength(0);
    });

    it('should create a client with custom options', () => {
      const client = new MCPClient({
        requestTimeout: 60000,
        enableBrowserTools: false,
      });

      expect(client.getServers()).toHaveLength(0);
    });
  });

  describe('browser tools', () => {
    it('should register browser tools when enabled', async () => {
      const client = new MCPClient({ enableBrowserTools: true });

      await client.initialize([]);

      const tools = await client.listTools();

      // Should have browser tools
      expect(tools.length).toBeGreaterThan(0);
      expect(tools.some((t: Tool) => t.name.startsWith('browser_'))).toBe(true);

      await client.shutdown();
    });

    it('should not register browser tools when disabled', async () => {
      const client = new MCPClient({ enableBrowserTools: false });

      await client.initialize([]);

      const tools = await client.listTools();

      // Should not have browser tools
      expect(tools.filter((t: Tool) => t.name.startsWith('browser_')).length).toBe(0);

      await client.shutdown();
    });

    it('should call browser tools', async () => {
      const client = new MCPClient({ enableBrowserTools: true });

      await client.initialize([]);

      const result = await client.callTool('browser_navigate', { url: 'https://example.com' });

      expect(result).toBeDefined();
      expect(result.content).toBeInstanceOf(Array);

      await client.shutdown();
    });
  });

  describe('rate limiting', () => {
    it('should track rate limit status for browser', async () => {
      const client = new MCPClient({ enableBrowserTools: true });

      await client.initialize([]);

      const status = client.getRateLimitStatus('browser');

      expect(status).toBeDefined();
      expect(status?.tokens).toBeGreaterThan(0);

      await client.shutdown();
    });
  });

  describe('server connection', () => {
    const mockServerConfig: MCPServerConfig = {
      name: 'test-mcp-server',
      command: 'node',
      args: ['-e', `
        const rl = require('readline').createInterface({
          input: process.stdin,
          output: process.stdout,
          terminal: false
        });

        rl.on('line', (line) => {
          if (!line.trim()) return;

          try {
            const msg = JSON.parse(line);

            if (msg.method === 'initialize') {
              console.log(JSON.stringify({
                jsonrpc: '2.0',
                id: msg.id,
                result: {
                  protocolVersion: '2024-11-05',
                  capabilities: { tools: {} },
                  serverInfo: { name: 'test-server', version: '1.0.0' }
                }
              }));
            } else if (msg.method === 'tools/list') {
              console.log(JSON.stringify({
                jsonrpc: '2.0',
                id: msg.id,
                result: { tools: [] }
              }));
            } else if (msg.method === 'shutdown') {
              console.log(JSON.stringify({
                jsonrpc: '2.0',
                id: msg.id,
                result: {}
              }));
              process.exit(0);
            }
          } catch (e) {}
        });
      `],
    };

    it('should connect to an MCP server', async () => {
      const client = new MCPClient({ enableBrowserTools: false });

      await client.initialize([mockServerConfig]);

      const servers = client.getServers();
      expect(servers.length).toBe(1);
      expect(servers[0].config.name).toBe('test-mcp-server');

      await client.shutdown();
    });

    it('should handle connection failures gracefully', async () => {
      const badConfig: MCPServerConfig = {
        name: 'bad-server',
        command: 'nonexistent-command',
      };

      const client = new MCPClient({ enableBrowserTools: false });

      // Should not throw, should continue with other servers
      await client.initialize([badConfig]);

      // Should have no connected servers
      expect(client.getServers().length).toBe(0);

      await client.shutdown();
    });
  });
});
