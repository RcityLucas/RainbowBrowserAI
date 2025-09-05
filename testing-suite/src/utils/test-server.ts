import { spawn, ChildProcess } from 'child_process';
import axios from 'axios';
import path from 'path';

export class TestServer {
  private process: ChildProcess | null = null;
  private readonly port: number = 3002;
  private readonly maxRetries: number = 30;
  private readonly retryDelay: number = 1000;

  constructor(port: number = 3002) {
    this.port = port;
  }

  async start(): Promise<void> {
    try {
      // Kill any existing process on the port
      await this.killExistingProcess();
      
      // Start the server process
      await this.startServerProcess();
      
      // Wait for server to be ready
      await this.waitForServer();
      
    } catch (error) {
      throw new Error(`Failed to start test server: ${error}`);
    }
  }

  async stop(): Promise<void> {
    if (this.process) {
      return new Promise((resolve) => {
        this.process!.on('exit', () => {
          this.process = null;
          resolve();
        });
        
        this.process!.kill('SIGTERM');
        
        // Force kill if doesn't respond within 5 seconds
        setTimeout(() => {
          if (this.process) {
            this.process.kill('SIGKILL');
          }
        }, 5000);
      });
    }
  }

  async isHealthy(): Promise<boolean> {
    try {
      const response = await axios.get(`http://localhost:${this.port}/api/health`, {
        timeout: 5000
      });
      return response.status === 200 && response.data?.status;
    } catch {
      return false;
    }
  }

  getBaseUrl(): string {
    return `http://localhost:${this.port}`;
  }

  private async killExistingProcess(): Promise<void> {
    try {
      // On Windows
      if (process.platform === 'win32') {
        spawn('taskkill', ['/F', '/IM', 'rainbow-poc.exe'], { stdio: 'ignore' });
      } else {
        // On Unix-like systems
        spawn('pkill', ['-f', `serve.*--port.*${this.port}`], { stdio: 'ignore' });
      }
      
      // Wait a bit for processes to be killed
      await this.delay(2000);
    } catch (error) {
      // Ignore errors - process might not exist
    }
  }

  private async startServerProcess(): Promise<void> {
    const projectRoot = path.resolve(__dirname, '../../../..');
    const pocPath = path.join(projectRoot, 'poc-chromiumoxide');
    
    return new Promise((resolve, reject) => {
      const env = {
        ...process.env,
        RAINBOW_MOCK_MODE: 'true',
        RUST_LOG: 'info'
      };

      this.process = spawn('cargo', ['run', '--release', '--', 'serve', '--port', this.port.toString(), '--headless'], {
        cwd: pocPath,
        env,
        stdio: ['ignore', 'pipe', 'pipe']
      });

      let startupOutput = '';

      this.process.stdout?.on('data', (data) => {
        startupOutput += data.toString();
        // Look for startup indicators
        if (startupOutput.includes('Server running') || startupOutput.includes('listening')) {
          resolve();
        }
      });

      this.process.stderr?.on('data', (data) => {
        const errorOutput = data.toString();
        console.log('Server stderr:', errorOutput);
        startupOutput += errorOutput;
      });

      this.process.on('error', (error) => {
        reject(new Error(`Failed to spawn server process: ${error.message}`));
      });

      this.process.on('exit', (code, signal) => {
        if (code !== 0 && signal !== 'SIGTERM') {
          reject(new Error(`Server process exited with code ${code}, signal ${signal}`));
        }
      });

      // Timeout if server doesn't start within reasonable time
      setTimeout(() => {
        if (this.process && this.process.exitCode === null) {
          resolve(); // Assume it's starting up
        }
      }, 10000);
    });
  }

  private async waitForServer(): Promise<void> {
    for (let i = 0; i < this.maxRetries; i++) {
      if (await this.isHealthy()) {
        return;
      }
      
      console.log(`Waiting for server to be ready... (${i + 1}/${this.maxRetries})`);
      await this.delay(this.retryDelay);
    }
    
    throw new Error(`Server did not become healthy within ${this.maxRetries * this.retryDelay}ms`);
  }

  private delay(ms: number): Promise<void> {
    return new Promise(resolve => setTimeout(resolve, ms));
  }
}