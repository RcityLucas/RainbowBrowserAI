import { FullConfig } from '@playwright/test';
import { TestServer } from '../utils/test-server';

async function globalSetup(config: FullConfig) {
  console.log('🚀 Starting Playwright global setup...');
  
  // Start the test server if not already running
  const testServer = new TestServer(3002);
  
  try {
    if (!(await testServer.isHealthy())) {
      console.log('Starting test server for E2E tests...');
      await testServer.start();
      console.log('✅ Test server started successfully');
    } else {
      console.log('✅ Test server already running');
    }
  } catch (error) {
    console.error('❌ Failed to start test server:', error);
    throw error;
  }
  
  // Store server reference for cleanup
  (global as any).__TEST_SERVER__ = testServer;
  
  console.log('✅ Playwright global setup completed');
}

export default globalSetup;