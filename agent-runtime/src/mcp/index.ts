/**
 * MCP (Model Context Protocol) Module
 *
 * Main export point for MCP functionality.
 */

export { MCPClient, type MCPClientOptions } from './client.js';
export { MCPStdioTransport } from './stdio.js';

export type {
  // Core types
  MCPServerConfig,
  MCPServerConnection,
  MCPServerInfo,
  MCPStdio,

  // Protocol types
  JSONRPCRequest,
  JSONRPCResponse,
  JSONRPCError,
  JSONRPCNotification,
  MCPMethod,

  // Initialize
  InitializeParams,
  InitializeResult,
  ClientCapabilities,
  ServerCapabilities,
  ClientInfo,
  ServerInfo,

  // Tools
  Tool,
  ToolInputSchema,
  ToolProperty,
  ToolsListParams,
  ToolsListResult,
  ToolCallParams,
  ToolCallResult,
  ToolContent,
  TextContent,
  ImageContent,
  ResourceContent,

  // Resources
  Resource,
  ResourceTemplate,
  ResourcesListParams,
  ResourcesListResult,
  ResourceReadParams,
  ResourceReadResult,
  ResourceContents,
  TextResourceContents,
  BlobResourceContents,

  // Prompts
  Prompt,
  PromptArgument,
  PromptsListParams,
  PromptsListResult,
  PromptGetParams,
  PromptGetResult,
  PromptMessage,

  // Completion
  CompleteParams,
  CompleteResult,
  CompletionRef,
  CompletionArgument,
  CompletionCompletion,
  CompletionValue,

  // Logging
  LoggingLevel,
  SetLevelParams,

  // Notifications
  Notification,
  InitializedNotification,
  CancelledNotification,
  ProgressNotification,
  LogLevelNotification,
  MessageNotification,

  // Errors
  MCPError,
  MCPConnectionError,
  MCPTimeoutError,
} from './types.js';

export interface StdioTransportOptions {
  timeout?: number;
  maxBufferSize?: number;
  env?: Record<string, string>;
}

export * from './browser.js';
