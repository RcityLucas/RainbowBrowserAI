// Utility functions for testing
export class TestUtils {
  /**
   * Validates if a response object is a valid API response
   */
  static isValidApiResponse(response: any): boolean {
    if (!response || typeof response !== 'object') {
      return false;
    }
    return typeof response.success === 'boolean';
  }

  /**
   * Validates if a string is a valid CSS selector
   */
  static isValidSelector(selector: string): boolean {
    if (!selector || typeof selector !== 'string') {
      return false;
    }
    
    const trimmed = selector.trim();
    return trimmed.length > 0 && !trimmed.includes('>>');
  }

  /**
   * Generates a mock API response
   */
  static createMockResponse(success: boolean, data?: any, error?: string) {
    return {
      success,
      data: success ? data : null,
      error: success ? null : error,
      timestamp: new Date().toISOString()
    };
  }

  /**
   * Waits for a specified amount of time
   */
  static async delay(ms: number): Promise<void> {
    return new Promise(resolve => setTimeout(resolve, ms));
  }

  /**
   * Generates test data for form testing
   */
  static generateFormData(fields: string[]): Record<string, string> {
    const data: Record<string, string> = {};
    
    fields.forEach((field, index) => {
      switch (field) {
        case 'name':
          data[field] = `Test User ${index + 1}`;
          break;
        case 'email':
          data[field] = `test${index + 1}@example.com`;
          break;
        case 'phone':
          data[field] = `555-${String(1000 + index).padStart(4, '0')}`;
          break;
        default:
          data[field] = `test-value-${index + 1}`;
      }
    });
    
    return data;
  }

  /**
   * Calculates test execution time
   */
  static measureExecutionTime<T>(fn: () => T | Promise<T>): Promise<{ result: T; time: number }> {
    const start = performance.now();
    
    const execute = async () => {
      const result = await fn();
      const end = performance.now();
      return { result, time: end - start };
    };

    return execute();
  }

  /**
   * Creates a test selector for different scenarios
   */
  static createTestSelector(type: 'valid' | 'invalid' | 'complex'): string {
    switch (type) {
      case 'valid':
        return 'input[name="test"]';
      case 'invalid':
        return 'invalid>>selector';
      case 'complex':
        return 'form .field-group input[type="text"]:not(.disabled)';
      default:
        throw new Error(`Unknown selector type: ${type}`);
    }
  }

  /**
   * Validates test environment
   */
  static validateTestEnvironment(): { valid: boolean; issues: string[] } {
    const issues: string[] = [];
    
    if (typeof performance === 'undefined') {
      issues.push('Performance API not available');
    }
    
    if (typeof Promise === 'undefined') {
      issues.push('Promise not available');
    }
    
    if (typeof setTimeout === 'undefined') {
      issues.push('setTimeout not available');
    }
    
    return {
      valid: issues.length === 0,
      issues
    };
  }
}