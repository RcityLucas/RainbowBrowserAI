import { test, expect } from '@playwright/test';

test.describe('User Interface', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await expect(page).toHaveTitle(/RainbowBrowserAI/);
  });

  test('should display main interface elements', async ({ page }) => {
    // Check main title
    await expect(page.locator('h1')).toContainText('RainbowBrowserAI Dashboard');
    
    // Check edition branding
    await expect(page.locator('.edition-badge')).toContainText('Chromiumoxide Edition');
    
    // Check navigation tabs
    const navTabs = page.locator('.nav-item[data-tab]');
    await expect(navTabs).toHaveCount(6); // command, browse, workflow, sessions, perception, settings
    
    // Verify tab names
    await expect(page.locator('[data-tab="command"]')).toBeVisible();
    await expect(page.locator('[data-tab="browse"]')).toBeVisible();
    await expect(page.locator('[data-tab="workflow"]')).toBeVisible();
    await expect(page.locator('[data-tab="sessions"]')).toBeVisible();
    await expect(page.locator('[data-tab="perception"]')).toBeVisible();
    await expect(page.locator('[data-tab="settings"]')).toBeVisible();
  });

  test('should have functional navigation tabs', async ({ page }) => {
    // Test navigation to each tab
    await page.click('[data-tab="perception"]');
    await expect(page.locator('#perception-tab')).toBeVisible();
    
    await page.click('[data-tab="browse"]');
    await expect(page.locator('#browse-tab')).toBeVisible();
    
    await page.click('[data-tab="workflow"]');
    await expect(page.locator('#workflow-tab')).toBeVisible();
    
    await page.click('[data-tab="sessions"]');
    await expect(page.locator('#sessions-tab')).toBeVisible();
    
    await page.click('[data-tab="settings"]');
    await expect(page.locator('#settings-tab')).toBeVisible();
    
    // Return to command tab
    await page.click('[data-tab="command"]');
    await expect(page.locator('#command-tab')).toBeVisible();
  });

  test('should display tool categories', async ({ page }) => {
    // Check tool categories are present
    await expect(page.locator('h3')).toContainText('Navigation Tools');
    await expect(page.locator('h3')).toContainText('Interaction Tools');
    await expect(page.locator('h3')).toContainText('Data Extraction Tools');
    await expect(page.locator('h3')).toContainText('Utility & Wait Tools');
    
    // Check interaction tools count
    await expect(page.locator('h3')).toContainText('Interaction Tools (5)');
  });

  test('should have working tool input forms', async ({ page }) => {
    // Test navigation tool form
    await page.fill('input[placeholder*="URL"]', 'https://example.com');
    await page.click('button:has-text("Navigate")');
    
    // Wait for some feedback (success or error message)
    await expect(page.locator('#result')).toBeVisible({ timeout: 10000 });
    
    // Test click tool form
    await page.fill('input[placeholder*="selector"]', 'h1');
    await page.click('button:has-text("Click Element")');
    
    await expect(page.locator('#result')).toBeVisible({ timeout: 10000 });
  });

  test('should display help system', async ({ page }) => {
    // Check help sections are present
    await expect(page.locator('details summary')).toContainText('Common Element Selectors Guide');
    
    // Expand help section
    await page.click('details summary');
    await expect(page.locator('.help-content')).toBeVisible();
    
    // Check specific help items
    await expect(page.locator('.help-section h4')).toContainText('GitHub.com');
    await expect(page.locator('.help-section h4')).toContainText('CSS Syntax');
  });

  test('should show perception interface', async ({ page }) => {
    await page.click('[data-tab="perception"]');
    
    // Check perception sections
    await expect(page.locator('h3')).toContainText('Page Analysis');
    await expect(page.locator('h3')).toContainText('Smart Element Detection');
    await expect(page.locator('h3')).toContainText('Intelligent Commands');
    await expect(page.locator('h3')).toContainText('Smart Form Analysis');
    
    // Test page analysis
    await page.click('button:has-text("Analyze Current Page")');
    await expect(page.locator('#perception-result')).toBeVisible({ timeout: 10000 });
  });

  test('should handle form interactions', async ({ page }) => {
    // Test various input types
    const inputs = page.locator('input[type="text"], input:not([type]), textarea');
    const inputCount = await inputs.count();
    expect(inputCount).toBeGreaterThan(10);
    
    // Test button interactions
    const buttons = page.locator('button[onclick]');
    const buttonCount = await buttons.count();
    expect(buttonCount).toBeGreaterThan(15);
    
    // Test specific form fields
    await page.fill('input[placeholder*="URL"]', 'test-url');
    await expect(page.locator('input[placeholder*="URL"]')).toHaveValue('test-url');
    
    await page.fill('input[placeholder*="selector"]', 'test-selector');
    await expect(page.locator('input[placeholder*="selector"]')).toHaveValue('test-selector');
  });

  test('should display error messages appropriately', async ({ page }) => {
    // Trigger an error by using invalid selector
    await page.fill('input[placeholder*="selector"]', 'invalid>>selector');
    await page.click('button:has-text("Click Element")');
    
    // Check for error display
    const result = page.locator('#result');
    await expect(result).toBeVisible({ timeout: 10000 });
    
    // Should contain error information
    await expect(result).toContainText('Error');
  });

  test('should be responsive', async ({ page }) => {
    // Test mobile viewport
    await page.setViewportSize({ width: 375, height: 667 });
    
    // Check that main elements are still visible
    await expect(page.locator('h1')).toBeVisible();
    await expect(page.locator('.nav-item')).toBeVisible();
    
    // Test tablet viewport
    await page.setViewportSize({ width: 768, height: 1024 });
    await expect(page.locator('h1')).toBeVisible();
    
    // Return to desktop
    await page.setViewportSize({ width: 1920, height: 1080 });
    await expect(page.locator('h1')).toBeVisible();
  });

  test('should load assets correctly', async ({ page }) => {
    // Check that CSS is loaded
    const backgroundColor = await page.locator('body').evaluate(
      el => window.getComputedStyle(el).backgroundColor
    );
    expect(backgroundColor).not.toBe('rgba(0, 0, 0, 0)'); // Should have some styling
    
    // Check JavaScript functionality
    await page.click('[data-tab="perception"]');
    await expect(page.locator('#perception-tab')).toBeVisible();
    
    // Verify dynamic content works
    await page.click('button:has-text("Analyze Current Page")');
    await expect(page.locator('#perception-result')).toBeVisible({ timeout: 10000 });
  });

  test('should handle concurrent interactions', async ({ page }) => {
    // Open multiple tabs quickly
    await Promise.all([
      page.click('[data-tab="perception"]'),
      page.click('[data-tab="browse"]'),
      page.click('[data-tab="workflow"]')
    ]);
    
    // The last one should be visible
    await expect(page.locator('#workflow-tab')).toBeVisible();
    
    // Test multiple form interactions
    await page.click('[data-tab="command"]');
    await Promise.all([
      page.fill('input[placeholder*="URL"]', 'https://example.com'),
      page.fill('input[placeholder*="selector"]:first', 'h1')
    ]);
    
    await expect(page.locator('input[placeholder*="URL"]')).toHaveValue('https://example.com');
    await expect(page.locator('input[placeholder*="selector"]:first')).toHaveValue('h1');
  });
});