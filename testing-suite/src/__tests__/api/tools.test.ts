import { ApiClient } from '../../utils/api-client';

describe('Tools API', () => {
  let apiClient: ApiClient;

  beforeAll(() => {
    apiClient = new ApiClient(global.TEST_CONFIG.serverUrl);
  });

  beforeEach(async () => {
    // Ensure we have a clean state for each test
    await apiClient.navigate('https://example.com');
  });

  describe('GET /api/tools', () => {
    it('should return list of available tools', async () => {
      const response = await apiClient.getTools();
      
      expect(response).toBeDefined();
      expect(response.tools).toBeInstanceOf(Array);
      expect(response.tools.length).toBeGreaterThan(0);
    });

    it('should include required tools', async () => {
      const response = await apiClient.getTools();
      const toolNames = response.tools.map(tool => tool.name);
      
      const requiredTools = [
        'navigate_to_url',
        'click',
        'type_text',
        'extract_text',
        'extract_links'
      ];

      requiredTools.forEach(toolName => {
        expect(toolNames).toContain(toolName);
      });
    });
  });

  describe('Navigation Tools', () => {
    it('should navigate to valid URL', async () => {
      const response = await apiClient.navigate('https://httpbin.org');
      
      expect(response).toBeValidApiResponse();
      expect(response.success).toBe(true);
    });

    it('should handle invalid URLs gracefully', async () => {
      const response = await apiClient.navigate('invalid-url');
      
      expect(response).toBeValidApiResponse();
      expect(response.success).toBe(false);
      expect(response.error).toBeDefined();
    });

    it('should support browser navigation controls', async () => {
      // Navigate to first page
      await apiClient.navigate('https://example.com');
      
      // Navigate to second page
      await apiClient.navigate('https://httpbin.org');
      
      // Go back
      const backResponse = await apiClient.goBack();
      expect(backResponse.success).toBe(true);
      
      // Go forward
      const forwardResponse = await apiClient.goForward();
      expect(forwardResponse.success).toBe(true);
      
      // Refresh
      const refreshResponse = await apiClient.refresh();
      expect(refreshResponse.success).toBe(true);
    });
  });

  describe('Interaction Tools', () => {
    beforeEach(async () => {
      await apiClient.navigate('https://httpbin.org/forms/post');
    });

    it('should click on elements', async () => {
      const response = await apiClient.click('input[name="custname"]');
      
      expect(response).toBeValidApiResponse();
      expect(response.success).toBe(true);
    });

    it('should type text into input fields', async () => {
      const testText = 'Test User Name';
      const response = await apiClient.typeText('input[name="custname"]', testText);
      
      expect(response).toBeValidApiResponse();
      expect(response.success).toBe(true);
      
      // Verify text was entered
      const extractResponse = await apiClient.extractText('input[name="custname"]');
      expect(extractResponse.success).toBe(true);
      expect(extractResponse.data?.text).toContain(testText);
    });

    it('should hover over elements', async () => {
      const response = await apiClient.hover('input[type="submit"]');
      
      expect(response).toBeValidApiResponse();
      expect(response.success).toBe(true);
    });

    it('should focus on elements', async () => {
      const response = await apiClient.focus('input[name="custel"]');
      
      expect(response).toBeValidApiResponse();
      expect(response.success).toBe(true);
    });

    it('should handle non-existent elements gracefully', async () => {
      const response = await apiClient.click('.non-existent-element');
      
      expect(response).toBeValidApiResponse();
      expect(response.success).toBe(false);
      expect(response.error).toBeDefined();
    });
  });

  describe('Extraction Tools', () => {
    beforeEach(async () => {
      await apiClient.navigate('https://example.com');
    });

    it('should extract text from elements', async () => {
      const response = await apiClient.extractText('h1');
      
      expect(response).toBeValidApiResponse();
      expect(response.success).toBe(true);
      expect(response.data?.text).toBeDefined();
      expect(typeof response.data?.text).toBe('string');
    });

    it('should extract links from page', async () => {
      const response = await apiClient.extractLinks();
      
      expect(response).toBeValidApiResponse();
      expect(response.success).toBe(true);
      expect(response.data?.links).toBeDefined();
      expect(Array.isArray(response.data?.links)).toBe(true);
    });

    it('should get element information', async () => {
      const response = await apiClient.getElementInfo('body');
      
      expect(response).toBeValidApiResponse();
      expect(response.success).toBe(true);
      expect(response.data).toBeDefined();
    });

    it('should handle extraction from non-existent elements', async () => {
      const response = await apiClient.extractText('.non-existent');
      
      expect(response).toBeValidApiResponse();
      expect(response.success).toBe(false);
    });
  });

  describe('Wait Tools', () => {
    beforeEach(async () => {
      await apiClient.navigate('https://example.com');
    });

    it('should wait for existing elements', async () => {
      const response = await apiClient.waitForElement('body', 5000);
      
      expect(response).toBeValidApiResponse();
      expect(response.success).toBe(true);
    });

    it('should timeout for non-existent elements', async () => {
      const response = await apiClient.waitForElement('.never-exists', 2000);
      
      expect(response).toBeValidApiResponse();
      expect(response.success).toBe(false);
    });
  });

  describe('Session Management', () => {
    it('should set and get session storage', async () => {
      const key = 'test_key';
      const value = 'test_value';
      
      // Set session storage
      const setResponse = await apiClient.setSessionStorage(key, value);
      expect(setResponse.success).toBe(true);
      
      // Get session storage
      const getResponse = await apiClient.getSessionStorage(key);
      expect(getResponse.success).toBe(true);
      expect(getResponse.data?.value).toBe(value);
    });

    it('should clear session storage', async () => {
      // Set some data first
      await apiClient.setSessionStorage('temp_key', 'temp_value');
      
      // Clear storage
      const clearResponse = await apiClient.clearSessionStorage();
      expect(clearResponse.success).toBe(true);
      
      // Verify data is cleared
      const getResponse = await apiClient.getSessionStorage('temp_key');
      expect(getResponse.data?.value).toBeNull();
    });
  });

  describe('Error Handling', () => {
    it('should handle malformed tool requests', async () => {
      const response = await apiClient.executeTool({
        tool_name: 'invalid_tool',
        parameters: {}
      });
      
      expect(response).toBeValidApiResponse();
      expect(response.success).toBe(false);
      expect(response.error).toBeDefined();
    });

    it('should handle missing parameters', async () => {
      const response = await apiClient.executeTool({
        tool_name: 'type_text',
        parameters: { selector: 'input' } // missing 'text' parameter
      });
      
      expect(response).toBeValidApiResponse();
      expect(response.success).toBe(false);
    });

    it('should recover from errors and continue working', async () => {
      // Cause an error
      await apiClient.click('.non-existent');
      
      // Verify system still works
      const healthResponse = await apiClient.health();
      expect(healthResponse.status).toBeTruthy();
      
      // Verify tools still work
      const navResponse = await apiClient.navigate('https://example.com');
      expect(navResponse.success).toBe(true);
    });
  });
});