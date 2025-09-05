import dotenv from 'dotenv';
import { TestServer } from '../utils/test-server';

// Load environment variables
dotenv.config();

// Global test configuration
global.TEST_CONFIG = {
  serverUrl: process.env.TEST_SERVER_URL || 'http://localhost:3002',
  timeout: parseInt(process.env.TEST_TIMEOUT || '30000'),
  retries: parseInt(process.env.TEST_RETRIES || '3'),
  headless: process.env.HEADLESS !== 'false',
  mockMode: process.env.RAINBOW_MOCK_MODE === 'true'
};

// Global test server instance
let testServer: TestServer;

// Setup before all tests
beforeAll(async () => {
  console.log('🚀 Starting test server...');
  testServer = new TestServer();
  await testServer.start();
  console.log('✅ Test server started successfully');
}, 60000);

// Cleanup after all tests
afterAll(async () => {
  console.log('🧹 Cleaning up test server...');
  if (testServer) {
    await testServer.stop();
  }
  console.log('✅ Test server stopped');
}, 30000);

// Global error handler for unhandled rejections
process.on('unhandledRejection', (reason, promise) => {
  console.error('Unhandled Rejection at:', promise, 'reason:', reason);
});

// Global error handler for uncaught exceptions
process.on('uncaughtException', (error) => {
  console.error('Uncaught Exception:', error);
  process.exit(1);
});

// Extend Jest matchers
expect.extend({
  toBeValidApiResponse(received) {
    const pass = received &&
                 typeof received === 'object' &&
                 typeof received.success === 'boolean';
    
    if (pass) {
      return {
        message: () => `expected ${received} not to be a valid API response`,
        pass: true,
      };
    } else {
      return {
        message: () => `expected ${received} to be a valid API response with success field`,
        pass: false,
      };
    }
  },
  
  toHaveValidSelector(received) {
    const pass = received && 
                 typeof received === 'string' && 
                 received.length > 0 &&
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

// Declare global types
declare global {
  const TEST_CONFIG: {
    serverUrl: string;
    timeout: number;
    retries: number;
    headless: boolean;
    mockMode: boolean;
  };
  
  namespace jest {
    interface Matchers<R> {
      toBeValidApiResponse(): R;
      toHaveValidSelector(): R;
    }
  }
}