/**
 * MCP (Model Context Protocol) Type Definitions
 *
 * Based on the Model Context Protocol specification for stdio transport.
 */

// JSON-RPC 2.0 base types
export interface JSONRPCRequest {
  jsonrpc: '2.0';
  id: number | string;
  method: string;
  params?: any;
}

export interface JSONRPCResponse {
  jsonrpc: '2.0';
  id: number | string;
  result?: any;
  error?: JSONRPCError;
}

export interface JSONRPCError {
  code: number;
  message: string;
  data?: any;
}

// MCP Protocol Methods
export type MCPMethod =
  | 'initialize'
  | 'initialized'
  | 'shutdown'
  | 'tools/list'
  | 'tools/call'
  | 'resources/list'
  | 'resources/read'
  | 'resources/templates/list'
  | 'prompts/list'
  | 'prompts/get'
  | 'prompts/complete'
  | 'completion/complete';

// Initialize
export interface InitializeParams {
  protocolVersion: string;
  capabilities: ClientCapabilities;
  clientInfo: ClientInfo;
}

export interface ClientCapabilities {
  roots?: boolean;
  sampling?: boolean;
  experimental?: Record<string, any>;
}

export interface ClientInfo {
  name: string;
  version: string;
}

export interface InitializeResult {
  protocolVersion: string;
  capabilities: ServerCapabilities;
  serverInfo: ServerInfo;
}

export interface ServerCapabilities {
  tools?: {};
  resources?: {};
  prompts?: {};
  logging?: {};
  experimental?: Record<string, any>;
}

export interface ServerInfo {
  name: string;
  version: string;
}

// Tools
export interface Tool {
  name: string;
  description?: string;
  inputSchema: ToolInputSchema;
}

export interface ToolInputSchema {
  type: 'object';
  properties?: Record<string, ToolProperty>;
  required?: string[];
  additionalProperties?: boolean;
}

export interface ToolProperty {
  type?: string;
  description?: string;
  enum?: string[];
  items?: ToolProperty;
  properties?: Record<string, ToolProperty>;
  required?: string[];
  [key: string]: any;
}

export interface ToolsListParams {
  cursor?: string;
}

export interface ToolsListResult {
  tools: Tool[];
  nextCursor?: string;
}

export interface ToolCallParams {
  name: string;
  arguments?: Record<string, any>;
}

export interface ToolCallResult {
  content: ToolContent[];
  isError?: boolean;
}

export type ToolContent =
  | TextContent
  | ImageContent
  | ResourceContent;

export interface TextContent {
  type: 'text';
  text: string;
}

export interface ImageContent {
  type: 'image';
  data: string;
  mimeType: string;
}

export interface ResourceContent {
  type: 'resource';
  uri: string;
}

// Resources
export interface Resource {
  uri: string;
  name: string;
  description?: string;
  mimeType?: string;
}

export interface ResourceTemplate {
  uriTemplate: string;
  name: string;
  description?: string;
  mimeType?: string;
}

export interface ResourcesListParams {
  cursor?: string;
}

export interface ResourcesListResult {
  resources: Resource[];
  nextCursor?: string;
}

export interface ResourceReadParams {
  uri: string;
}

export interface ResourceReadResult {
  contents: ResourceContents[];
}

export type ResourceContents =
  | TextResourceContents
  | BlobResourceContents;

export interface TextResourceContents {
  uri: string;
  mimeType?: string;
  text: string;
}

export interface BlobResourceContents {
  uri: string;
  mimeType?: string;
  blob: string; // base64 encoded
}

// Prompts
export interface Prompt {
  name: string;
  description?: string;
  arguments?: PromptArgument[];
}

export interface PromptArgument {
  name: string;
  description?: string;
  required?: boolean;
}

export interface PromptsListParams {
  cursor?: string;
}

export interface PromptsListResult {
  prompts: Prompt[];
  nextCursor?: string;
}

export interface PromptGetParams {
  name: string;
  arguments?: Record<string, string>;
}

export interface PromptGetResult {
  description?: string;
  messages: PromptMessage[];
}

export interface PromptMessage {
  role: 'user' | 'assistant';
  content:
    | TextContent
    | ImageContent
    | ResourceContent;
}

// Completion
export interface CompleteParams {
  ref: CompletionRef;
  argument: CompletionArgument;
}

export interface CompletionRef {
  type: 'ref/resource';
  uri: string;
}

export interface CompletionArgument {
  name: string;
  value: string;
}

export interface CompleteResult {
  completion: CompletionCompletion;
}

export interface CompletionCompletion {
  values: CompletionValue[];
  total?: number;
  hasMore?: boolean;
}

export interface CompletionValue {
  value: string;
  description?: string;
}

// Logging
export type LoggingLevel = 'debug' | 'info' | 'notice' | 'warning' | 'error' | 'critical' | 'alert' | 'emergency';

export interface SetLevelParams {
  level: LoggingLevel;
}

// Server connection state
export interface MCPServerConnection {
  config: MCPServerConfig;
  process: any | null;
  stdio: MCPStdio | null;
  ready: boolean;
  capabilities: ServerCapabilities | null;
  tools: Tool[];
  requestId: number;
}

export interface MCPServerConfig {
  name: string;
  command: string;
  args?: string[];
  env?: Record<string, string>;
  cwd?: string;
}

export interface MCPServerInfo {
  name: string;
  version: string;
  protocolVersion: string;
  capabilities: ServerCapabilities;
}

// Stdio transport
export interface MCPStdio {
  stdin: NodeJS.WritableStream;
  stdout: NodeJS.ReadableStream;
  stderr: NodeJS.ReadableStream;
}

// Error types
export class MCPError extends Error {
  constructor(
    message: string,
    public readonly code: number,
    public readonly data?: any
  ) {
    super(message);
    this.name = 'MCPError';
  }
}

export class MCPConnectionError extends MCPError {
  constructor(message: string, public readonly cause?: Error) {
    super(message, -32603);
    this.name = 'MCPConnectionError';
  }
}

export class MCPTimeoutError extends MCPError {
  constructor(message: string, public readonly timeout: number) {
    super(message, -32603);
    this.name = 'MCPTimeoutError';
  }
}

// Notification types (no response expected)
export interface JSONRPCNotification {
  jsonrpc: '2.0';
  method: string;
  params?: any;
}

export type Notification =
  | InitializedNotification
  | CancelledNotification
  | ProgressNotification
  | LogLevelNotification
  | MessageNotification;

export interface InitializedNotification extends JSONRPCNotification {
  method: 'notifications/initialized';
}

export interface CancelledNotification extends JSONRPCNotification {
  method: 'notifications/cancelled';
  params: {
    requestId: string | number;
    reason?: string;
  };
}

export interface ProgressNotification extends JSONRPCNotification {
  method: 'notifications/progress';
  params: {
    progressToken: string | number;
    progress: number;
    total?: number;
  };
}

export interface LogLevelNotification extends JSONRPCNotification {
  method: 'notifications/set_level';
  params: SetLevelParams;
}

export interface MessageNotification extends JSONRPCNotification {
  method: 'notifications/message';
  params: {
    level: LoggingLevel;
    logger: string;
    data: string;
  };
}
