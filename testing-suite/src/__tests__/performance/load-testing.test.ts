import { ApiClient } from '../../utils/api-client';

describe('Performance and Load Testing', () => {
  let apiClient: ApiClient;

  beforeAll(() => {
    apiClient = new ApiClient(global.TEST_CONFIG.serverUrl);
  });

  describe('Response Time Performance', () => {
    const measureExecutionTime = async (operation: () => Promise<any>): Promise<number> => {
      const startTime = performance.now();
      await operation();
      const endTime = performance.now();
      return endTime - startTime;
    };

    it('should have fast health check response', async () => {
      const executionTimes = [];
      
      // Measure multiple executions
      for (let i = 0; i < 10; i++) {
        const time = await measureExecutionTime(() => apiClient.health());
        executionTimes.push(time);
      }
      
      const averageTime = executionTimes.reduce((a, b) => a + b) / executionTimes.length;
      const maxTime = Math.max(...executionTimes);
      
      expect(averageTime).toBeLessThan(100); // Average should be < 100ms
      expect(maxTime).toBeLessThan(500); // No single request should take > 500ms
    });

    it('should have acceptable navigation performance', async () => {
      const navigationTimes = [];
      const testUrls = [
        'https://example.com',
        'https://httpbin.org',
        'https://github.com'
      ];
      
      for (const url of testUrls) {
        const time = await measureExecutionTime(() => apiClient.navigate(url));
        navigationTimes.push(time);
      }
      
      const averageTime = navigationTimes.reduce((a, b) => a + b) / navigationTimes.length;
      expect(averageTime).toBeLessThan(8000); // Average navigation < 8 seconds
    });

    it('should have fast tool execution', async () => {
      await apiClient.navigate('https://example.com');
      
      const toolTests = [
        () => apiClient.extractText('h1'),
        () => apiClient.extractLinks(),
        () => apiClient.getElementInfo('body'),
        () => apiClient.click('h1'),
        () => apiClient.hover('a')
      ];
      
      for (const toolTest of toolTests) {
        const time = await measureExecutionTime(toolTest);
        expect(time).toBeLessThan(3000); // Each tool execution < 3 seconds
      }
    });

    it('should have acceptable perception performance', async () => {
      await apiClient.navigate('https://httpbin.org/forms/post');
      
      const perceptionTests = [
        () => apiClient.analyzePage(),
        () => apiClient.findElement({ description: 'heading' }),
        () => apiClient.findElement({ description: 'input field' }),
        () => apiClient.analyzeForms({ form_selector: 'form' })
      ];
      
      for (const test of perceptionTests) {
        const time = await measureExecutionTime(test);
        expect(time).toBeLessThan(10000); // Perception operations < 10 seconds
      }
    });
  });

  describe('Concurrent Load Testing', () => {
    it('should handle concurrent health checks', async () => {
      const concurrentRequests = 20;
      const promises = Array(concurrentRequests)
        .fill(null)
        .map(() => apiClient.health());
      
      const startTime = performance.now();
      const results = await Promise.all(promises);
      const totalTime = performance.now() - startTime;
      
      // All requests should succeed
      results.forEach(result => {
        expect(result.status).toBeTruthy();
      });
      
      // Total time should be reasonable for concurrent execution
      expect(totalTime).toBeLessThan(5000); // All 20 requests in < 5 seconds
    });

    it('should handle concurrent navigation requests', async () => {
      const urls = [
        'https://example.com',
        'https://httpbin.org',
        'https://github.com',
        'https://httpbin.org/forms/post'
      ];
      
      const promises = urls.map(url => apiClient.navigate(url));
      
      const startTime = performance.now();
      const results = await Promise.all(promises);
      const totalTime = performance.now() - startTime;
      
      // At least some requests should succeed (browser pool limitations)
      const successCount = results.filter(r => r.success).length;
      expect(successCount).toBeGreaterThan(0);
      
      // Should complete within reasonable time
      expect(totalTime).toBeLessThan(30000); // All navigation in < 30 seconds
    });

    it('should handle concurrent tool executions', async () => {
      await apiClient.navigate('https://example.com');
      
      const toolPromises = [
        apiClient.extractText('h1'),
        apiClient.extractText('p'),
        apiClient.extractLinks(),
        apiClient.getElementInfo('body'),
        apiClient.hover('h1')
      ];
      
      const results = await Promise.all(toolPromises);
      
      // Most operations should succeed
      const successCount = results.filter(r => r.success).length;
      expect(successCount).toBeGreaterThan(3);
    });

    it('should handle mixed concurrent operations', async () => {
      const mixedPromises = [
        apiClient.health(),
        apiClient.health(),
        apiClient.navigate('https://example.com'),
        apiClient.extractText('body'),
        apiClient.analyzePage(),
        apiClient.health(),
        apiClient.getElementInfo('html')
      ];
      
      const results = await Promise.all(mixedPromises);
      
      // Health checks should always succeed
      expect(results[0].status).toBeTruthy();
      expect(results[1].status).toBeTruthy();
      expect(results[5].status).toBeTruthy();
      
      // At least some other operations should succeed
      const otherResults = [results[2], results[3], results[4], results[6]];
      const successCount = otherResults.filter(r => r.success).length;
      expect(successCount).toBeGreaterThan(2);
    });
  });

  describe('Memory and Resource Management', () => {
    it('should handle repeated operations without degradation', async () => {
      const iterations = 50;
      const operationTimes = [];
      
      for (let i = 0; i < iterations; i++) {
        const startTime = performance.now();
        await apiClient.health();
        const endTime = performance.now();
        
        operationTimes.push(endTime - startTime);
      }
      
      // Check that performance doesn't degrade over time
      const firstTenAverage = operationTimes.slice(0, 10)
        .reduce((a, b) => a + b) / 10;
      const lastTenAverage = operationTimes.slice(-10)
        .reduce((a, b) => a + b) / 10;
      
      // Last 10 operations shouldn't be significantly slower than first 10
      expect(lastTenAverage).toBeLessThan(firstTenAverage * 2);
    });

    it('should handle memory-intensive operations', async () => {
      await apiClient.navigate('https://github.com');
      
      // Perform memory-intensive operations
      const operations = [
        () => apiClient.extractText('body'), // Large text extraction
        () => apiClient.extractLinks(), // Many links
        () => apiClient.getElementInfo('*'), // Many elements
        () => apiClient.analyzePage() // Complex analysis
      ];
      
      // Run each operation multiple times
      for (let i = 0; i < 5; i++) {
        for (const operation of operations) {
          const result = await operation();
          // System should remain responsive
          expect(result).toBeValidApiResponse();
        }
        
        // Verify system health after each round
        const health = await apiClient.health();
        expect(health.status).toBeTruthy();
      }
    });
  });

  describe('Stress Testing', () => {
    it('should survive rapid successive requests', async () => {
      const rapidRequests = 100;
      const delay = 10; // 10ms between requests
      
      const results = [];
      
      for (let i = 0; i < rapidRequests; i++) {
        results.push(apiClient.health());
        await new Promise(resolve => setTimeout(resolve, delay));
      }
      
      const responses = await Promise.all(results);
      
      // At least 90% should succeed under rapid load
      const successCount = responses.filter(r => r.status).length;
      const successRate = successCount / rapidRequests;
      expect(successRate).toBeGreaterThan(0.9);
    });

    it('should handle burst traffic patterns', async () => {
      // Simulate burst patterns: high load, then quiet period
      const burstSize = 15;
      const quietPeriod = 1000; // 1 second
      const burstCount = 3;
      
      for (let burst = 0; burst < burstCount; burst++) {
        // Generate burst of requests
        const burstPromises = Array(burstSize)
          .fill(null)
          .map(() => apiClient.health());
        
        const burstResults = await Promise.all(burstPromises);
        
        // Most requests in burst should succeed
        const successCount = burstResults.filter(r => r.status).length;
        expect(successCount).toBeGreaterThan(burstSize * 0.7);
        
        // Quiet period
        if (burst < burstCount - 1) {
          await new Promise(resolve => setTimeout(resolve, quietPeriod));
        }
      }
    });
  });

  describe('Throughput Testing', () => {
    it('should maintain acceptable throughput under load', async () => {
      const duration = 10000; // 10 seconds
      const startTime = Date.now();
      let requestCount = 0;
      const errors = [];
      
      // Send requests continuously for the duration
      while (Date.now() - startTime < duration) {
        try {
          await apiClient.health();
          requestCount++;
        } catch (error) {
          errors.push(error);
        }
      }
      
      const actualDuration = Date.now() - startTime;
      const throughput = requestCount / (actualDuration / 1000); // requests per second
      
      expect(throughput).toBeGreaterThan(5); // At least 5 requests per second
      expect(errors.length / requestCount).toBeLessThan(0.1); // Less than 10% errors
    });
  });

  describe('Error Recovery Under Load', () => {
    it('should recover from errors under load', async () => {
      // Mix of valid and invalid operations
      const operations = [
        () => apiClient.health(),
        () => apiClient.navigate('invalid-url'),
        () => apiClient.click('.non-existent'),
        () => apiClient.extractText('.missing'),
        () => apiClient.health()
      ];
      
      // Run operations multiple times concurrently
      const allPromises = [];
      for (let i = 0; i < 5; i++) {
        allPromises.push(...operations.map(op => op()));
      }
      
      const results = await Promise.all(allPromises.map(p => 
        p.catch(error => ({ success: false, error: error.message }))
      ));
      
      // Health checks should still work
      const healthResults = results.filter((_, index) => 
        index % operations.length === 0 || index % operations.length === 4
      );
      
      const healthSuccessCount = healthResults.filter(r => r.status || r.success !== false).length;
      expect(healthSuccessCount / healthResults.length).toBeGreaterThan(0.8);
    });
  });
});