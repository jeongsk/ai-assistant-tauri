/**
 * Agent Types for v0.6
 *
 * Types for AI agent multimodal processing, context management, and orchestration.
 */

// ============================================================================
// Multimodal Types
// ============================================================================

export enum ImageFormat {
  Png = 'png',
  Jpeg = 'jpeg',
  Gif = 'gif',
  WebP = 'webp',
  Bmp = 'bmp',
}

export interface ImageData {
  data: number[];
  format: ImageFormat;
  width?: number;
  height?: number;
}

export type InputType =
  | { type: 'text'; text: string }
  | { type: 'image'; data: number[]; format: ImageFormat }
  | { type: 'mixed'; text: string; images: ImageData[] };

export interface ObjectDetection {
  label: string;
  confidence: number;
  boundingBox?: BoundingBox;
}

export interface BoundingBox {
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface ImageAnalysis {
  caption: string;
  objects: ObjectDetection[];
  extractedText?: string;
  tags: string[];
  confidence: number;
}

export type MultimodalResult =
  | { type: 'text'; text: string }
  | { type: 'image'; analysis: ImageAnalysis }
  | { type: 'mixed'; text: string; imageAnalyses: ImageAnalysis[] };

// ============================================================================
// Context Types
// ============================================================================

export enum MessageRole {
  System = 'system',
  User = 'user',
  Assistant = 'assistant',
  Tool = 'tool',
}

export enum MessagePriority {
  Low = 'low',
  Normal = 'normal',
  High = 'high',
  Critical = 'critical',
}

export interface Message {
  role: MessageRole;
  content: string;
  tokenCount: number;
  priority: MessagePriority;
  timestamp: number;
}

export enum CompressionStrategy {
  RemoveOldest = 'remove_oldest',
  Summarize = 'summarize',
  PriorityOnly = 'priority_only',
  Hybrid = 'hybrid',
}

export interface CompressorConfig {
  strategy: CompressionStrategy;
  minTokens: number;
  targetRatio: number;
}

export interface CompressionResult {
  originalTokens: number;
  compressedTokens: number;
  removedCount: number;
  summarizedCount: number;
}

// ============================================================================
// Orchestrator Types
// ============================================================================

export enum AgentType {
  General = 'general',
  CodeGenerator = 'code_generator',
  CodeReviewer = 'code_reviewer',
  Researcher = 'researcher',
  DataAnalyst = 'data_analyst',
  FileOperator = 'file_operator',
  WebScraper = 'web_scraper',
  Custom = 'custom',
}

export enum TaskPriority {
  Low = 'low',
  Normal = 'normal',
  High = 'high',
  Urgent = 'urgent',
}

export interface TaskInput {
  description: string;
  data: unknown;
  priority: TaskPriority;
  timeoutSeconds?: number;
}

export interface SubAgentTask {
  id: string;
  agentType: AgentType;
  input: TaskInput;
  dependencies: string[];
  createdAt: number;
}

export interface TaskResult {
  taskId: string;
  success: boolean;
  data: unknown;
  error?: string;
  executionTimeMs: number;
}

export interface AggregatedResult {
  totalTasks: number;
  successful: number;
  failed: number;
  results: Record<string, TaskResult>;
  combinedData: unknown;
}
