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
  BaseProvider,
} from "./providers/base.js";

// Simple logger
const logger = {
  info: (msg: string) => console.error(`[INFO] ${msg}`),
  error: (msg: string, err?: any) => console.error(`[ERROR] ${msg}`, err || ""),
  debug: (msg: string) => console.error(`[DEBUG] ${msg}`),
};

// Provider storage
let providers: Map<string, BaseProvider> = new Map();
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

      case "execute_skill":
        result = await handleExecuteSkill(params);
        break;

      case "execute_recipe":
        result = await handleExecuteRecipe(params);
        break;

      case "execute_prompt":
        result = await handleExecutePrompt(params);
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
function getActiveProvider(): BaseProvider {
  if (!activeProvider) {
    throw new Error("No active provider configured");
  }

  const provider = providers.get(activeProvider);
  if (!provider) {
    throw new Error(`Active provider '${activeProvider}' not found`);
  }

  return provider;
}

// Export provider accessor for use by AgentCore
export { getActiveProvider };

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

// Execute a skill
async function handleExecuteSkill(params: any) {
  const { skillId, input, variables } = params;

  logger.info(`Executing skill: ${skillId}`);

  try {
    const provider = getActiveProvider();

    // Build skill-specific prompt
    const skillPrompt = `You are executing the skill "${skillId}".

${input || 'No specific input provided.'}

${variables ? `Variables: ${JSON.stringify(variables)}` : ''}

Execute this skill and provide the result.`;

    const response: ChatResponse = await provider.chat([
      { role: 'user', content: skillPrompt },
    ]);

    return {
      success: true,
      result: response.content,
      metadata: {
        skillId,
        timestamp: new Date().toISOString(),
        usage: response.usage,
      },
    };
  } catch (error: any) {
    logger.error(`Skill execution failed: ${skillId}`, error);
    return {
      success: false,
      error: error.message || "Skill execution failed",
    };
  }
}

// Execute a recipe
async function handleExecuteRecipe(params: any) {
  const { recipeId, variables } = params;

  logger.info(`Executing recipe: ${recipeId}`);

  // For now, return a placeholder response
  // In a full implementation, this would:
  // 1. Load the recipe from the database
  // 2. Execute each step using the RecipeEngine
  // 3. Return the aggregated results

  return {
    success: true,
    result: `Recipe '${recipeId}' executed with variables: ${JSON.stringify(variables || {})}`,
    metadata: {
      recipeId,
      timestamp: new Date().toISOString(),
      steps: 0,
    },
  };
}

// Execute a prompt
async function handleExecutePrompt(params: any) {
  const { prompt, provider: providerOverride } = params;

  logger.info("Executing prompt job");

  try {
    // Use specified provider or active provider
    const providerToUse = providerOverride
      ? providers.get(providerOverride)
      : getActiveProvider();

    if (!providerToUse) {
      throw new Error(`Provider not found: ${providerOverride || activeProvider}`);
    }

    const response: ChatResponse = await providerToUse.chat([
      { role: 'user', content: prompt },
    ]);

    return {
      success: true,
      result: response.content,
      metadata: {
        timestamp: new Date().toISOString(),
        usage: response.usage,
      },
    };
  } catch (error: any) {
    logger.error("Prompt execution failed", error);
    return {
      success: false,
      error: error.message || "Prompt execution failed",
    };
  }
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
