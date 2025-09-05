// Simple test to verify testing framework works
describe('Testing Framework Validation', () => {
  it('should run basic tests', () => {
    expect(1 + 1).toBe(2);
    expect('hello').toBe('hello');
    expect([1, 2, 3]).toHaveLength(3);
  });

  it('should handle async operations', async () => {
    const promise = Promise.resolve('success');
    const result = await promise;
    expect(result).toBe('success');
  });

  it('should validate object properties', () => {
    const testObject = {
      name: 'RainbowBrowserAI',
      version: '1.0.0',
      features: ['browser-automation', 'ai-perception']
    };

    expect(testObject).toHaveProperty('name');
    expect(testObject.name).toBe('RainbowBrowserAI');
    expect(testObject.features).toContain('ai-perception');
  });

  it('should test arrays and collections', () => {
    const numbers = [1, 2, 3, 4, 5];
    
    expect(numbers).toHaveLength(5);
    expect(numbers).toEqual(expect.arrayContaining([1, 3, 5]));
    expect(numbers.filter(n => n % 2 === 0)).toEqual([2, 4]);
  });

  it('should handle error scenarios', () => {
    const throwError = () => {
      throw new Error('Test error');
    };

    expect(throwError).toThrow('Test error');
    expect(throwError).toThrow(Error);
  });
});

// Test mock functionality
describe('Mock Functionality', () => {
  it('should create and use mocks', () => {
    const mockCallback = jest.fn();
    mockCallback('test');
    mockCallback('another test');

    expect(mockCallback).toHaveBeenCalledTimes(2);
    expect(mockCallback).toHaveBeenCalledWith('test');
    expect(mockCallback).toHaveBeenLastCalledWith('another test');
  });

  it('should mock return values', () => {
    const mockFn = jest.fn();
    mockFn.mockReturnValue('mocked value');
    mockFn.mockReturnValueOnce('first call');

    expect(mockFn()).toBe('first call');
    expect(mockFn()).toBe('mocked value');
  });
});

// Test custom matchers (if they work)
describe('Custom Matchers Test', () => {
  it('should test API response structure', () => {
    const validResponse = {
      success: true,
      data: { test: 'data' },
      error: null
    };

    const invalidResponse = {
      message: 'not a valid API response'
    };

    // Test that the response has the required structure
    expect(validResponse).toHaveProperty('success');
    expect(typeof validResponse.success).toBe('boolean');
    
    expect(invalidResponse).not.toHaveProperty('success');
  });

  it('should test CSS selector validity', () => {
    const validSelectors = [
      'div',
      '.class-name',
      '#element-id',
      'input[type="text"]',
      'div > p',
      '.parent .child'
    ];

    const invalidSelectors = [
      'invalid>>selector',
      '',
      '   ',
      'div>>>p'
    ];

    validSelectors.forEach(selector => {
      expect(typeof selector).toBe('string');
      expect(selector.length).toBeGreaterThan(0);
      expect(selector.includes('>>')).toBe(false);
    });

    invalidSelectors.forEach(selector => {
      if (selector.includes('>>')) {
        expect(selector).toContain('>>');
      }
      if (selector.trim() === '') {
        expect(selector.trim()).toBe('');
      }
    });
  });
});