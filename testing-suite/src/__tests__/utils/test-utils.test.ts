import { TestUtils } from '../../utils/test-utils';

describe('TestUtils', () => {
  describe('isValidApiResponse', () => {
    it('should return true for valid API responses', () => {
      const validResponses = [
        { success: true, data: 'test' },
        { success: false, error: 'error' },
        { success: true, data: null, error: null }
      ];

      validResponses.forEach(response => {
        expect(TestUtils.isValidApiResponse(response)).toBe(true);
      });
    });

    it('should return false for invalid responses', () => {
      const invalidResponses = [
        null,
        undefined,
        'string',
        123,
        [],
        { message: 'not valid' },
        { success: 'not boolean' }
      ];

      invalidResponses.forEach(response => {
        expect(TestUtils.isValidApiResponse(response)).toBe(false);
      });
    });
  });

  describe('isValidSelector', () => {
    it('should return true for valid CSS selectors', () => {
      const validSelectors = [
        'div',
        '.class',
        '#id',
        'input[type="text"]',
        'div > p',
        '.parent .child'
      ];

      validSelectors.forEach(selector => {
        expect(TestUtils.isValidSelector(selector)).toBe(true);
      });
    });

    it('should return false for invalid selectors', () => {
      const invalidSelectors = [
        '',
        '   ',
        'invalid>>selector',
        null as any,
        undefined as any,
        123 as any
      ];

      invalidSelectors.forEach(selector => {
        expect(TestUtils.isValidSelector(selector)).toBe(false);
      });
    });
  });

  describe('createMockResponse', () => {
    it('should create success responses correctly', () => {
      const data = { message: 'test' };
      const response = TestUtils.createMockResponse(true, data);

      expect(response.success).toBe(true);
      expect(response.data).toEqual(data);
      expect(response.error).toBeNull();
      expect(response.timestamp).toBeDefined();
    });

    it('should create error responses correctly', () => {
      const error = 'Something went wrong';
      const response = TestUtils.createMockResponse(false, undefined, error);

      expect(response.success).toBe(false);
      expect(response.data).toBeNull();
      expect(response.error).toBe(error);
      expect(response.timestamp).toBeDefined();
    });
  });

  describe('delay', () => {
    it('should wait for specified time', async () => {
      const startTime = Date.now();
      await TestUtils.delay(100);
      const endTime = Date.now();
      
      const elapsed = endTime - startTime;
      expect(elapsed).toBeGreaterThanOrEqual(90); // Allow some variance
      expect(elapsed).toBeLessThan(150); // But not too much
    });
  });

  describe('generateFormData', () => {
    it('should generate form data for common fields', () => {
      const fields = ['name', 'email', 'phone'];
      const data = TestUtils.generateFormData(fields);

      expect(data.name).toContain('Test User');
      expect(data.email).toContain('@example.com');
      expect(data.phone).toMatch(/555-\d{4}/);
    });

    it('should generate generic data for unknown fields', () => {
      const fields = ['customField', 'anotherField'];
      const data = TestUtils.generateFormData(fields);

      expect(data.customField).toContain('test-value');
      expect(data.anotherField).toContain('test-value');
    });

    it('should handle empty field array', () => {
      const data = TestUtils.generateFormData([]);
      expect(Object.keys(data)).toHaveLength(0);
    });
  });

  describe('measureExecutionTime', () => {
    it('should measure synchronous function execution time', async () => {
      const syncFunction = () => {
        // Simulate some work
        let sum = 0;
        for (let i = 0; i < 1000; i++) {
          sum += i;
        }
        return sum;
      };

      const { result, time } = await TestUtils.measureExecutionTime(syncFunction);
      
      expect(result).toBe(499500); // Sum of 0 to 999
      expect(time).toBeGreaterThan(0);
      expect(time).toBeLessThan(100); // Should be very fast
    });

    it('should measure asynchronous function execution time', async () => {
      const asyncFunction = async () => {
        await TestUtils.delay(50);
        return 'done';
      };

      const { result, time } = await TestUtils.measureExecutionTime(asyncFunction);
      
      expect(result).toBe('done');
      expect(time).toBeGreaterThanOrEqual(40); // Account for some variance
      expect(time).toBeLessThan(100);
    });
  });

  describe('createTestSelector', () => {
    it('should create valid selectors', () => {
      const validSelector = TestUtils.createTestSelector('valid');
      expect(TestUtils.isValidSelector(validSelector)).toBe(true);
    });

    it('should create invalid selectors', () => {
      const invalidSelector = TestUtils.createTestSelector('invalid');
      expect(TestUtils.isValidSelector(invalidSelector)).toBe(false);
    });

    it('should create complex selectors', () => {
      const complexSelector = TestUtils.createTestSelector('complex');
      expect(TestUtils.isValidSelector(complexSelector)).toBe(true);
      expect(complexSelector).toContain(':not(');
    });

    it('should throw for unknown selector type', () => {
      expect(() => {
        TestUtils.createTestSelector('unknown' as any);
      }).toThrow('Unknown selector type');
    });
  });

  describe('validateTestEnvironment', () => {
    it('should validate current test environment', () => {
      const validation = TestUtils.validateTestEnvironment();
      
      expect(validation.valid).toBe(true);
      expect(validation.issues).toHaveLength(0);
    });

    // This test demonstrates how we would handle environment issues
    it('should report issues in incomplete environments', () => {
      // We can't actually remove global functions in Jest, but we can test the logic
      const originalPerformance = global.performance;
      delete (global as any).performance;
      
      const validation = TestUtils.validateTestEnvironment();
      
      expect(validation.valid).toBe(false);
      expect(validation.issues).toContain('Performance API not available');
      
      // Restore
      global.performance = originalPerformance;
    });
  });

  // Integration test using multiple utilities together
  describe('Integration Tests', () => {
    it('should work together in realistic scenarios', async () => {
      // Generate test data
      const formData = TestUtils.generateFormData(['name', 'email']);
      expect(formData.name).toBeDefined();
      expect(formData.email).toBeDefined();

      // Create mock response
      const response = TestUtils.createMockResponse(true, formData);
      expect(TestUtils.isValidApiResponse(response)).toBe(true);

      // Measure performance
      const { result, time } = await TestUtils.measureExecutionTime(async () => {
        await TestUtils.delay(10);
        return TestUtils.createTestSelector('valid');
      });

      expect(TestUtils.isValidSelector(result)).toBe(true);
      expect(time).toBeGreaterThanOrEqual(5);
    });
  });
});