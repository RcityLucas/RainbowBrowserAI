// Simple test setup without complex dependencies
import 'dotenv/config';

// Simple custom matchers
expect.extend({
  toBeValidApiResponse(received) {
    const pass = received &&
                 typeof received === 'object' &&
                 typeof received.success === 'boolean';
    
    if (pass) {
      return {
        message: () => `expected ${JSON.stringify(received)} not to be a valid API response`,
        pass: true,
      };
    } else {
      return {
        message: () => `expected ${JSON.stringify(received)} to be a valid API response with success field`,
        pass: false,
      };
    }
  },
  
  toHaveValidSelector(received) {
    const pass = received && 
                 typeof received === 'string' && 
                 received.trim().length > 0 &&
                 !received.includes('>>');
    
    if (pass) {
      return {
        message: () => `expected ${received} not to be a valid CSS selector`,
        pass: true,
      };
    } else {
      return {
        message: () => `expected ${received} to be a valid CSS selector`,
        pass: false,
      };
    }
  }
});

// Declare custom matchers
declare global {
  namespace jest {
    interface Matchers<R> {
      toBeValidApiResponse(): R;
      toHaveValidSelector(): R;
    }
  }
}