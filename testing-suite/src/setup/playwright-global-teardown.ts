import { FullConfig } from '@playwright/test';

async function globalTeardown(config: FullConfig) {
  console.log('🧹 Starting Playwright global teardown...');
  
  const testServer = (global as any).__TEST_SERVER__;
  
  if (testServer) {
    try {
      await testServer.stop();
      console.log('✅ Test server stopped successfully');
    } catch (error) {
      console.error('⚠️ Error stopping test server:', error);
    }
  }
  
  console.log('✅ Playwright global teardown completed');
}

export default globalTeardown;