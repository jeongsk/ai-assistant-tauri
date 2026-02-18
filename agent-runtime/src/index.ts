/**
 * Agent Runtime Entry Point
 *
 * Communicates with Tauri Rust core via stdio JSON-RPC
 */

import { createInterface } from "readline";
import { OpenAIProvider } from "./providers/openai.js";
import { AnthropicProvider } from "./providers/anthropic.js";
import { OllamaProvider } from "./providers/ollama.js";
import type {
  Message,
  ChatOptions,
  ChatResponse,
  ProviderConfig,
} from "./providers/base.js";

// Simple logger
const logger = {
  info: (msg: string) => console.error(`[INFO] ${msg}`),
  error: (msg: string, err?: any) => console.error(`[ERROR] ${msg}`, err || ""),
  debug: (msg: string) => console.error(`[DEBUG] ${msg}`),
};

// Provider storage
let providers: Map<string, any> = new Map();
let activeProvider: string | null = null;

// JSON-RPC interface
const rl = createInterface({
  input: process.stdin,
  output: process.stdout,
  terminal: false,
});

// Handle incoming JSON-RPC messages
rl.on("line", async (line: string) => {
  try {
    const message = JSON.parse(line.trim());
    await handleRequest(message);
  } catch (error) {
    logger.error("Parse error", error);
    sendError("Parse error", -32700);
  }
});

// Route requests to handlers
async function handleRequest(message: any) {
  const { method, params, id } = message;

  logger.debug(`Received: ${method}`);

  try {
    let result: any;

    switch (method) {
      case "initialize":
        result = await handleInitialize(params);
        break;

      case "chat":
        result = await handleChat(params);
        break;

      case "tool_call":
        result = await handleToolCall(params);
        break;

      case "get_tools":
        result = await handleGetTools(params);
        break;

      case "configure_providers":
        result = await handleConfigureProviders(params);
        break;

      case "shutdown":
        result = await handleShutdown();
        sendResponse(result, id);
        process.exit(0);
        break;

      default:
        sendError("Method not found", -32601, id);
        return;
    }

    sendResponse(result, id);
  } catch (error: any) {
    logger.error(`Error in ${method}`, error);
    sendError(error.message || "Internal error", -32603, id);
  }
}

// Initialize agent with config
async function handleInitialize(params: any) {
  const { config } = params;

  // Initialize providers from config
  if (config?.providers) {
    for (const providerConfig of config.providers) {
      if (!providerConfig.enabled) continue;

      try {
        const provider = createProvider(providerConfig);
        providers.set(providerConfig.type, provider);
        logger.info(`Initialized provider: ${providerConfig.type}`);
      } catch (error) {
        logger.error(`Failed to initialize ${providerConfig.type}`, error);
      }
    }
  }

  // Set active provider
  if (config?.activeProvider) {
    activeProvider = config.activeProvider;
  }

  logger.info("Agent initialized");
  return { status: "initialized" };
}

// Configure providers
async function handleConfigureProviders(params: any) {
  const { providers: providerConfigs, activeProvider: active } = params;

  providers.clear();

  for (const providerConfig of providerConfigs) {
    if (!providerConfig.enabled) continue;

    try {
      const provider = createProvider(providerConfig);
      providers.set(providerConfig.type, provider);
      logger.info(`Configured provider: ${providerConfig.type}`);
    } catch (error) {
      logger.error(`Failed to configure ${providerConfig.type}`, error);
    }
  }

  if (active) {
    activeProvider = active;
  }

  return { status: "configured" };
}

// Create provider instance
function createProvider(config: ProviderConfig) {
  switch (config.type) {
    case "openai":
      return new OpenAIProvider(config);
    case "anthropic":
      return new AnthropicProvider(config);
    case "ollama":
      return new OllamaProvider(config);
    default:
      throw new Error(`Unknown provider type: ${config.type}`);
  }
}

// Get active provider
function getActiveProvider() {
  if (!activeProvider) {
    throw new Error("No active provider configured");
  }

  const provider = providers.get(activeProvider);
  if (!provider) {
    throw new Error(`Active provider '${activeProvider}' not found`);
  }

  return provider;
}

// Handle chat request
async function handleChat(params: any) {
  const { messages, options } = params;

  const provider = getActiveProvider();
  const response: ChatResponse = await provider.chat(messages, options);

  return {
    content: response.content,
    metadata: {
      provider: activeProvider,
      timestamp: new Date().toISOString(),
      usage: response.usage,
    },
  };
}

// Handle tool call
async function handleToolCall(params: any) {
  const { tool, args } = params;

  // Simple tool execution simulation
  return {
    result: `Tool '${tool}' called with args: ${JSON.stringify(args)}`,
  };
}

// Get available tools
async function handleGetTools(params: any) {
  return {
    tools: [
      {
        name: "read_file",
        description: "Read file content from disk",
        inputSchema: {
          type: "object",
          properties: {
            path: { type: "string", description: "File path to read" },
          },
          required: ["path"],
        },
      },
      {
        name: "write_file",
        description: "Write content to a file",
        inputSchema: {
          type: "object",
          properties: {
            path: { type: "string", description: "File path to write" },
            content: { type: "string", description: "Content to write" },
          },
          required: ["path", "content"],
        },
      },
      {
        name: "list_directory",
        description: "List files in a directory",
        inputSchema: {
          type: "object",
          properties: {
            path: { type: "string", description: "Directory path to list" },
          },
          required: ["path"],
        },
      },
    ],
  };
}

// Shutdown
async function handleShutdown() {
  logger.info("Shutting down...");
  return { status: "shutdown" };
}

// Send JSON-RPC response
function sendResponse(result: any, id?: string) {
  const response = {
    jsonrpc: "2.0",
    result,
    id,
  };
  console.log(JSON.stringify(response));
}

// Send JSON-RPC error
function sendError(message: string, code: number, id?: string) {
  const response = {
    jsonrpc: "2.0",
    error: { code, message },
    id,
  };
  console.log(JSON.stringify(response));
}

// Handle process lifecycle
process.on("SIGTERM", async () => {
  logger.info("Received SIGTERM, shutting down...");
  await handleShutdown();
  process.exit(0);
});

process.on("SIGINT", async () => {
  logger.info("Received SIGINT, shutting down...");
  await handleShutdown();
  process.exit(0);
});

// Signal ready
logger.info("Agent runtime started");
sendResponse({ status: "ready" }, "init");
