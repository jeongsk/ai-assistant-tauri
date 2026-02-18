/**
 * Browser MCP Configuration
 */

export interface BrowserMCPConfig {
  serverName: 'browser';
  command: 'npx';
  args: string[];
  env?: Record<string, string>;
}

export type BrowserToolName =
  | 'browser_navigate'
  | 'browser_screenshot'
  | 'browser_click'
  | 'browser_type'
  | 'browser_extract_dom'
  | 'browser_scroll'
  | 'browser_wait'
  | 'browser_close';

export interface BrowserTool {
  name: BrowserToolName;
  description: string;
  inputSchema: {
    type: 'object';
    properties: Record<string, unknown>;
    required?: string[];
  };
}

// Browser tools configuration
export const BROWSER_TOOLS: BrowserTool[] = [
  {
    name: 'browser_navigate',
    description: 'Navigate to a URL',
    inputSchema: {
      type: 'object',
      properties: {
        url: {
          type: 'string',
          description: 'The URL to navigate to',
        },
      },
      required: ['url'],
    },
  },
  {
    name: 'browser_screenshot',
    description: 'Take a screenshot of the current page',
    inputSchema: {
      type: 'object',
      properties: {
        fullPage: {
          type: 'boolean',
          description: 'Whether to capture the full page',
        },
        selector: {
          type: 'string',
          description: 'Optional CSS selector to capture specific element',
        },
      },
    },
  },
  {
    name: 'browser_click',
    description: 'Click on an element',
    inputSchema: {
      type: 'object',
      properties: {
        selector: {
          type: 'string',
          description: 'CSS selector for the element to click',
        },
      },
      required: ['selector'],
    },
  },
  {
    name: 'browser_type',
    description: 'Type text into an input field',
    inputSchema: {
      type: 'object',
      properties: {
        selector: {
          type: 'string',
          description: 'CSS selector for the input field',
        },
        text: {
          type: 'string',
          description: 'Text to type',
        },
      },
      required: ['selector', 'text'],
    },
  },
  {
    name: 'browser_extract_dom',
    description: 'Extract DOM content from the page',
    inputSchema: {
      type: 'object',
      properties: {
        selector: {
          type: 'string',
          description: 'Optional CSS selector to extract specific content',
        },
        maxBytes: {
          type: 'number',
          description: 'Maximum bytes to extract (default: 1MB)',
        },
      },
    },
  },
  {
    name: 'browser_scroll',
    description: 'Scroll the page',
    inputSchema: {
      type: 'object',
      properties: {
        direction: {
          type: 'string',
          enum: ['up', 'down'],
          description: 'Scroll direction',
        },
        amount: {
          type: 'number',
          description: 'Number of pixels to scroll',
        },
      },
    },
  },
  {
    name: 'browser_wait',
    description: 'Wait for an element or condition',
    inputSchema: {
      type: 'object',
      properties: {
        selector: {
          type: 'string',
          description: 'CSS selector to wait for',
        },
        timeout: {
          type: 'number',
          description: 'Timeout in milliseconds',
        },
      },
    },
  },
  {
    name: 'browser_close',
    description: 'Close the browser',
    inputSchema: {
      type: 'object',
      properties: {},
    },
  },
];

// Browser timeouts
export const BROWSER_TIMEOUTS = {
  navigation: 30000, // 30s
  click: 10000, // 10s
  type: 10000, // 10s
  screenshot: 5000, // 5s
  extract: 10000, // 10s
};

// Browser configuration
export interface BrowserConfig {
  enabled: boolean;
  headless: boolean;
  browserType: 'chromium' | 'firefox' | 'webkit';
  downloadOnFirstUse: boolean;
}

export const DEFAULT_BROWSER_CONFIG: BrowserConfig = {
  enabled: false,
  headless: true,
  browserType: 'chromium',
  downloadOnFirstUse: true,
};

/**
 * Get MCP server config for browser
 */
export function getBrowserMCPConfig(): BrowserMCPConfig {
  return {
    serverName: 'browser',
    command: 'npx',
    args: ['@playwright/mcp@latest'],
  };
}
