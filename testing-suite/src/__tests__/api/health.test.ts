import { ApiClient } from '../../utils/api-client';

describe('Health API', () => {
  let apiClient: ApiClient;

  beforeAll(() => {
    apiClient = new ApiClient(global.TEST_CONFIG.serverUrl);
  });

  describe('GET /api/health', () => {
    it('should return healthy status', async () => {
      const response = await apiClient.health();
      
      expect(response).toBeDefined();
      expect(response.status).toBeTruthy();
    });

    it('should respond within acceptable time', async () => {
      const startTime = Date.now();
      await apiClient.health();
      const endTime = Date.now();
      
      const responseTime = endTime - startTime;
      expect(responseTime).toBeLessThan(1000); // Should respond within 1 second
    });

    it('should handle multiple concurrent requests', async () => {
      const promises = Array(5).fill(null).map(() => apiClient.health());
      
      const responses = await Promise.all(promises);
      
      responses.forEach(response => {
        expect(response.status).toBeTruthy();
      });
    });
  });
});