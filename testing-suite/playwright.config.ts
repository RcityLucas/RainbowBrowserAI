import { defineConfig, devices } from '@playwright/test';

export default defineConfig({
  testDir: './src/__tests__/e2e',
  
  /* Run tests in files in parallel */
  fullyParallel: true,
  
  /* Fail the build on CI if you accidentally left test.only in the source code. */
  forbidOnly: !!process.env.CI,
  
  /* Retry on CI only */
  retries: process.env.CI ? 2 : 0,
  
  /* Opt out of parallel tests on CI. */
  workers: process.env.CI ? 1 : undefined,
  
  /* Reporter to use. See https://playwright.dev/docs/test-reporters */
  reporter: [
    ['html', { outputFolder: 'reports/playwright-report' }],
    ['junit', { outputFile: 'reports/e2e-results.xml' }],
    ['json', { outputFile: 'reports/e2e-results.json' }]
  ],
  
  /* Shared settings for all the projects below. */
  use: {
    /* Base URL to use in actions like `await page.goto('/')`. */
    baseURL: process.env.TEST_SERVER_URL || 'http://localhost:3002',
    
    /* Collect trace when retrying the failed test. */
    trace: 'on-first-retry',
    
    /* Take screenshot on failure */
    screenshot: 'only-on-failure',
    
    /* Record video on failure */
    video: 'retain-on-failure',
    
    /* Global timeout for each action */
    actionTimeout: 10000,
    
    /* Global timeout for navigation */
    navigationTimeout: 30000,
  },

  /* Global timeout for each test */
  timeout: 60000,

  /* Global setup and teardown */
  globalSetup: require.resolve('./src/setup/playwright-global-setup'),
  globalTeardown: require.resolve('./src/setup/playwright-global-teardown'),

  /* Configure projects for major browsers */
  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },

    {
      name: 'firefox',
      use: { ...devices['Desktop Firefox'] },
    },

    {
      name: 'webkit',
      use: { ...devices['Desktop Safari'] },
    },

    /* Test against mobile viewports. */
    {
      name: 'Mobile Chrome',
      use: { ...devices['Pixel 5'] },
    },
    {
      name: 'Mobile Safari',
      use: { ...devices['iPhone 12'] },
    },

    /* Test against branded browsers. */
    {
      name: 'Microsoft Edge',
      use: { ...devices['Desktop Edge'], channel: 'msedge' },
    },
    {
      name: 'Google Chrome',
      use: { ...devices['Desktop Chrome'], channel: 'chrome' },
    },
  ],

  /* Run your local dev server before starting the tests */
  webServer: {
    command: 'npm run start:test-server',
    url: 'http://localhost:3002/api/health',
    reuseExistingServer: !process.env.CI,
    timeout: 120000,
  },

  /* Output directory for test artifacts */
  outputDir: 'test-results/',
});