// Test custom matchers functionality
describe('Custom Matchers', () => {
  describe('toBeValidApiResponse matcher', () => {
    it('should pass for valid API responses', () => {
      const validResponse = {
        success: true,
        data: { message: 'test' },
        error: null
      };

      expect(validResponse).toBeValidApiResponse();
    });

    it('should pass for valid error responses', () => {
      const errorResponse = {
        success: false,
        data: null,
        error: 'Something went wrong'
      };

      expect(errorResponse).toBeValidApiResponse();
    });

    it('should fail for invalid responses', () => {
      const invalidResponse = {
        message: 'not a valid response',
        status: 'error'
      };

      expect(() => {
        expect(invalidResponse).toBeValidApiResponse();
      }).toThrow();
    });

    it('should fail for null or undefined', () => {
      expect(() => {
        expect(null).toBeValidApiResponse();
      }).toThrow();

      expect(() => {
        expect(undefined).toBeValidApiResponse();
      }).toThrow();
    });
  });

  describe('toHaveValidSelector matcher', () => {
    it('should pass for valid CSS selectors', () => {
      const validSelectors = [
        'div',
        '.class-name',
        '#element-id',
        'input[type="text"]',
        'div > p',
        '.parent .child',
        'button:hover',
        'tr:nth-child(2n)'
      ];

      validSelectors.forEach(selector => {
        expect(selector).toHaveValidSelector();
      });
    });

    it('should fail for invalid selectors', () => {
      const invalidSelectors = [
        'invalid>>selector',  // Contains >>
        '',                   // Empty string
        '   ',               // Only whitespace
      ];

      invalidSelectors.forEach(selector => {
        expect(() => {
          expect(selector).toHaveValidSelector();
        }).toThrow();
      });
    });

    it('should fail for non-string values', () => {
      const nonStringValues = [null, undefined, 123, {}, []];

      nonStringValues.forEach(value => {
        expect(() => {
          expect(value).toHaveValidSelector();
        }).toThrow();
      });
    });
  });

  // Test realistic API response scenarios
  describe('Realistic API Response Testing', () => {
    it('should validate tool execution response', () => {
      const toolResponse = {
        success: true,
        data: {
          tool_name: 'click',
          result: 'Element clicked successfully',
          timestamp: new Date().toISOString()
        },
        error: null
      };

      expect(toolResponse).toBeValidApiResponse();
      expect(toolResponse.success).toBe(true);
      expect(toolResponse.data.tool_name).toBe('click');
    });

    it('should validate perception response', () => {
      const perceptionResponse = {
        success: true,
        data: {
          selector: 'input[name="username"]',
          confidence: 0.95,
          element_type: 'input'
        },
        error: null
      };

      expect(perceptionResponse).toBeValidApiResponse();
      expect(perceptionResponse.data.selector).toHaveValidSelector();
      expect(perceptionResponse.data.confidence).toBeGreaterThan(0.9);
    });

    it('should validate error response', () => {
      const errorResponse = {
        success: false,
        data: null,
        error: 'Element not found: .non-existent-selector'
      };

      expect(errorResponse).toBeValidApiResponse();
      expect(errorResponse.success).toBe(false);
      expect(errorResponse.error).toContain('Element not found');
    });
  });
});

// Test performance of custom matchers
describe('Custom Matcher Performance', () => {
  it('should perform validations quickly', () => {
    const startTime = performance.now();
    
    // Run many validations
    for (let i = 0; i < 1000; i++) {
      const response = {
        success: true,
        data: { test: `data-${i}` },
        error: null
      };
      expect(response).toBeValidApiResponse();
    }
    
    const endTime = performance.now();
    const executionTime = endTime - startTime;
    
    // Should complete within reasonable time
    expect(executionTime).toBeLessThan(100); // 100ms for 1000 validations
  });

  it('should handle selector validation efficiently', () => {
    const selectors = [
      'div', '.class', '#id', 'input[type="text"]',
      'div > p', '.a .b', 'button:hover', 'tr:nth-child(2n)',
      'form input', '.modal .close-btn'
    ];

    const startTime = performance.now();
    
    // Validate each selector 100 times
    selectors.forEach(selector => {
      for (let i = 0; i < 100; i++) {
        expect(selector).toHaveValidSelector();
      }
    });
    
    const endTime = performance.now();
    const executionTime = endTime - startTime;
    
    // Should be very fast
    expect(executionTime).toBeLessThan(50); // 50ms for 1000 validations
  });
});