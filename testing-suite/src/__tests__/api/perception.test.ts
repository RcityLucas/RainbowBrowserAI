import { ApiClient } from '../../utils/api-client';

describe('Perception API', () => {
  let apiClient: ApiClient;

  beforeAll(() => {
    apiClient = new ApiClient(global.TEST_CONFIG.serverUrl);
  });

  beforeEach(async () => {
    // Navigate to a test page before each perception test
    await apiClient.navigate('https://example.com');
  });

  describe('Page Analysis', () => {
    it('should analyze current page', async () => {
      const response = await apiClient.analyzePage();
      
      expect(response).toBeValidApiResponse();
      expect(response.success).toBe(true);
      expect(response.data).toBeDefined();
      expect(response.data.url).toBeDefined();
      expect(response.data.title).toBeDefined();
      expect(response.data.page_type).toBeDefined();
      expect(response.data.timestamp).toBeDefined();
    });

    it('should identify page URL correctly', async () => {
      await apiClient.navigate('https://httpbin.org');
      const response = await apiClient.analyzePage();
      
      expect(response.success).toBe(true);
      expect(response.data.url).toContain('httpbin.org');
    });

    it('should classify different page types', async () => {
      // Test with form page
      await apiClient.navigate('https://httpbin.org/forms/post');
      const formResponse = await apiClient.analyzePage();
      
      expect(formResponse.success).toBe(true);
      expect(formResponse.data.page_type).toBeDefined();
      
      // Test with simple page
      await apiClient.navigate('https://example.com');
      const simpleResponse = await apiClient.analyzePage();
      
      expect(simpleResponse.success).toBe(true);
      expect(simpleResponse.data.page_type).toBeDefined();
    });

    it('should include semantic analysis', async () => {
      const response = await apiClient.analyzePage();
      
      expect(response.success).toBe(true);
      expect(response.data).toHaveProperty('semantic_elements');
    });
  });

  describe('Element Finding', () => {
    it('should find elements by natural language description', async () => {
      const testCases = [
        { description: 'heading', expectedType: 'string' },
        { description: 'link', expectedType: 'string' },
        { description: 'paragraph', expectedType: 'string' }
      ];

      for (const testCase of testCases) {
        const response = await apiClient.findElement({ description: testCase.description });
        
        expect(response).toBeValidApiResponse();
        expect(response.success).toBe(true);
        expect(response.data?.selector).toBeDefined();
        expect(response.data?.selector).toHaveValidSelector();
        expect(response.data?.confidence).toBeGreaterThanOrEqual(0);
        expect(response.data?.confidence).toBeLessThanOrEqual(1);
      }
    });

    it('should provide confidence scores', async () => {
      const response = await apiClient.findElement({ description: 'main heading' });
      
      expect(response.success).toBe(true);
      expect(response.data?.confidence).toBeDefined();
      expect(typeof response.data?.confidence).toBe('number');
      expect(response.data?.confidence).toBeGreaterThanOrEqual(0);
      expect(response.data?.confidence).toBeLessThanOrEqual(1);
    });

    it('should handle non-existent element descriptions gracefully', async () => {
      const response = await apiClient.findElement({ 
        description: 'super-unique-non-existent-element-that-never-exists' 
      });
      
      expect(response).toBeValidApiResponse();
      // Should either return low confidence or fail gracefully
      if (response.success) {
        expect(response.data?.confidence).toBeLessThan(0.5);
      } else {
        expect(response.error).toBeDefined();
      }
    });

    it('should find form elements specifically', async () => {
      await apiClient.navigate('https://httpbin.org/forms/post');
      
      const formElementTests = [
        'customer name input',
        'telephone field',
        'email input',
        'submit button'
      ];

      for (const description of formElementTests) {
        const response = await apiClient.findElement({ description });
        
        expect(response).toBeValidApiResponse();
        if (response.success) {
          expect(response.data?.selector).toHaveValidSelector();
        }
      }
    });
  });

  describe('Intelligent Commands', () => {
    beforeEach(async () => {
      await apiClient.navigate('https://httpbin.org/forms/post');
    });

    it('should execute click commands with natural language', async () => {
      const response = await apiClient.executePerceptionCommand({
        command: {
          action: 'click',
          description: 'customer name field',
          parameters: {}
        }
      });
      
      expect(response).toBeValidApiResponse();
      expect(response.success).toBe(true);
    });

    it('should execute extract commands', async () => {
      const response = await apiClient.executePerceptionCommand({
        command: {
          action: 'extract',
          description: 'page title',
          parameters: {}
        }
      });
      
      expect(response).toBeValidApiResponse();
      expect(response.success).toBe(true);
    });

    it('should execute analysis commands', async () => {
      const response = await apiClient.executePerceptionCommand({
        command: {
          action: 'analyze',
          description: 'form structure',
          parameters: {}
        }
      });
      
      expect(response).toBeValidApiResponse();
      expect(response.success).toBe(true);
    });

    it('should handle invalid command actions', async () => {
      const response = await apiClient.executePerceptionCommand({
        command: {
          action: 'invalid_action',
          description: 'something',
          parameters: {}
        }
      });
      
      expect(response).toBeValidApiResponse();
      expect(response.success).toBe(false);
      expect(response.error).toBeDefined();
    });
  });

  describe('Form Analysis', () => {
    beforeEach(async () => {
      await apiClient.navigate('https://httpbin.org/forms/post');
    });

    it('should analyze forms on the page', async () => {
      const response = await apiClient.analyzeForms({ form_selector: 'form' });
      
      expect(response).toBeValidApiResponse();
      expect(response.success).toBe(true);
      expect(response.data?.form_type).toBeDefined();
      expect(response.data?.fields).toBeDefined();
      expect(Array.isArray(response.data?.fields)).toBe(true);
    });

    it('should detect form fields correctly', async () => {
      const response = await apiClient.analyzeForms({ form_selector: 'form' });
      
      expect(response.success).toBe(true);
      
      if (response.data?.fields && response.data.fields.length > 0) {
        const field = response.data.fields[0];
        expect(field).toHaveProperty('field_type');
        expect(field).toHaveProperty('name');
        expect(field).toHaveProperty('selector');
      }
    });

    it('should handle pages without forms', async () => {
      await apiClient.navigate('https://example.com');
      const response = await apiClient.analyzeForms({ form_selector: null });
      
      expect(response).toBeValidApiResponse();
      // Should either succeed with empty results or indicate no forms found
      if (response.success) {
        expect(response.data?.fields).toBeDefined();
      }
    });

    it('should analyze specific forms by selector', async () => {
      const response = await apiClient.analyzeForms({ form_selector: 'form' });
      
      expect(response).toBeValidApiResponse();
      expect(response.success).toBe(true);
    });
  });

  describe('Performance', () => {
    it('should analyze pages within reasonable time', async () => {
      const startTime = Date.now();
      await apiClient.analyzePage();
      const endTime = Date.now();
      
      const responseTime = endTime - startTime;
      expect(responseTime).toBeLessThan(10000); // Should complete within 10 seconds
    });

    it('should find elements within reasonable time', async () => {
      const startTime = Date.now();
      await apiClient.findElement({ description: 'link' });
      const endTime = Date.now();
      
      const responseTime = endTime - startTime;
      expect(responseTime).toBeLessThan(5000); // Should complete within 5 seconds
    });

    it('should handle concurrent perception requests', async () => {
      const promises = [
        apiClient.findElement({ description: 'heading' }),
        apiClient.findElement({ description: 'link' }),
        apiClient.findElement({ description: 'paragraph' })
      ];
      
      const responses = await Promise.all(promises);
      
      responses.forEach(response => {
        expect(response).toBeValidApiResponse();
      });
    });
  });

  describe('Integration with Tools', () => {
    it('should integrate with navigation tools', async () => {
      // Use perception to analyze page after navigation
      await apiClient.navigate('https://github.com');
      const analysis = await apiClient.analyzePage();
      
      expect(analysis.success).toBe(true);
      expect(analysis.data.url).toContain('github');
      
      // Use perception to find elements on the new page
      const elementFind = await apiClient.findElement({ description: 'search' });
      expect(elementFind).toBeValidApiResponse();
    });

    it('should provide selectors usable by interaction tools', async () => {
      await apiClient.navigate('https://httpbin.org/forms/post');
      
      // Find element using perception
      const findResponse = await apiClient.findElement({ 
        description: 'customer name input' 
      });
      
      if (findResponse.success && findResponse.data?.selector) {
        // Use the found selector with interaction tools
        const clickResponse = await apiClient.click(findResponse.data.selector);
        expect(clickResponse).toBeValidApiResponse();
        
        const typeResponse = await apiClient.typeText(
          findResponse.data.selector, 
          'Perception Integration Test'
        );
        expect(typeResponse).toBeValidApiResponse();
      }
    });
  });
});