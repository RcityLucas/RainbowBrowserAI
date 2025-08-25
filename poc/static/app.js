// RainbowBrowserAI Dashboard Application
class RainbowDashboard {
    constructor() {
        this.apiUrl = localStorage.getItem('apiUrl') || 'http://localhost:3001';
        this.currentSession = null;
        this.autoRefresh = true;
        this.refreshInterval = null;
        this.costChart = null;
        this.eventSource = null;
        this.realTimeEnabled = true;
        
        this.init();
    }

    init() {
        this.setupEventListeners();
        this.loadSettings();
        this.checkConnection();
        this.initializeCostChart();
        this.startRealTimeUpdates();
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

        document.getElementById('real-time-updates')?.addEventListener('change', (e) => {
            this.toggleRealTimeUpdates(e.target.checked);
        });

        // Plugin management
        document.getElementById('discover-plugins')?.addEventListener('click', () => {
            this.discoverPlugins();
        });

        document.getElementById('refresh-plugins')?.addEventListener('click', () => {
            this.loadPlugins();
        });

        // Browser action buttons
        document.getElementById('scroll-top-btn')?.addEventListener('click', () => {
            this.executeScrollAction('top');
        });

        document.getElementById('scroll-bottom-btn')?.addEventListener('click', () => {
            this.executeScrollAction('bottom');
        });

        document.getElementById('scroll-page-up-btn')?.addEventListener('click', () => {
            this.executeScrollAction('page_up');
        });

        document.getElementById('scroll-page-down-btn')?.addEventListener('click', () => {
            this.executeScrollAction('page_down');
        });

        document.getElementById('refresh-page-btn')?.addEventListener('click', () => {
            this.executePageAction('refresh');
        });

        document.getElementById('go-back-btn')?.addEventListener('click', () => {
            this.executePageAction('back');
        });

        document.getElementById('go-forward-btn')?.addEventListener('click', () => {
            this.executePageAction('forward');
        });

        document.getElementById('click-element-btn')?.addEventListener('click', () => {
            this.executeElementAction('click');
        });

        document.getElementById('input-text-btn')?.addEventListener('click', () => {
            this.executeElementAction('input');
        });

        // V8.0 Advanced Scroll Actions
        document.getElementById('scroll-to-position-btn')?.addEventListener('click', () => {
            this.executeV8ScrollToPosition();
        });

        document.getElementById('scroll-to-element-btn')?.addEventListener('click', () => {
            this.executeV8ScrollToElement();
        });

        document.getElementById('scroll-smooth-test-btn')?.addEventListener('click', () => {
            this.executeV8ScrollTest(true);
        });

        document.getElementById('scroll-instant-test-btn')?.addEventListener('click', () => {
            this.executeV8ScrollTest(false);
        });

        // V8.0 Smart Navigation
        document.getElementById('nav-smart-wait-btn')?.addEventListener('click', () => {
            this.executeV8Navigation('smart');
        });

        document.getElementById('nav-preload-btn')?.addEventListener('click', () => {
            this.executeV8Navigation('preload');
        });

        document.getElementById('nav-immediate-btn')?.addEventListener('click', () => {
            this.executeV8Navigation('immediate');
        });

        // V8.0 Smart Click Actions
        document.getElementById('click-double-btn')?.addEventListener('click', () => {
            this.executeV8Click('double');
        });

        document.getElementById('click-right-btn')?.addEventListener('click', () => {
            this.executeV8Click('right');
        });

        document.getElementById('click-middle-btn')?.addEventListener('click', () => {
            this.executeV8Click('middle');
        });

        document.getElementById('click-with-ctrl-btn')?.addEventListener('click', () => {
            this.executeV8Click('left', ['ctrl']);
        });

        document.getElementById('click-with-shift-btn')?.addEventListener('click', () => {
            this.executeV8Click('left', ['shift']);
        });

        document.getElementById('click-smart-btn')?.addEventListener('click', () => {
            this.executeV8Click('smart');
        });

        // V8.0 Advanced Input
        document.getElementById('type-slow-btn')?.addEventListener('click', () => {
            this.executeV8TypeText('slow');
        });

        document.getElementById('type-instant-btn')?.addEventListener('click', () => {
            this.executeV8TypeText('instant');
        });

        document.getElementById('clear-and-type-btn')?.addEventListener('click', () => {
            this.executeV8TypeText('clear');
        });

        // V8.0 Page Analysis
        document.getElementById('analyze-lightning-btn')?.addEventListener('click', () => {
            this.executeV8Analysis('lightning');
        });

        document.getElementById('analyze-quick-btn')?.addEventListener('click', () => {
            this.executeV8Analysis('quick');
        });

        document.getElementById('analyze-standard-btn')?.addEventListener('click', () => {
            this.executeV8Analysis('standard');
        });

        document.getElementById('analyze-deep-btn')?.addEventListener('click', () => {
            this.executeV8Analysis('deep');
        });

        // V8.0 Data Extraction
        document.getElementById('extract-text-btn')?.addEventListener('click', () => {
            this.executeV8Extraction('text');
        });

        document.getElementById('extract-attribute-btn')?.addEventListener('click', () => {
            this.executeV8Extraction('attributes');
        });

        document.getElementById('extract-links-btn')?.addEventListener('click', () => {
            this.executeV8Extraction('links');
        });

        document.getElementById('extract-images-btn')?.addEventListener('click', () => {
            this.executeV8Extraction('images');
        });

        // V8.0 Wait Strategies
        document.getElementById('wait-element-btn')?.addEventListener('click', () => {
            this.executeV8Wait('element');
        });

        document.getElementById('wait-visible-btn')?.addEventListener('click', () => {
            this.executeV8Wait('visible');
        });

        document.getElementById('wait-clickable-btn')?.addEventListener('click', () => {
            this.executeV8Wait('clickable');
        });

        document.getElementById('wait-network-btn')?.addEventListener('click', () => {
            this.executeV8Wait('network');
        });

        // V8.0 Session Management
        document.getElementById('session-save-btn')?.addEventListener('click', () => {
            this.executeV8SessionAction('save');
        });

        document.getElementById('session-restore-btn')?.addEventListener('click', () => {
            this.executeV8SessionAction('restore');
        });

        document.getElementById('session-clear-cache-btn')?.addEventListener('click', () => {
            this.executeV8SessionAction('clear-cache');
        });

        document.getElementById('session-clear-cookies-btn')?.addEventListener('click', () => {
            this.executeV8SessionAction('clear-cookies');
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
            case 'plugins':
                this.loadPlugins();
                this.loadPluginMetrics();
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
                this.showNotification('OpenAI API key not configured. Please check settings or enable mock mode.', 'warning');
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
            const response = await this.apiRequest('/api/command', {
                method: 'POST',
                body: JSON.stringify({
                    command: command,
                    session_id: this.currentSession
                })
            });

            resultContainer.textContent = JSON.stringify(response, null, 2);
            
            // Check if this is a mock response
            if (response.action === 'mock') {
                this.showNotification('Mock mode: Command simulated successfully', 'info');
            } else {
                this.showNotification('Command executed successfully', 'success');
            }
            
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

        const realTimeEnabled = localStorage.getItem('realTimeEnabled') !== 'false';
        document.getElementById('real-time-updates').checked = realTimeEnabled;
        this.realTimeEnabled = realTimeEnabled;
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
        this.apiUrl = 'http://localhost:3001';
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

    // Real-time Updates via Server-Sent Events
    startRealTimeUpdates() {
        if (!this.realTimeEnabled || this.eventSource) {
            return;
        }

        try {
            this.eventSource = new EventSource(`${this.apiUrl}/events`);

            this.eventSource.addEventListener('metrics', (event) => {
                try {
                    const data = JSON.parse(event.data);
                    this.updateMetricsUI(data);
                } catch (error) {
                    console.error('Error parsing metrics event:', error);
                }
            });

            this.eventSource.addEventListener('cost', (event) => {
                try {
                    const data = JSON.parse(event.data);
                    this.updateBudgetUI(data);
                } catch (error) {
                    console.error('Error parsing cost event:', error);
                }
            });

            this.eventSource.addEventListener('plugin', (event) => {
                try {
                    const data = JSON.parse(event.data);
                    this.updatePluginMetrics(data);
                    if (data.action !== 'status') {
                        this.showNotification(`Plugin ${data.action}: ${data.plugin_name}`, 'info');
                        this.loadPlugins(); // Refresh plugin list
                    }
                } catch (error) {
                    console.error('Error parsing plugin event:', error);
                }
            });

            this.eventSource.addEventListener('heartbeat', (event) => {
                try {
                    const data = JSON.parse(event.data);
                    this.updateLastUpdateTime(data.timestamp);
                } catch (error) {
                    console.error('Error parsing heartbeat event:', error);
                }
            });

            this.eventSource.onerror = (error) => {
                console.warn('SSE connection error, falling back to polling:', error);
                this.stopRealTimeUpdates();
                this.startAutoRefresh();
            };

            this.eventSource.onopen = () => {
                console.log('Real-time updates connected');
                this.stopAutoRefresh(); // Stop polling when SSE is active
            };

        } catch (error) {
            console.error('Failed to start real-time updates:', error);
            this.startAutoRefresh(); // Fallback to polling
        }
    }

    stopRealTimeUpdates() {
        if (this.eventSource) {
            this.eventSource.close();
            this.eventSource = null;
        }
    }

    updateMetricsUI(data) {
        // Update metrics tab if it's visible
        document.getElementById('metric-operations').textContent = data.operations_total;
        document.getElementById('metric-success').textContent = `${data.success_rate.toFixed(1)}%`;
        document.getElementById('metric-response').textContent = `${data.avg_response_time_ms.toFixed(0)}ms`;
        document.getElementById('metric-browsers').textContent = data.active_browsers;
    }

    updateBudgetUI(data) {
        // Update budget display in header
        document.getElementById('budget').textContent = 
            `Budget: $${data.spent_today.toFixed(2)} / $${data.daily_budget.toFixed(2)}`;
    }

    updateLastUpdateTime(timestamp) {
        document.getElementById('last-update').textContent = 
            `Last update: ${new Date(timestamp).toLocaleTimeString()}`;
    }

    toggleRealTimeUpdates(enabled) {
        this.realTimeEnabled = enabled;
        localStorage.setItem('realTimeEnabled', enabled.toString());
        
        if (enabled) {
            this.stopAutoRefresh();
            this.startRealTimeUpdates();
        } else {
            this.stopRealTimeUpdates();
            this.startAutoRefresh();
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

    // Plugin Management Methods
    async loadPlugins() {
        try {
            const response = await this.makeRequest('/plugins', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ action: 'list' })
            });

            if (response.plugins) {
                this.displayPlugins(response.plugins);
            }
        } catch (error) {
            console.error('Error loading plugins:', error);
            this.showNotification('Failed to load plugins', 'error');
        }
    }

    async discoverPlugins() {
        try {
            this.showNotification('Discovering plugins...', 'info');
            const response = await this.makeRequest('/plugins', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ action: 'discover' })
            });

            this.showNotification(response.message, 'success');
            await this.loadPlugins(); // Refresh the list
        } catch (error) {
            console.error('Error discovering plugins:', error);
            this.showNotification('Failed to discover plugins', 'error');
        }
    }

    displayPlugins(plugins) {
        const grid = document.getElementById('plugins-grid');
        if (!grid) return;

        if (plugins.length === 0) {
            grid.innerHTML = `
                <div class="empty-state">
                    <i class="fas fa-puzzle-piece"></i>
                    <p>No plugins discovered yet</p>
                    <button class="btn btn-primary" onclick="dashboard.discoverPlugins()">
                        <i class="fas fa-search"></i> Discover Plugins
                    </button>
                </div>
            `;
            return;
        }

        grid.innerHTML = plugins.map(plugin => `
            <div class="plugin-card" data-plugin-id="${plugin.id}">
                <div class="plugin-type">${plugin.plugin_type}</div>
                <div class="plugin-header">
                    <div class="plugin-info">
                        <h4>${plugin.name}</h4>
                        <div class="version">v${plugin.version}</div>
                    </div>
                    <div class="plugin-status ${plugin.state.toLowerCase().replace(/\s+/g, '-')}">
                        <i class="fas ${this.getPluginStatusIcon(plugin.state)}"></i>
                        ${plugin.state}
                    </div>
                </div>
                <div class="plugin-description">${plugin.description}</div>
                <div class="plugin-meta">
                    ${plugin.author ? `<div class="meta-item"><i class="fas fa-user"></i> ${plugin.author}</div>` : ''}
                    ${plugin.dependencies.length > 0 ? `<div class="meta-item"><i class="fas fa-link"></i> ${plugin.dependencies.length} deps</div>` : ''}
                    ${plugin.permissions.length > 0 ? `<div class="meta-item"><i class="fas fa-shield-alt"></i> ${plugin.permissions.length} perms</div>` : ''}
                </div>
                <div class="plugin-actions">
                    ${this.getPluginActions(plugin)}
                </div>
                ${plugin.dependencies.length > 0 ? `
                    <div class="plugin-dependencies">
                        ${plugin.dependencies.map(dep => `<span class="dep-tag">${dep}</span>`).join('')}
                    </div>
                ` : ''}
            </div>
        `).join('');
    }

    getPluginStatusIcon(state) {
        const iconMap = {
            'Discovered': 'fa-eye',
            'Loading': 'fa-spinner fa-spin',
            'Loaded': 'fa-check-circle',
            'Active': 'fa-play-circle',
            'Suspended': 'fa-pause-circle',
            'Unloading': 'fa-spinner fa-spin',
            'Error': 'fa-exclamation-triangle'
        };
        return iconMap[state] || 'fa-question-circle';
    }

    getPluginActions(plugin) {
        const state = plugin.state.toLowerCase();
        let actions = [];

        if (state === 'discovered') {
            actions.push(`<button class="btn btn-primary" onclick="dashboard.loadPlugin('${plugin.id}')">
                <i class="fas fa-download"></i> Load
            </button>`);
        } else if (state === 'loaded' || state === 'active') {
            actions.push(`<button class="btn btn-warning" onclick="dashboard.unloadPlugin('${plugin.id}')">
                <i class="fas fa-stop"></i> Unload
            </button>`);
            actions.push(`<button class="btn btn-secondary" onclick="dashboard.reloadPlugin('${plugin.id}')">
                <i class="fas fa-redo"></i> Reload
            </button>`);
            actions.push(`<button class="btn btn-info" onclick="dashboard.configurePlugin('${plugin.id}', '${plugin.name}')">
                <i class="fas fa-cog"></i> Configure
            </button>`);
        } else if (state.includes('error')) {
            actions.push(`<button class="btn btn-secondary" onclick="dashboard.reloadPlugin('${plugin.id}')">
                <i class="fas fa-redo"></i> Retry
            </button>`);
        }

        return actions.join('');
    }

    async loadPlugin(pluginId) {
        try {
            this.showNotification('Loading plugin...', 'info');
            const response = await this.makeRequest('/plugins', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ action: 'load', plugin_id: pluginId })
            });

            this.showNotification(response.message, 'success');
            await this.loadPlugins();
        } catch (error) {
            console.error('Error loading plugin:', error);
            this.showNotification('Failed to load plugin', 'error');
        }
    }

    async unloadPlugin(pluginId) {
        try {
            this.showNotification('Unloading plugin...', 'info');
            const response = await this.makeRequest('/plugins', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ action: 'unload', plugin_id: pluginId })
            });

            this.showNotification(response.message, 'success');
            await this.loadPlugins();
        } catch (error) {
            console.error('Error unloading plugin:', error);
            this.showNotification('Failed to unload plugin', 'error');
        }
    }

    async reloadPlugin(pluginId) {
        try {
            this.showNotification('Reloading plugin...', 'info');
            const response = await this.makeRequest('/plugins', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ action: 'reload', plugin_id: pluginId })
            });

            this.showNotification(response.message, 'success');
            await this.loadPlugins();
        } catch (error) {
            console.error('Error reloading plugin:', error);
            this.showNotification('Failed to reload plugin', 'error');
        }
    }

    configurePlugin(pluginId, pluginName) {
        const modal = document.getElementById('plugin-config-modal');
        const nameElement = document.getElementById('config-plugin-name');
        const form = document.getElementById('plugin-config-form');
        
        nameElement.textContent = `Configure ${pluginName}`;
        modal.style.display = 'flex';
        
        // Store plugin ID for form submission
        form.dataset.pluginId = pluginId;
        
        // Set up form submission
        form.onsubmit = async (e) => {
            e.preventDefault();
            await this.savePluginConfig(pluginId);
        };
    }

    async savePluginConfig(pluginId) {
        try {
            const settingsText = document.getElementById('plugin-settings').value;
            let config;
            
            try {
                config = JSON.parse(settingsText || '{}');
            } catch (parseError) {
                this.showNotification('Invalid JSON configuration', 'error');
                return;
            }

            this.showNotification('Saving plugin configuration...', 'info');
            const response = await this.makeRequest('/plugins', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ 
                    action: 'configure', 
                    plugin_id: pluginId,
                    config: config
                })
            });

            this.showNotification(response.message, 'success');
            this.closePluginConfig();
            await this.loadPlugins();
        } catch (error) {
            console.error('Error saving plugin configuration:', error);
            this.showNotification('Failed to save plugin configuration', 'error');
        }
    }

    closePluginConfig() {
        const modal = document.getElementById('plugin-config-modal');
        const settingsTextarea = document.getElementById('plugin-settings');
        
        modal.style.display = 'none';
        settingsTextarea.value = '';
    }

    updatePluginMetrics(data) {
        // Update plugin metrics in the UI
        document.getElementById('total-plugins').textContent = data.total_plugins || 0;
        document.getElementById('active-plugins').textContent = data.active_plugins || 0;
        document.getElementById('failed-plugins').textContent = data.failed_plugins || 0;
    }

    async loadPluginMetrics() {
        try {
            const response = await this.makeRequest('/plugins/metrics');
            this.updatePluginMetrics(response);
        } catch (error) {
            console.error('Error loading plugin metrics:', error);
        }
    }

    // Browser Action Functions
    async executeScrollAction(scrollType) {
        try {
            const response = await this.apiRequest('/command', {
                method: 'POST',
                body: JSON.stringify({
                    command: `scroll_${scrollType}`,
                    session_id: this.currentSession
                })
            });

            this.showNotification(`Scroll ${scrollType} executed`, response.success ? 'success' : 'warning');
            
            // Update browser result display
            const resultContainer = document.getElementById('browser-result');
            resultContainer.innerHTML = `
                <div><strong>Action:</strong> Scroll ${scrollType}</div>
                <div><strong>Result:</strong> ${response.success ? 'Success' : 'Failed'}</div>
                <pre>${JSON.stringify(response, null, 2)}</pre>
            `;
        } catch (error) {
            this.showNotification(`Scroll action failed: ${error.message}`, 'error');
        }
    }

    async executePageAction(actionType) {
        try {
            const response = await this.apiRequest('/command', {
                method: 'POST',
                body: JSON.stringify({
                    command: `page_${actionType}`,
                    session_id: this.currentSession
                })
            });

            this.showNotification(`Page ${actionType} executed`, response.success ? 'success' : 'warning');
            
            // Update browser result display
            const resultContainer = document.getElementById('browser-result');
            resultContainer.innerHTML = `
                <div><strong>Action:</strong> Page ${actionType}</div>
                <div><strong>Result:</strong> ${response.success ? 'Success' : 'Failed'}</div>
                <pre>${JSON.stringify(response, null, 2)}</pre>
            `;
        } catch (error) {
            this.showNotification(`Page action failed: ${error.message}`, 'error');
        }
    }

    async executeElementAction(actionType) {
        try {
            const selector = document.getElementById('click-selector')?.value;
            const inputText = document.getElementById('input-text')?.value;
            
            let command;
            if (actionType === 'click') {
                if (!selector) {
                    this.showNotification('Please enter a CSS selector', 'warning');
                    return;
                }
                command = `click element with selector "${selector}"`;
            } else if (actionType === 'input') {
                if (!selector || !inputText) {
                    this.showNotification('Please enter both selector and text', 'warning');
                    return;
                }
                command = `input "${inputText}" into element with selector "${selector}"`;
            }

            const response = await this.apiRequest('/command', {
                method: 'POST',
                body: JSON.stringify({
                    command: command,
                    session_id: this.currentSession
                })
            });

            this.showNotification(`Element ${actionType} executed`, response.success ? 'success' : 'warning');
            
            // Update browser result display
            const resultContainer = document.getElementById('browser-result');
            resultContainer.innerHTML = `
                <div><strong>Action:</strong> Element ${actionType}</div>
                <div><strong>Selector:</strong> ${selector}</div>
                ${inputText ? `<div><strong>Input Text:</strong> ${inputText}</div>` : ''}
                <div><strong>Result:</strong> ${response.success ? 'Success' : 'Failed'}</div>
                <pre>${JSON.stringify(response, null, 2)}</pre>
            `;
        } catch (error) {
            this.showNotification(`Element action failed: ${error.message}`, 'error');
        }
    }

    // V8.0 Advanced Functions
    async executeV8ScrollToPosition() {
        try {
            const x = document.getElementById('scroll-to-x')?.value;
            const y = document.getElementById('scroll-to-y')?.value;
            
            if (!x || !y) {
                this.showNotification('Please enter both X and Y coordinates', 'warning');
                return;
            }

            const command = `scroll to position ${x},${y}`;
            const response = await this.apiRequest('/command', {
                method: 'POST',
                body: JSON.stringify({
                    command: command,
                    session_id: this.currentSession
                })
            });

            this.showNotification(`Scrolled to position (${x}, ${y})`, response.success ? 'success' : 'warning');
            this.updateBrowserResult('Scroll to Position', response);
        } catch (error) {
            this.showNotification(`Scroll to position failed: ${error.message}`, 'error');
        }
    }

    async executeV8ScrollToElement() {
        try {
            const elementId = document.getElementById('scroll-to-element')?.value;
            
            if (!elementId) {
                this.showNotification('Please enter an element ID', 'warning');
                return;
            }

            const command = `scroll to element with id "${elementId}"`;
            const response = await this.apiRequest('/command', {
                method: 'POST',
                body: JSON.stringify({
                    command: command,
                    session_id: this.currentSession
                })
            });

            this.showNotification(`Scrolled to element: ${elementId}`, response.success ? 'success' : 'warning');
            this.updateBrowserResult('Scroll to Element', response);
        } catch (error) {
            this.showNotification(`Scroll to element failed: ${error.message}`, 'error');
        }
    }

    async executeV8ScrollTest(smooth) {
        try {
            const command = smooth ? 
                'scroll down 500 pixels smoothly' : 
                'scroll down 500 pixels instantly';
            
            const response = await this.apiRequest('/command', {
                method: 'POST',
                body: JSON.stringify({
                    command: command,
                    session_id: this.currentSession
                })
            });

            this.showNotification(`${smooth ? 'Smooth' : 'Instant'} scroll test executed`, response.success ? 'success' : 'warning');
            this.updateBrowserResult(`${smooth ? 'Smooth' : 'Instant'} Scroll Test`, response);
        } catch (error) {
            this.showNotification(`Scroll test failed: ${error.message}`, 'error');
        }
    }

    async executeV8Navigation(mode) {
        try {
            const urlInput = document.getElementById('url-input');
            const url = urlInput?.value || 'https://example.com';
            
            let command;
            switch(mode) {
                case 'smart':
                    command = `navigate to ${url} with smart wait strategy`;
                    break;
                case 'preload':
                    command = `navigate to ${url} with preload enabled`;
                    break;
                case 'immediate':
                    command = `navigate to ${url} immediately without waiting`;
                    break;
                default:
                    command = `navigate to ${url}`;
            }

            const response = await this.apiRequest('/command', {
                method: 'POST',
                body: JSON.stringify({
                    command: command,
                    session_id: this.currentSession
                })
            });

            this.showNotification(`V8.0 ${mode} navigation executed`, response.success ? 'success' : 'warning');
            this.updateBrowserResult(`V8.0 ${mode} Navigation`, response);
        } catch (error) {
            this.showNotification(`V8.0 navigation failed: ${error.message}`, 'error');
        }
    }

    async executeV8Click(clickType, modifiers = []) {
        try {
            const selector = document.getElementById('v8-click-selector')?.value || 
                            document.getElementById('click-selector')?.value;
            
            if (!selector) {
                this.showNotification('Please enter a CSS selector', 'warning');
                return;
            }

            let command = '';
            if (clickType === 'double') {
                command = `double click element with selector "${selector}"`;
            } else if (clickType === 'right') {
                command = `right click element with selector "${selector}"`;
            } else if (clickType === 'middle') {
                command = `middle click element with selector "${selector}"`;
            } else if (clickType === 'smart') {
                command = `smart click element with selector "${selector}"`;
            } else if (modifiers.length > 0) {
                command = `click element with selector "${selector}" while holding ${modifiers.join('+')}`;
            } else {
                command = `click element with selector "${selector}"`;
            }

            const response = await this.apiRequest('/command', {
                method: 'POST',
                body: JSON.stringify({
                    command: command,
                    session_id: this.currentSession
                })
            });

            this.showNotification(`V8.0 ${clickType} click executed`, response.success ? 'success' : 'warning');
            this.updateBrowserResult(`V8.0 ${clickType} Click`, response);
        } catch (error) {
            this.showNotification(`V8.0 click failed: ${error.message}`, 'error');
        }
    }

    async executeV8TypeText(mode) {
        try {
            const selector = document.getElementById('v8-input-selector')?.value;
            const text = document.getElementById('v8-input-text')?.value;
            
            if (!selector || !text) {
                this.showNotification('Please enter both selector and text', 'warning');
                return;
            }

            let command;
            switch(mode) {
                case 'slow':
                    command = `type "${text}" slowly into element with selector "${selector}"`;
                    break;
                case 'instant':
                    command = `type "${text}" instantly into element with selector "${selector}"`;
                    break;
                case 'clear':
                    command = `clear and type "${text}" into element with selector "${selector}"`;
                    break;
                default:
                    command = `type "${text}" into element with selector "${selector}"`;
            }

            const response = await this.apiRequest('/command', {
                method: 'POST',
                body: JSON.stringify({
                    command: command,
                    session_id: this.currentSession
                })
            });

            this.showNotification(`V8.0 ${mode} type executed`, response.success ? 'success' : 'warning');
            this.updateBrowserResult(`V8.0 ${mode} Type`, response);
        } catch (error) {
            this.showNotification(`V8.0 type failed: ${error.message}`, 'error');
        }
    }

    updateBrowserResult(action, response) {
        const resultContainer = document.getElementById('browser-result');
        if (resultContainer) {
            resultContainer.innerHTML = `
                <div><strong>Action:</strong> ${action}</div>
                <div><strong>Result:</strong> ${response.success ? 'Success' : 'Failed'}</div>
                ${response.explanation ? `<div><strong>Explanation:</strong> ${response.explanation}</div>` : ''}
                <pre>${JSON.stringify(response, null, 2)}</pre>
            `;
        }
    }

    // V8.0 Page Analysis Functions
    async executeV8Analysis(level) {
        try {
            let command;
            switch(level) {
                case 'lightning':
                    command = 'analyze page with lightning speed (under 50ms)';
                    break;
                case 'quick':
                    command = 'analyze page quickly (under 200ms)';
                    break;
                case 'standard':
                    command = 'analyze page standard (under 500ms)';
                    break;
                case 'deep':
                    command = 'analyze page deeply (under 1000ms)';
                    break;
                default:
                    command = 'analyze page';
            }

            const response = await this.apiRequest('/command', {
                method: 'POST',
                body: JSON.stringify({
                    command: command,
                    session_id: this.currentSession
                })
            });

            this.showNotification(`V8.0 ${level} analysis completed`, response.success ? 'success' : 'warning');
            this.updateBrowserResult(`V8.0 ${level} Analysis`, response);
        } catch (error) {
            this.showNotification(`V8.0 analysis failed: ${error.message}`, 'error');
        }
    }

    // V8.0 Data Extraction Functions
    async executeV8Extraction(type) {
        try {
            const selector = document.getElementById('extract-selector')?.value || '*';
            
            let command;
            switch(type) {
                case 'text':
                    command = `extract text from elements matching "${selector}"`;
                    break;
                case 'attributes':
                    command = `extract attributes from elements matching "${selector}"`;
                    break;
                case 'links':
                    command = `extract all links from page`;
                    break;
                case 'images':
                    command = `extract all image sources from page`;
                    break;
                default:
                    command = `extract data from "${selector}"`;
            }

            const response = await this.apiRequest('/command', {
                method: 'POST',
                body: JSON.stringify({
                    command: command,
                    session_id: this.currentSession
                })
            });

            this.showNotification(`V8.0 ${type} extraction completed`, response.success ? 'success' : 'warning');
            this.updateBrowserResult(`V8.0 ${type} Extraction`, response);
        } catch (error) {
            this.showNotification(`V8.0 extraction failed: ${error.message}`, 'error');
        }
    }

    // V8.0 Wait Strategy Functions
    async executeV8Wait(strategy) {
        try {
            const selector = document.getElementById('wait-selector')?.value;
            const timeout = document.getElementById('wait-timeout')?.value || 5000;
            
            let command;
            switch(strategy) {
                case 'element':
                    command = `wait for element "${selector}" to appear within ${timeout}ms`;
                    break;
                case 'visible':
                    command = `wait for element "${selector}" to be visible within ${timeout}ms`;
                    break;
                case 'clickable':
                    command = `wait for element "${selector}" to be clickable within ${timeout}ms`;
                    break;
                case 'network':
                    command = `wait for network to be idle within ${timeout}ms`;
                    break;
                default:
                    command = `wait ${timeout}ms`;
            }

            const response = await this.apiRequest('/command', {
                method: 'POST',
                body: JSON.stringify({
                    command: command,
                    session_id: this.currentSession
                })
            });

            this.showNotification(`V8.0 ${strategy} wait completed`, response.success ? 'success' : 'warning');
            this.updateBrowserResult(`V8.0 ${strategy} Wait`, response);
        } catch (error) {
            this.showNotification(`V8.0 wait failed: ${error.message}`, 'error');
        }
    }

    // V8.0 Session Management Functions
    async executeV8SessionAction(action) {
        try {
            let command;
            switch(action) {
                case 'save':
                    command = 'save current browser session state';
                    break;
                case 'restore':
                    command = 'restore previous browser session state';
                    break;
                case 'clear-cache':
                    command = 'clear browser cache';
                    break;
                case 'clear-cookies':
                    command = 'clear browser cookies';
                    break;
                default:
                    command = `session ${action}`;
            }

            const response = await this.apiRequest('/command', {
                method: 'POST',
                body: JSON.stringify({
                    command: command,
                    session_id: this.currentSession
                })
            });

            this.showNotification(`V8.0 session ${action} completed`, response.success ? 'success' : 'warning');
            this.updateBrowserResult(`V8.0 Session ${action}`, response);
        } catch (error) {
            this.showNotification(`V8.0 session action failed: ${error.message}`, 'error');
        }
    }
}

// Initialize dashboard when DOM is ready
let dashboard;
document.addEventListener('DOMContentLoaded', () => {
    dashboard = new RainbowDashboard();
});

// Global functions for modal controls
function closePluginConfig() {
    if (dashboard) {
        dashboard.closePluginConfig();
    }
}

function discoverPlugins() {
    if (dashboard) {
        dashboard.discoverPlugins();
    }
}

// Add slide out animation
const style = document.createElement('style');
style.textContent = `
    @keyframes slideOut {
        from { transform: translateX(0); opacity: 1; }
        to { transform: translateX(100%); opacity: 0; }
    }
`;
document.head.appendChild(style);