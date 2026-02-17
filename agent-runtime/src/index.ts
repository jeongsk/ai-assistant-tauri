/**
 * Agent Runtime Entry Point
 * 
 * Communicates with Tauri Rust core via stdio JSON-RPC
 */

import { createInterface } from 'readline';
import { AgentCore } from './agent/core.js';
import { MCPClient } from './mcp/client.js';
import { ProviderRouter } from './providers/router.js';
import { MemoryManager } from './memory/manager.js';
import { logger } from './utils/logger.js';

// Initialize components
const agentCore = new AgentCore();
const mcpClient = new MCPClient();
const providerRouter = new ProviderRouter();
const memoryManager = new MemoryManager();

// JSON-RPC interface
const rl = createInterface({
  input: process.stdin,
  output: process.stdout,
  terminal: false,
});

// Handle incoming JSON-RPC messages
rl.on('line', async (line: string) => {
  try {
    const message = JSON.parse(line);
    await handleRequest(message);
  } catch (error) {
    logger.error('Parse error', error);
    sendError('Parse error', -32700);
  }
});

// Route requests to handlers
async function handleRequest(message: any) {
  const { method, params, id } = message;

  logger.debug(`Received: ${method}`);

  try {
    let result: any;

    switch (method) {
      case 'initialize':
        result = await handleInitialize(params);
        break;
      
      case 'chat':
        result = await handleChat(params);
        break;
      
      case 'tool_call':
        result = await handleToolCall(params);
        break;
      
      case 'get_tools':
        result = await handleGetTools(params);
        break;
      
      case 'shutdown':
        result = await handleShutdown();
        sendResponse(result, id);
        process.exit(0);
        break;
      
      default:
        sendError('Method not found', -32601, id);
        return;
    }

    sendResponse(result, id);
  } catch (error: any) {
    logger.error(`Error in ${method}`, error);
    sendError(error.message || 'Internal error', -32603, id);
  }
}

// Initialize agent with config
async function handleInitialize(params: any) {
  const { config } = params;
  
  // Initialize provider
  await providerRouter.initialize(config.providers, config.activeProvider);
  
  // Initialize MCP servers
  await mcpClient.initialize(config.mcpServers || []);
  
  // Initialize memory
  await memoryManager.initialize(config.memoryPath);
  
  logger.info('Agent initialized');
  
  return { status: 'initialized' };
}

// Handle chat request
async function handleChat(params: any) {
  const { messages, options } = params;
  
  // Get active provider
  const provider = providerRouter.getActiveProvider();
  
  // Add context from memory
  const contextMessages = await memoryManager.getContext();
  const fullMessages = [...contextMessages, ...messages];
  
  // Call LLM
  const response = await provider.chat(fullMessages, options);
  
  // Update memory
  await memoryManager.addMessages(messages, response);
  
  return response;
}

// Handle tool call
async function handleToolCall(params: any) {
  const { tool, args } = params;
  
  // Execute via MCP
  const result = await mcpClient.callTool(tool, args);
  
  return result;
}

// Get available tools
async function handleGetTools(params: any) {
  const tools = await mcpClient.listTools();
  return { tools };
}

// Shutdown
async function handleShutdown() {
  await mcpClient.shutdown();
  await memoryManager.close();
  return { status: 'shutdown' };
}

// Send JSON-RPC response
function sendResponse(result: any, id?: string) {
  const response = {
    jsonrpc: '2.0',
    result,
    id,
  };
  console.log(JSON.stringify(response));
}

// Send JSON-RPC error
function sendError(message: string, code: number, id?: string) {
  const response = {
    jsonrpc: '2.0',
    error: { code, message },
    id,
  };
  console.log(JSON.stringify(response));
}

// Handle process lifecycle
process.on('SIGTERM', async () => {
  logger.info('Received SIGTERM, shutting down...');
  await handleShutdown();
  process.exit(0);
});

process.on('SIGINT', async () => {
  logger.info('Received SIGINT, shutting down...');
  await handleShutdown();
  process.exit(0);
});

// Signal ready
logger.info('Agent runtime started');
sendResponse({ status: 'ready' }, 'init');
