// RainbowBrowserAI Dashboard Application
class RainbowDashboard {
    constructor() {
        this.apiUrl = localStorage.getItem('apiUrl') || 'http://localhost:3000';
        this.currentSession = null;
        this.autoRefresh = true;
        this.refreshInterval = null;
        this.costChart = null;
        
        this.init();
    }

    init() {
        this.setupEventListeners();
        this.loadSettings();
        this.checkConnection();
        this.startAutoRefresh();
        this.initializeCostChart();
    }

    setupEventListeners() {
        // Tab navigation
        document.querySelectorAll('.nav-item').forEach(item => {
            item.addEventListener('click', (e) => {
                e.preventDefault();
                this.switchTab(item.dataset.tab);
            });
        });

        // Command interface
        document.getElementById('execute-btn')?.addEventListener('click', () => {
            this.executeCommand();
        });

        document.getElementById('command-input')?.addEventListener('keypress', (e) => {
            if (e.key === 'Enter') {
                this.executeCommand();
            }
        });

        // Browser controls
        document.getElementById('navigate-btn')?.addEventListener('click', () => {
            this.navigateToUrl();
        });

        document.getElementById('screenshot-btn')?.addEventListener('click', () => {
            this.takeScreenshot();
        });

        document.getElementById('url-input')?.addEventListener('keypress', (e) => {
            if (e.key === 'Enter') {
                this.navigateToUrl();
            }
        });

        // Workflow controls
        document.getElementById('run-workflow')?.addEventListener('click', () => {
            this.runWorkflow();
        });

        document.getElementById('load-template')?.addEventListener('click', () => {
            this.loadWorkflowTemplate();
        });

        // Session management
        document.getElementById('new-session')?.addEventListener('click', () => {
            this.createSession();
        });

        document.getElementById('refresh-sessions')?.addEventListener('click', () => {
            this.refreshSessions();
        });

        // Settings
        document.getElementById('save-settings')?.addEventListener('click', () => {
            this.saveSettings();
        });

        document.getElementById('reset-settings')?.addEventListener('click', () => {
            this.resetSettings();
        });

        document.getElementById('dark-mode')?.addEventListener('change', (e) => {
            this.toggleDarkMode(e.target.checked);
        });

        document.getElementById('auto-refresh')?.addEventListener('change', (e) => {
            this.toggleAutoRefresh(e.target.checked);
        });
    }

    // Tab Management
    switchTab(tabName) {
        // Update navigation
        document.querySelectorAll('.nav-item').forEach(item => {
            item.classList.remove('active');
        });
        document.querySelector(`[data-tab="${tabName}"]`)?.classList.add('active');

        // Update content
        document.querySelectorAll('.tab-content').forEach(content => {
            content.classList.remove('active');
        });
        document.getElementById(`${tabName}-tab`)?.classList.add('active');

        // Load tab-specific data
        switch(tabName) {
            case 'sessions':
                this.refreshSessions();
                break;
            case 'metrics':
                this.refreshMetrics();
                break;
        }
    }

    // API Communication
    async apiRequest(endpoint, options = {}) {
        try {
            const response = await fetch(`${this.apiUrl}${endpoint}`, {
                ...options,
                headers: {
                    'Content-Type': 'application/json',
                    ...options.headers
                }
            });

            if (!response.ok) {
                const error = await response.json();
                throw new Error(error.error || `API Error: ${response.status}`);
            }

            return await response.json();
        } catch (error) {
            // Check for specific error types
            if (error.message.includes('OpenAI API key not configured')) {
                this.showNotification('OpenAI API key not configured. Please check settings.', 'warning');
            } else if (error.message.includes('Invalid OpenAI API key')) {
                this.showNotification('Invalid OpenAI API key. Please update in settings.', 'error');
            } else {
                this.showNotification(`API Error: ${error.message}`, 'error');
            }
            throw error;
        }
    }

    // Connection Management
    async checkConnection() {
        try {
            const health = await this.apiRequest('/health');
            this.updateConnectionStatus(true, health);
            await this.updateBudget();
        } catch (error) {
            this.updateConnectionStatus(false);
        }
    }

    updateConnectionStatus(connected, health = null) {
        const statusElement = document.getElementById('status');
        const connectionElement = document.getElementById('connection-status');
        
        if (connected) {
            statusElement.innerHTML = '<i class="fas fa-circle"></i> <span>Connected</span>';
            statusElement.classList.remove('offline');
            connectionElement.textContent = 'Connected';
            
            if (health) {
                document.getElementById('status').innerHTML = 
                    `<i class="fas fa-circle"></i> <span>v${health.version} • ${health.active_sessions} sessions</span>`;
            }
        } else {
            statusElement.innerHTML = '<i class="fas fa-circle"></i> <span>Disconnected</span>';
            statusElement.classList.add('offline');
            connectionElement.textContent = 'Disconnected';
        }
    }

    async updateBudget() {
        try {
            const cost = await this.apiRequest('/cost');
            document.getElementById('budget').textContent = 
                `Budget: $${cost.spent_today.toFixed(2)} / $${cost.daily_budget.toFixed(2)}`;
            
            // Update cost chart
            if (this.costChart && cost.operations) {
                this.updateCostChart(cost.operations);
            }
        } catch (error) {
            console.error('Failed to update budget:', error);
        }
    }

    // Command Execution
    async executeCommand() {
        const input = document.getElementById('command-input');
        const command = input.value.trim();
        
        if (!command) {
            this.showNotification('Please enter a command', 'warning');
            return;
        }

        const resultContainer = document.getElementById('command-result');
        resultContainer.textContent = 'Executing command...';

        try {
            const response = await this.apiRequest('/command', {
                method: 'POST',
                body: JSON.stringify({
                    command: command,
                    session_id: this.currentSession
                })
            });

            resultContainer.textContent = JSON.stringify(response, null, 2);
            this.showNotification('Command executed successfully', 'success');
            
            // Clear input
            input.value = '';
            
            // Update budget after operation
            await this.updateBudget();
        } catch (error) {
            resultContainer.textContent = `Error: ${error.message}`;
        }
    }

    // Browser Control
    async navigateToUrl() {
        const urlInput = document.getElementById('url-input');
        const url = urlInput.value.trim();
        
        if (!url) {
            this.showNotification('Please enter a URL', 'warning');
            return;
        }

        const resultContainer = document.getElementById('browser-result');
        resultContainer.innerHTML = '<div class="empty-state"><i class="fas fa-spinner fa-spin"></i><p>Navigating...</p></div>';

        try {
            const takeScreenshot = document.getElementById('take-screenshot')?.checked;
            const response = await this.apiRequest('/navigate', {
                method: 'POST',
                body: JSON.stringify({
                    url: url,
                    screenshot: takeScreenshot,
                    session_id: this.currentSession
                })
            });

            let html = `<div><strong>Title:</strong> ${response.title || 'N/A'}</div>`;
            if (response.screenshot_path) {
                html += `<div><strong>Screenshot:</strong> ${response.screenshot_path}</div>`;
                html += `<img src="${this.apiUrl}/${response.screenshot_path}" style="max-width: 100%; margin-top: 1rem;">`;
            }
            
            resultContainer.innerHTML = html;
            this.showNotification('Navigation successful', 'success');
        } catch (error) {
            resultContainer.innerHTML = `<div class="empty-state"><i class="fas fa-exclamation-triangle"></i><p>Error: ${error.message}</p></div>`;
        }
    }

    async takeScreenshot() {
        const urlInput = document.getElementById('url-input');
        const url = urlInput.value.trim();
        
        const resultContainer = document.getElementById('browser-result');
        resultContainer.innerHTML = '<div class="empty-state"><i class="fas fa-spinner fa-spin"></i><p>Taking screenshot...</p></div>';

        try {
            const fullPage = document.getElementById('full-page')?.checked;
            const width = document.getElementById('viewport-width')?.value;
            const height = document.getElementById('viewport-height')?.value;
            
            const response = await this.apiRequest('/screenshot', {
                method: 'POST',
                body: JSON.stringify({
                    url: url || undefined,
                    full_page: fullPage,
                    width: parseInt(width),
                    height: parseInt(height),
                    session_id: this.currentSession
                })
            });

            resultContainer.innerHTML = `
                <div><strong>Screenshot saved:</strong> ${response.path}</div>
                <div><strong>URL:</strong> ${response.url}</div>
                <img src="${this.apiUrl}/${response.path}" style="max-width: 100%; margin-top: 1rem;">
            `;
            
            this.showNotification('Screenshot captured', 'success');
        } catch (error) {
            resultContainer.innerHTML = `<div class="empty-state"><i class="fas fa-exclamation-triangle"></i><p>Error: ${error.message}</p></div>`;
        }
    }

    // Workflow Management
    async runWorkflow() {
        const yamlContent = document.getElementById('workflow-yaml')?.value;
        const resultContainer = document.getElementById('workflow-result');
        
        if (!yamlContent) {
            this.showNotification('Please enter a workflow', 'warning');
            return;
        }

        resultContainer.textContent = 'Running workflow...';

        try {
            // Parse YAML/JSON
            let workflow;
            try {
                workflow = JSON.parse(yamlContent);
            } catch {
                // If not JSON, assume it's YAML and send as-is
                // The server will handle YAML parsing
                workflow = { yaml: yamlContent };
            }

            const response = await this.apiRequest('/workflow', {
                method: 'POST',
                body: JSON.stringify({ workflow })
            });

            resultContainer.textContent = JSON.stringify(response, null, 2);
            this.showNotification('Workflow executed successfully', 'success');
        } catch (error) {
            resultContainer.textContent = `Error: ${error.message}`;
        }
    }

    loadWorkflowTemplate() {
        const template = {
            name: "example-workflow",
            description: "Example multi-step workflow",
            steps: [
                {
                    name: "navigate-github",
                    action: {
                        type: "navigate",
                        url: "https://github.com"
                    }
                },
                {
                    name: "wait",
                    action: {
                        type: "wait",
                        seconds: 2
                    }
                },
                {
                    name: "screenshot",
                    action: {
                        type: "screenshot",
                        filename: "github.png",
                        full_page: true
                    }
                }
            ]
        };

        document.getElementById('workflow-yaml').value = JSON.stringify(template, null, 2);
        this.showNotification('Template loaded', 'info');
    }

    // Session Management
    async createSession() {
        try {
            const response = await this.apiRequest('/session', {
                method: 'POST',
                body: JSON.stringify({ action: 'create' })
            });

            this.currentSession = response.session_id;
            this.showNotification(`Session created: ${response.session_id}`, 'success');
            await this.refreshSessions();
        } catch (error) {
            console.error('Failed to create session:', error);
        }
    }

    async refreshSessions() {
        try {
            const response = await this.apiRequest('/session', {
                method: 'POST',
                body: JSON.stringify({ action: 'list' })
            });

            const tbody = document.getElementById('sessions-tbody');
            
            if (response.sessions && response.sessions.length > 0) {
                tbody.innerHTML = response.sessions.map(session => `
                    <tr>
                        <td>${session.id}</td>
                        <td>${new Date(session.created_at).toLocaleString()}</td>
                        <td>${new Date(session.last_used).toLocaleString()}</td>
                        <td>
                            <button class="btn btn-sm" onclick="dashboard.useSession('${session.id}')">Use</button>
                            <button class="btn btn-sm" onclick="dashboard.destroySession('${session.id}')">Destroy</button>
                        </td>
                    </tr>
                `).join('');
            } else {
                tbody.innerHTML = '<tr><td colspan="4" class="empty-message">No active sessions</td></tr>';
            }
        } catch (error) {
            console.error('Failed to refresh sessions:', error);
        }
    }

    async useSession(sessionId) {
        this.currentSession = sessionId;
        this.showNotification(`Using session: ${sessionId}`, 'info');
        document.getElementById('use-session').checked = true;
    }

    async destroySession(sessionId) {
        try {
            await this.apiRequest('/session', {
                method: 'POST',
                body: JSON.stringify({
                    action: 'destroy',
                    session_id: sessionId
                })
            });

            if (this.currentSession === sessionId) {
                this.currentSession = null;
            }

            this.showNotification('Session destroyed', 'success');
            await this.refreshSessions();
        } catch (error) {
            console.error('Failed to destroy session:', error);
        }
    }

    // Metrics
    async refreshMetrics() {
        try {
            const metrics = await this.apiRequest('/metrics');
            
            document.getElementById('metric-operations').textContent = metrics.operations_total;
            document.getElementById('metric-success').textContent = `${metrics.success_rate.toFixed(1)}%`;
            document.getElementById('metric-response').textContent = `${metrics.avg_response_time_ms.toFixed(0)}ms`;
            document.getElementById('metric-browsers').textContent = metrics.active_browsers;
            
            document.getElementById('last-update').textContent = 
                `Last update: ${new Date().toLocaleTimeString()}`;
        } catch (error) {
            console.error('Failed to refresh metrics:', error);
        }
    }

    // Cost Chart
    initializeCostChart() {
        const ctx = document.getElementById('cost-canvas');
        if (!ctx) return;

        this.costChart = new Chart(ctx, {
            type: 'line',
            data: {
                labels: [],
                datasets: [{
                    label: 'Cost ($)',
                    data: [],
                    borderColor: '#667eea',
                    backgroundColor: 'rgba(102, 126, 234, 0.1)',
                    tension: 0.4
                }]
            },
            options: {
                responsive: true,
                maintainAspectRatio: false,
                plugins: {
                    legend: {
                        display: false
                    }
                },
                scales: {
                    y: {
                        beginAtZero: true,
                        ticks: {
                            callback: function(value) {
                                return '$' + value.toFixed(3);
                            }
                        }
                    }
                }
            }
        });
    }

    updateCostChart(operations) {
        if (!this.costChart) return;

        const last20 = operations.slice(-20);
        const labels = last20.map(op => new Date(op.timestamp).toLocaleTimeString());
        const data = last20.map(op => op.cost);

        this.costChart.data.labels = labels;
        this.costChart.data.datasets[0].data = data;
        this.costChart.update();
    }

    // Settings
    loadSettings() {
        const apiEndpoint = document.getElementById('api-endpoint');
        if (apiEndpoint) {
            apiEndpoint.value = this.apiUrl;
        }

        const openaiApiKey = document.getElementById('openai-api-key');
        if (openaiApiKey) {
            openaiApiKey.value = localStorage.getItem('openaiApiKey') || '';
        }

        const darkMode = localStorage.getItem('darkMode') === 'true';
        document.getElementById('dark-mode').checked = darkMode;
        if (darkMode) {
            document.body.classList.add('dark-mode');
        }

        const autoRefresh = localStorage.getItem('autoRefresh') !== 'false';
        document.getElementById('auto-refresh').checked = autoRefresh;
        this.autoRefresh = autoRefresh;
    }

    saveSettings() {
        const apiEndpoint = document.getElementById('api-endpoint').value;
        this.apiUrl = apiEndpoint;
        localStorage.setItem('apiUrl', apiEndpoint);

        const openaiApiKey = document.getElementById('openai-api-key').value;
        if (openaiApiKey) {
            localStorage.setItem('openaiApiKey', openaiApiKey);
            // Note: In a production environment, the API key should be sent to the server
            // and stored securely, not in localStorage
        }

        this.showNotification('Settings saved', 'success');
        this.checkConnection();
    }

    resetSettings() {
        localStorage.clear();
        this.apiUrl = 'http://localhost:3000';
        this.loadSettings();
        this.showNotification('Settings reset to default', 'info');
    }

    toggleDarkMode(enabled) {
        if (enabled) {
            document.body.classList.add('dark-mode');
            localStorage.setItem('darkMode', 'true');
        } else {
            document.body.classList.remove('dark-mode');
            localStorage.setItem('darkMode', 'false');
        }
    }

    toggleAutoRefresh(enabled) {
        this.autoRefresh = enabled;
        localStorage.setItem('autoRefresh', enabled.toString());
        
        if (enabled) {
            this.startAutoRefresh();
        } else {
            this.stopAutoRefresh();
        }
    }

    startAutoRefresh() {
        if (!this.autoRefresh) return;
        
        this.stopAutoRefresh();
        this.refreshInterval = setInterval(() => {
            this.checkConnection();
            
            // Refresh current tab data
            const activeTab = document.querySelector('.nav-item.active')?.dataset.tab;
            if (activeTab === 'metrics') {
                this.refreshMetrics();
            }
        }, 5000);
    }

    stopAutoRefresh() {
        if (this.refreshInterval) {
            clearInterval(this.refreshInterval);
            this.refreshInterval = null;
        }
    }

    // Notifications
    showNotification(message, type = 'info') {
        const container = document.getElementById('notifications');
        const notification = document.createElement('div');
        notification.className = `notification ${type}`;
        
        const icon = {
            success: 'check-circle',
            error: 'exclamation-circle',
            warning: 'exclamation-triangle',
            info: 'info-circle'
        }[type];

        notification.innerHTML = `
            <i class="fas fa-${icon}"></i>
            <span>${message}</span>
        `;

        container.appendChild(notification);

        setTimeout(() => {
            notification.style.animation = 'slideOut 0.3s ease';
            setTimeout(() => notification.remove(), 300);
        }, 3000);
    }
}

// Initialize dashboard when DOM is ready
let dashboard;
document.addEventListener('DOMContentLoaded', () => {
    dashboard = new RainbowDashboard();
});

// Add slide out animation
const style = document.createElement('style');
style.textContent = `
    @keyframes slideOut {
        from { transform: translateX(0); opacity: 1; }
        to { transform: translateX(100%); opacity: 0; }
    }
`;
document.head.appendChild(style);