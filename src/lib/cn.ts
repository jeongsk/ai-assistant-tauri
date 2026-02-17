import { clsx, type ClassValue } from 'clsx';

/**
 * Conditional class name utility
 * Combines clsx for conditional classes
 */
export function cn(...inputs: ClassValue[]): string {
  return clsx(inputs);
}
