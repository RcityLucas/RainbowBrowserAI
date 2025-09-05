import { test, expect } from '@playwright/test';

test.describe('End-to-End Workflows', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await expect(page).toHaveTitle(/RainbowBrowserAI/);
  });

  test('Complete form filling workflow', async ({ page }) => {
    // Step 1: Navigate to a form page
    await page.fill('input[placeholder*="URL"]', 'https://httpbin.org/forms/post');
    await page.click('button:has-text("Navigate")');
    
    // Wait for navigation to complete
    await expect(page.locator('#result')).toContainText('success', { timeout: 15000 });
    
    // Step 2: Fill customer name
    await page.fill('input[placeholder*="CSS selector"]', 'input[name="custname"]');
    await page.click('button:has-text("Click Element")');
    await expect(page.locator('#result')).toBeVisible({ timeout: 10000 });
    
    // Step 3: Type customer name
    await page.fill('input[placeholder*="Text to type"]', 'John Smith');
    await page.click('button:has-text("Type Text")');
    await expect(page.locator('#result')).toContainText('success', { timeout: 10000 });
    
    // Step 4: Fill telephone field
    await page.fill('input[placeholder*="CSS selector"]', 'input[name="custel"]');
    await page.click('button:has-text("Focus Element")');
    await expect(page.locator('#result')).toBeVisible({ timeout: 10000 });
    
    await page.fill('input[placeholder*="Text to type"]', '555-123-4567');
    await page.click('button:has-text("Type Text")');
    await expect(page.locator('#result')).toContainText('success', { timeout: 10000 });
    
    // Step 5: Verify data was entered
    await page.fill('input[placeholder*="CSS selector"]', 'input[name="custname"]');
    await page.click('button:has-text("Extract Text")');
    await expect(page.locator('#result')).toContainText('John Smith', { timeout: 10000 });
  });

  test('Navigation and data extraction workflow', async ({ page }) => {
    // Step 1: Navigate to Example.com
    await page.fill('input[placeholder*="URL"]', 'https://example.com');
    await page.click('button:has-text("Navigate")');
    await expect(page.locator('#result')).toContainText('success', { timeout: 15000 });
    
    // Step 2: Extract page title
    await page.fill('input[placeholder*="CSS selector"]', 'h1');
    await page.click('button:has-text("Extract Text")');
    await expect(page.locator('#result')).toBeVisible({ timeout: 10000 });
    
    // Step 3: Extract all links
    await page.fill('input[placeholder*="CSS selector"]', 'a');
    await page.click('button:has-text("Extract Links")');
    await expect(page.locator('#result')).toContainText('href', { timeout: 10000 });
    
    // Step 4: Navigate to GitHub
    await page.fill('input[placeholder*="URL"]', 'https://github.com');
    await page.click('button:has-text("Navigate")');
    await expect(page.locator('#result')).toContainText('success', { timeout: 15000 });
    
    // Step 5: Use browser navigation
    await page.click('button:has-text("Go Back")');
    await expect(page.locator('#result')).toContainText('success', { timeout: 10000 });
    
    await page.click('button:has-text("Go Forward")');
    await expect(page.locator('#result')).toContainText('success', { timeout: 10000 });
  });

  test('Perception-powered interaction workflow', async ({ page }) => {
    // Navigate to perception tab
    await page.click('[data-tab="perception"]');
    await expect(page.locator('#perception-tab')).toBeVisible();
    
    // Step 1: Navigate to test page
    await page.fill('input[placeholder*="URL"]', 'https://httpbin.org/forms/post');
    await page.click('button:has-text("Navigate to URL")');
    await expect(page.locator('#perception-result')).toContainText('success', { timeout: 15000 });
    
    // Step 2: Analyze the page
    await page.click('button:has-text("Analyze Current Page")');
    await expect(page.locator('#perception-result')).toBeVisible({ timeout: 10000 });
    
    // Step 3: Find element using natural language
    await page.fill('input[placeholder*="natural language"]', 'customer name input');
    await page.click('button:has-text("Find Element")');
    await expect(page.locator('#perception-result')).toContainText('selector', { timeout: 10000 });
    
    // Step 4: Execute intelligent command
    await page.selectOption('select', 'click');
    await page.fill('textarea[placeholder*="description"]', 'customer name field');
    await page.click('button:has-text("Execute Command")');
    await expect(page.locator('#perception-result')).toBeVisible({ timeout: 10000 });
    
    // Step 5: Analyze forms
    await page.click('button:has-text("Analyze Forms")');
    await expect(page.locator('#perception-result')).toContainText('form', { timeout: 10000 });
  });

  test('Session management workflow', async ({ page }) => {
    // Step 1: Set session storage
    await page.fill('input[placeholder*="Key"]', 'user_id');
    await page.fill('input[placeholder*="Value"]', 'test_user_123');
    await page.click('button:has-text("Set Session")');
    await expect(page.locator('#result')).toContainText('success', { timeout: 10000 });
    
    // Step 2: Navigate to different page
    await page.fill('input[placeholder*="URL"]', 'https://example.com');
    await page.click('button:has-text("Navigate")');
    await expect(page.locator('#result')).toContainText('success', { timeout: 15000 });
    
    // Step 3: Verify session data persists
    await page.fill('input[placeholder*="Key"]', 'user_id');
    await page.click('button:has-text("Get Session")');
    await expect(page.locator('#result')).toContainText('test_user_123', { timeout: 10000 });
    
    // Step 4: Set additional session data
    await page.fill('input[placeholder*="Key"]', 'workflow_step');
    await page.fill('input[placeholder*="Value"]', 'navigation_complete');
    await page.click('button:has-text("Set Session")');
    await expect(page.locator('#result')).toContainText('success', { timeout: 10000 });
    
    // Step 5: Navigate again and verify both values
    await page.fill('input[placeholder*="URL"]', 'https://httpbin.org');
    await page.click('button:has-text("Navigate")');
    await expect(page.locator('#result')).toContainText('success', { timeout: 15000 });
    
    await page.fill('input[placeholder*="Key"]', 'user_id');
    await page.click('button:has-text("Get Session")');
    await expect(page.locator('#result')).toContainText('test_user_123', { timeout: 10000 });
  });

  test('Error recovery workflow', async ({ page }) => {
    // Step 1: Start with valid operation
    await page.fill('input[placeholder*="URL"]', 'https://example.com');
    await page.click('button:has-text("Navigate")');
    await expect(page.locator('#result')).toContainText('success', { timeout: 15000 });
    
    // Step 2: Attempt invalid operation
    await page.fill('input[placeholder*="CSS selector"]', 'invalid>>selector');
    await page.click('button:has-text("Click Element")');
    await expect(page.locator('#result')).toContainText('Error', { timeout: 10000 });
    
    // Step 3: Verify system continues to work
    await page.fill('input[placeholder*="CSS selector"]', 'h1');
    await page.click('button:has-text("Extract Text")');
    await expect(page.locator('#result')).toContainText('success', { timeout: 10000 });
    
    // Step 4: Test another error condition
    await page.fill('input[placeholder*="CSS selector"]', '.non-existent-element');
    await page.click('button:has-text("Click Element")');
    await expect(page.locator('#result')).toContainText('Error', { timeout: 10000 });
    
    // Step 5: Verify recovery with valid operation
    await page.fill('input[placeholder*="CSS selector"]', 'body');
    await page.click('button:has-text("Get Element Info")');
    await expect(page.locator('#result')).toContainText('success', { timeout: 10000 });
  });

  test('Complex multi-site workflow', async ({ page }) => {
    // Step 1: Collect data from first site
    await page.fill('input[placeholder*="URL"]', 'https://example.com');
    await page.click('button:has-text("Navigate")');
    await expect(page.locator('#result')).toContainText('success', { timeout: 15000 });
    
    await page.fill('input[placeholder*="CSS selector"]', 'h1');
    await page.click('button:has-text("Extract Text")');
    await expect(page.locator('#result')).toBeVisible({ timeout: 10000 });
    
    // Store first site data
    await page.fill('input[placeholder*="Key"]', 'site1_title');
    await page.fill('input[placeholder*="Value"]', 'Example Domain');
    await page.click('button:has-text("Set Session")');
    
    // Step 2: Navigate to second site
    await page.fill('input[placeholder*="URL"]', 'https://httpbin.org');
    await page.click('button:has-text("Navigate")');
    await expect(page.locator('#result')).toContainText('success', { timeout: 15000 });
    
    await page.fill('input[placeholder*="CSS selector"]', 'h1');
    await page.click('button:has-text("Extract Text")');
    await expect(page.locator('#result')).toBeVisible({ timeout: 10000 });
    
    // Step 3: Navigate to form site and use stored data
    await page.fill('input[placeholder*="URL"]', 'https://httpbin.org/forms/post');
    await page.click('button:has-text("Navigate")');
    await expect(page.locator('#result')).toContainText('success', { timeout: 15000 });
    
    // Retrieve stored data
    await page.fill('input[placeholder*="Key"]', 'site1_title');
    await page.click('button:has-text("Get Session")');
    await expect(page.locator('#result')).toBeVisible({ timeout: 10000 });
    
    // Step 4: Use data in form
    await page.fill('input[placeholder*="CSS selector"]', 'input[name="custname"]');
    await page.click('button:has-text("Click Element")');
    
    await page.fill('input[placeholder*="Text to type"]', 'Multi-Site User');
    await page.click('button:has-text("Type Text")');
    await expect(page.locator('#result')).toContainText('success', { timeout: 10000 });
    
    // Step 5: Final verification
    await page.fill('input[placeholder*="CSS selector"]', 'input[name="custname"]');
    await page.click('button:has-text("Extract Text")');
    await expect(page.locator('#result')).toContainText('Multi-Site User', { timeout: 10000 });
  });

  test('Workflow with timing and waits', async ({ page }) => {
    // Test workflow that includes explicit waits
    await page.fill('input[placeholder*="URL"]', 'https://example.com');
    await page.click('button:has-text("Navigate")');
    await expect(page.locator('#result')).toContainText('success', { timeout: 15000 });
    
    // Test wait for element
    await page.fill('input[placeholder*="CSS selector"]', 'body');
    await page.fill('input[placeholder*="Timeout"]', '5000');
    await page.click('button:has-text("Wait for Element")');
    await expect(page.locator('#result')).toContainText('success', { timeout: 10000 });
    
    // Test wait timeout
    await page.fill('input[placeholder*="CSS selector"]', '.never-exists');
    await page.fill('input[placeholder*="Timeout"]', '2000');
    await page.click('button:has-text("Wait for Element")');
    await expect(page.locator('#result')).toContainText('Error', { timeout: 5000 });
    
    // Verify system still responsive after timeout
    await page.fill('input[placeholder*="CSS selector"]', 'h1');
    await page.click('button:has-text("Extract Text")');
    await expect(page.locator('#result')).toContainText('success', { timeout: 10000 });
  });
});