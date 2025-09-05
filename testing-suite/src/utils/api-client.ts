import axios, { AxiosInstance } from 'axios';

export interface ApiResponse<T = any> {
  success: boolean;
  data?: T;
  error?: string;
}

export interface ToolExecutionRequest {
  tool_name: string;
  parameters: Record<string, any>;
}

export interface PerceptionRequest {
  description?: string;
  command?: {
    action: string;
    description: string;
    parameters: Record<string, any>;
  };
  form_selector?: string;
}

export class ApiClient {
  private client: AxiosInstance;
  
  constructor(baseUrl: string = 'http://localhost:3002') {
    this.client = axios.create({
      baseURL: baseUrl,
      timeout: 30000,
      headers: {
        'Content-Type': 'application/json'
      }
    });
    
    // Add response interceptor for better error handling
    this.client.interceptors.response.use(
      (response) => response,
      (error) => {
        if (error.response) {
          // Server responded with error status
          throw new Error(`API Error ${error.response.status}: ${JSON.stringify(error.response.data)}`);
        } else if (error.request) {
          // Request was made but no response received
          throw new Error('Network Error: No response from server');
        } else {
          // Something else happened
          throw new Error(`Request Error: ${error.message}`);
        }
      }
    );
  }

  // Health check
  async health(): Promise<{ status: string }> {
    const response = await this.client.get('/api/health');
    return response.data;
  }

  // Get available tools
  async getTools(): Promise<{ tools: any[] }> {
    const response = await this.client.get('/api/tools');
    return response.data;
  }

  // Execute a tool
  async executeTool(request: ToolExecutionRequest): Promise<ApiResponse> {
    const response = await this.client.post('/api/tools/execute', request);
    return response.data;
  }

  // Perception endpoints
  async analyzePage(request: {} = {}): Promise<ApiResponse> {
    const response = await this.client.post('/api/perception/analyze', request);
    return response.data;
  }

  async findElement(request: PerceptionRequest): Promise<ApiResponse> {
    const response = await this.client.post('/api/perception/find', request);
    return response.data;
  }

  async executePerceptionCommand(request: PerceptionRequest): Promise<ApiResponse> {
    const response = await this.client.post('/api/perception/command', request);
    return response.data;
  }

  async analyzeForms(request: PerceptionRequest): Promise<ApiResponse> {
    const response = await this.client.post('/api/perception/forms/analyze', request);
    return response.data;
  }

  // Helper methods for common tool operations
  async navigate(url: string): Promise<ApiResponse> {
    return this.executeTool({
      tool_name: 'navigate_to_url',
      parameters: { url }
    });
  }

  async click(selector: string): Promise<ApiResponse> {
    return this.executeTool({
      tool_name: 'click',
      parameters: { selector }
    });
  }

  async typeText(selector: string, text: string): Promise<ApiResponse> {
    return this.executeTool({
      tool_name: 'type_text',
      parameters: { selector, text }
    });
  }

  async extractText(selector: string): Promise<ApiResponse> {
    return this.executeTool({
      tool_name: 'extract_text',
      parameters: { selector }
    });
  }

  async extractLinks(selector: string = 'a'): Promise<ApiResponse> {
    return this.executeTool({
      tool_name: 'extract_links',
      parameters: { selector }
    });
  }

  async waitForElement(selector: string, timeout: number = 5000): Promise<ApiResponse> {
    return this.executeTool({
      tool_name: 'wait_for_element',
      parameters: { selector, timeout }
    });
  }

  async hover(selector: string): Promise<ApiResponse> {
    return this.executeTool({
      tool_name: 'hover',
      parameters: { selector }
    });
  }

  async focus(selector: string): Promise<ApiResponse> {
    return this.executeTool({
      tool_name: 'focus',
      parameters: { selector }
    });
  }

  async goBack(): Promise<ApiResponse> {
    return this.executeTool({
      tool_name: 'go_back',
      parameters: {}
    });
  }

  async goForward(): Promise<ApiResponse> {
    return this.executeTool({
      tool_name: 'go_forward',
      parameters: {}
    });
  }

  async refresh(): Promise<ApiResponse> {
    return this.executeTool({
      tool_name: 'refresh',
      parameters: {}
    });
  }

  async getElementInfo(selector: string): Promise<ApiResponse> {
    return this.executeTool({
      tool_name: 'get_element_info',
      parameters: { selector }
    });
  }

  // Session management
  async setSessionStorage(key: string, value: string): Promise<ApiResponse> {
    return this.executeTool({
      tool_name: 'session_storage',
      parameters: { action: 'set', key, value }
    });
  }

  async getSessionStorage(key: string): Promise<ApiResponse> {
    return this.executeTool({
      tool_name: 'session_storage',
      parameters: { action: 'get', key }
    });
  }

  async clearSessionStorage(): Promise<ApiResponse> {
    return this.executeTool({
      tool_name: 'session_storage',
      parameters: { action: 'clear' }
    });
  }

  // Utility method to retry operations
  async withRetry<T>(
    operation: () => Promise<T>, 
    maxRetries: number = 3, 
    delayMs: number = 1000
  ): Promise<T> {
    let lastError: Error;
    
    for (let attempt = 1; attempt <= maxRetries; attempt++) {
      try {
        return await operation();
      } catch (error) {
        lastError = error as Error;
        
        if (attempt === maxRetries) {
          throw lastError;
        }
        
        console.log(`Attempt ${attempt} failed, retrying in ${delayMs}ms...`);
        await new Promise(resolve => setTimeout(resolve, delayMs));
      }
    }
    
    throw lastError!;
  }
}