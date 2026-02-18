/**
 * Rate Limiter - Token bucket implementation
 */

export class RateLimiter {
  private tokens: number;
  private lastRefill: number;
  private readonly maxTokens: number;
  private readonly refillRate: number; // tokens per ms

  constructor(maxTokens: number, windowMs: number) {
    this.maxTokens = maxTokens;
    this.tokens = maxTokens;
    this.lastRefill = Date.now();
    this.refillRate = maxTokens / windowMs;
  }

  /**
   * Try to consume tokens
   * @returns true if tokens were consumed, false if rate limit exceeded
   */
  tryConsume(count: number = 1): boolean {
    this.refill();

    if (this.tokens >= count) {
      this.tokens -= count;
      return true;
    }

    return false;
  }

  /**
   * Get current token count
   */
  getTokens(): number {
    this.refill();
    return Math.floor(this.tokens);
  }

  /**
   * Get time until next token is available (in ms)
   */
  getTimeUntilAvailable(): number {
    if (this.tokens >= 1) return 0;
    return Math.ceil((1 - this.tokens) / this.refillRate);
  }

  /**
   * Reset the rate limiter
   */
  reset(): void {
    this.tokens = this.maxTokens;
    this.lastRefill = Date.now();
  }

  private refill(): void {
    const now = Date.now();
    const elapsed = now - this.lastRefill;
    this.tokens = Math.min(
      this.maxTokens,
      this.tokens + elapsed * this.refillRate
    );
    this.lastRefill = now;
  }
}

/**
 * Rate limit error
 */
export class RateLimitError extends Error {
  constructor(
    message: string,
    public readonly retryAfter: number
  ) {
    super(message);
    this.name = 'RateLimitError';
  }
}

/**
 * Browser rate limiter configuration
 */
export const BROWSER_RATE_LIMIT = {
  maxActions: 100,
  windowMs: 60000, // 1 minute
};

/**
 * Create a rate limiter for browser actions
 */
export function createBrowserRateLimiter(): RateLimiter {
  return new RateLimiter(BROWSER_RATE_LIMIT.maxActions, BROWSER_RATE_LIMIT.windowMs);
}
