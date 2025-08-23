// Production Dashboard for RainbowBrowserAI
// Integrates with Health Monitor, Error Recovery, and LLM Integration systems

class ProductionDashboard {
    constructor() {
        this.apiUrl = localStorage.getItem('apiUrl') || 'http://localhost:3000';
        this.healthStatus = 'unknown';
        this.lastHealthCheck = null;
        this.healthMetrics = null;
        this.errorRecoveryMetrics = null;
        this.llmMetrics = null;
        this.autoRefresh = true;
        this.eventSource = null;
        this.charts = {};
        
        this.init();
    }

    async init() {
        console.log('🚀 Production Dashboard initializing...');
        this.setupEventListeners();
        this.loadSettings();
        await this.checkSystemHealth();
        this.initializeCharts();
        this.startRealTimeUpdates();
        this.startPeriodicUpdates();
        console.log('✅ Production Dashboard initialized');
    }

    setupEventListeners() {
        // Health monitoring controls
        document.getElementById('force-health-check')?.addEventListener('click', () => {
            this.forceHealthCheck();
        });

        // Error recovery controls
        document.getElementById('view-error-logs')?.addEventListener('click', () => {
            this.showErrorLogs();
        });

        document.getElementById('test-error-recovery')?.addEventListener('click', () => {
            this.testErrorRecovery();
        });

        // LLM integration controls
        document.getElementById('test-llm')?.addEventListener('click', () => {
            this.testLLMIntegration();
        });

        document.getElementById('view-llm-metrics')?.addEventListener('click', () => {
            this.showLLMMetrics();
        });

        // System controls
        document.getElementById('generate-health-report')?.addEventListener('click', () => {
            this.generateHealthReport();
        });

        document.getElementById('export-diagnostics')?.addEventListener('click', () => {
            this.exportDiagnostics();
        });

        // Alert management
        document.getElementById('clear-alerts')?.addEventListener('click', () => {
            this.clearAlerts();
        });

        // Tab switching for production sections
        document.querySelectorAll('.prod-nav-item').forEach(item => {
            item.addEventListener('click', (e) => {
                e.preventDefault();
                this.switchProductionTab(item.dataset.tab);
            });
        });
    }

    // System Health Management
    async checkSystemHealth() {
        try {
            console.log('🏥 Checking system health...');
            const response = await fetch(`${this.apiUrl}/health`);
            const healthData = await response.json();
            
            this.healthStatus = healthData.overall_status;
            this.healthMetrics = healthData;
            this.lastHealthCheck = new Date();
            
            this.updateHealthDisplay();
            this.updateComponentHealth(healthData.component_health);
            
            console.log(`Health status: ${this.healthStatus}`);
        } catch (error) {
            console.error('Health check failed:', error);
            this.healthStatus = 'critical';
            this.showNotification('Health check failed', 'error');
        }
    }

    updateHealthDisplay() {
        const statusElement = document.getElementById('system-status');
        const statusIcon = document.getElementById('status-icon');
        
        if (statusElement && statusIcon) {
            const statusConfig = {
                'healthy': { text: 'Healthy', icon: 'fa-check-circle', class: 'status-healthy' },
                'warning': { text: 'Warning', icon: 'fa-exclamation-triangle', class: 'status-warning' },
                'degraded': { text: 'Degraded', icon: 'fa-exclamation-circle', class: 'status-degraded' },
                'critical': { text: 'Critical', icon: 'fa-times-circle', class: 'status-critical' },
                'down': { text: 'Down', icon: 'fa-times-circle', class: 'status-down' }
            };

            const config = statusConfig[this.healthStatus] || statusConfig['critical'];
            
            statusElement.textContent = config.text;
            statusIcon.className = `fas ${config.icon}`;
            statusElement.className = `system-status ${config.class}`;
        }

        // Update last check time
        const lastCheckElement = document.getElementById('last-health-check');
        if (lastCheckElement && this.lastHealthCheck) {
            lastCheckElement.textContent = this.lastHealthCheck.toLocaleTimeString();
        }
    }

    updateComponentHealth(componentHealth) {
        const container = document.getElementById('component-health-list');
        if (!container || !componentHealth) return;

        container.innerHTML = '';
        
        Object.values(componentHealth).forEach(component => {
            const componentElement = this.createComponentHealthCard(component);
            container.appendChild(componentElement);
        });
    }

    createComponentHealthCard(component) {
        const card = document.createElement('div');
        card.className = `component-health-card status-${component.status.toLowerCase()}`;
        
        card.innerHTML = `
            <div class="component-header">
                <h4>${component.name}</h4>
                <span class="status-badge status-${component.status.toLowerCase()}">${component.status}</span>
            </div>
            <div class="component-details">
                <p class="component-message">${component.message}</p>
                <div class="component-metrics">
                    <span>Response: ${component.response_time_ms}ms</span>
                    <span>Success: ${component.success_count}</span>
                    <span>Failures: ${component.failure_count}</span>
                </div>
                <div class="component-timestamp">
                    Last check: ${new Date(component.last_check).toLocaleTimeString()}
                </div>
            </div>
        `;
        
        return card;
    }

    // Error Recovery Management
    async loadErrorRecoveryMetrics() {
        try {
            console.log('📊 Loading error recovery metrics...');
            const response = await fetch(`${this.apiUrl}/error-recovery/metrics`);
            const metrics = await response.json();
            
            this.errorRecoveryMetrics = metrics;
            this.updateErrorRecoveryDisplay();
            
        } catch (error) {
            console.error('Failed to load error recovery metrics:', error);
        }
    }

    updateErrorRecoveryDisplay() {
        if (!this.errorRecoveryMetrics) return;

        const metrics = this.errorRecoveryMetrics;

        // Update summary metrics
        document.getElementById('total-errors').textContent = metrics.total_errors || 0;
        document.getElementById('successful-recoveries').textContent = metrics.successful_recoveries || 0;
        document.getElementById('failed-recoveries').textContent = metrics.failed_recoveries || 0;
        
        const recoveryRate = metrics.total_errors > 0 
            ? ((metrics.successful_recoveries / metrics.total_errors) * 100).toFixed(1)
            : 0;
        document.getElementById('recovery-rate').textContent = `${recoveryRate}%`;

        // Update average recovery time
        const avgTime = metrics.average_recovery_time_ms || 0;
        document.getElementById('avg-recovery-time').textContent = `${avgTime}ms`;

        // Update error patterns
        this.updateErrorPatterns(metrics.common_error_patterns);
    }

    updateErrorPatterns(patterns) {
        const container = document.getElementById('error-patterns-list');
        if (!container || !patterns) return;

        container.innerHTML = '';

        patterns.forEach(pattern => {
            const patternElement = document.createElement('div');
            patternElement.className = 'error-pattern-card';
            
            patternElement.innerHTML = `
                <div class="pattern-header">
                    <h5>${pattern.signature}</h5>
                    <span class="pattern-count">${pattern.count} occurrences</span>
                </div>
                <div class="pattern-details">
                    <span class="success-rate">Success rate: ${(pattern.success_rate * 100).toFixed(1)}%</span>
                    <span class="best-strategy">Best strategy: ${pattern.best_strategy}</span>
                    <span class="last-seen">Last seen: ${new Date(pattern.last_seen).toLocaleString()}</span>
                </div>
            `;
            
            container.appendChild(patternElement);
        });
    }

    async testErrorRecovery() {
        try {
            console.log('🧪 Testing error recovery system...');
            this.showNotification('Testing error recovery...', 'info');
            
            const response = await fetch(`${this.apiUrl}/error-recovery/test`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                    error_type: 'test_error',
                    severity: 'medium'
                })
            });
            
            const result = await response.json();
            
            if (response.ok) {
                this.showNotification(`Error recovery test completed: ${result.status}`, 'success');
            } else {
                this.showNotification(`Error recovery test failed: ${result.error}`, 'error');
            }
            
        } catch (error) {
            console.error('Error recovery test failed:', error);
            this.showNotification('Error recovery test failed', 'error');
        }
    }

    // LLM Integration Management
    async loadLLMMetrics() {
        try {
            console.log('🤖 Loading LLM integration metrics...');
            const response = await fetch(`${this.apiUrl}/llm/metrics`);
            const metrics = await response.json();
            
            this.llmMetrics = metrics;
            this.updateLLMDisplay();
            
        } catch (error) {
            console.error('Failed to load LLM metrics:', error);
        }
    }

    updateLLMDisplay() {
        if (!this.llmMetrics) return;

        const metrics = this.llmMetrics;

        // Update LLM summary metrics
        document.getElementById('llm-total-requests').textContent = metrics.total_requests || 0;
        document.getElementById('llm-successful-requests').textContent = metrics.successful_requests || 0;
        document.getElementById('llm-failed-requests').textContent = metrics.failed_requests || 0;

        const successRate = metrics.total_requests > 0
            ? ((metrics.successful_requests / metrics.total_requests) * 100).toFixed(1)
            : 0;
        document.getElementById('llm-success-rate').textContent = `${successRate}%`;

        document.getElementById('llm-avg-response-time').textContent = `${metrics.average_response_time_ms || 0}ms`;
        document.getElementById('llm-total-cost').textContent = `$${(metrics.total_cost_usd || 0).toFixed(4)}`;
        document.getElementById('llm-tokens-consumed').textContent = metrics.tokens_consumed || 0;

        // Update provider performance
        this.updateProviderPerformance(metrics.provider_performance);
    }

    updateProviderPerformance(providerPerformance) {
        const container = document.getElementById('provider-performance-list');
        if (!container || !providerPerformance) return;

        container.innerHTML = '';

        Object.entries(providerPerformance).forEach(([provider, performance]) => {
            const providerElement = document.createElement('div');
            providerElement.className = 'provider-performance-card';
            
            const reliabilityClass = performance.reliability_score > 0.8 ? 'good' : 
                                   performance.reliability_score > 0.6 ? 'warning' : 'poor';
            
            providerElement.innerHTML = `
                <div class="provider-header">
                    <h5>${provider}</h5>
                    <span class="reliability-badge ${reliabilityClass}">
                        ${(performance.reliability_score * 100).toFixed(1)}% reliable
                    </span>
                </div>
                <div class="provider-metrics">
                    <div class="metric">
                        <span class="metric-label">Requests:</span>
                        <span class="metric-value">${performance.total_requests}</span>
                    </div>
                    <div class="metric">
                        <span class="metric-label">Success:</span>
                        <span class="metric-value">${performance.successful_requests}</span>
                    </div>
                    <div class="metric">
                        <span class="metric-label">Response Time:</span>
                        <span class="metric-value">${performance.average_response_time_ms}ms</span>
                    </div>
                    <div class="metric">
                        <span class="metric-label">Cost:</span>
                        <span class="metric-value">$${performance.cost_usd.toFixed(4)}</span>
                    </div>
                </div>
            `;
            
            container.appendChild(providerElement);
        });
    }

    async testLLMIntegration() {
        try {
            console.log('🧪 Testing LLM integration...');
            this.showNotification('Testing LLM integration...', 'info');
            
            const response = await fetch(`${this.apiUrl}/llm/test`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                    prompt: 'Test LLM integration with a simple travel plan request',
                    provider: 'mock'
                })
            });
            
            const result = await response.json();
            
            if (response.ok) {
                this.showNotification(`LLM test completed successfully: ${result.confidence}% confidence`, 'success');
                console.log('LLM test result:', result);
            } else {
                this.showNotification(`LLM test failed: ${result.error}`, 'error');
            }
            
        } catch (error) {
            console.error('LLM test failed:', error);
            this.showNotification('LLM test failed', 'error');
        }
    }

    // Chart Management
    initializeCharts() {
        this.initializeHealthChart();
        this.initializeErrorRecoveryChart();
        this.initializeLLMPerformanceChart();
        this.initializeResourceUsageChart();
    }

    initializeHealthChart() {
        const canvas = document.getElementById('health-chart');
        if (!canvas) return;

        const ctx = canvas.getContext('2d');
        this.charts.health = new Chart(ctx, {
            type: 'doughnut',
            data: {
                labels: ['Healthy', 'Warning', 'Critical'],
                datasets: [{
                    data: [80, 15, 5],
                    backgroundColor: ['#28a745', '#ffc107', '#dc3545'],
                    borderWidth: 2,
                    borderColor: '#fff'
                }]
            },
            options: {
                responsive: true,
                maintainAspectRatio: false,
                plugins: {
                    legend: {
                        position: 'bottom'
                    }
                }
            }
        });
    }

    initializeErrorRecoveryChart() {
        const canvas = document.getElementById('error-recovery-chart');
        if (!canvas) return;

        const ctx = canvas.getContext('2d');
        this.charts.errorRecovery = new Chart(ctx, {
            type: 'line',
            data: {
                labels: [],
                datasets: [{
                    label: 'Recovery Success Rate',
                    data: [],
                    borderColor: '#28a745',
                    backgroundColor: 'rgba(40, 167, 69, 0.1)',
                    tension: 0.4
                }]
            },
            options: {
                responsive: true,
                maintainAspectRatio: false,
                scales: {
                    y: {
                        beginAtZero: true,
                        max: 100
                    }
                }
            }
        });
    }

    initializeLLMPerformanceChart() {
        const canvas = document.getElementById('llm-performance-chart');
        if (!canvas) return;

        const ctx = canvas.getContext('2d');
        this.charts.llmPerformance = new Chart(ctx, {
            type: 'bar',
            data: {
                labels: ['Mock', 'OpenAI', 'Anthropic', 'Gemini'],
                datasets: [{
                    label: 'Response Time (ms)',
                    data: [200, 800, 600, 750],
                    backgroundColor: ['#6c757d', '#17a2b8', '#fd7e14', '#28a745'],
                    borderWidth: 1
                }]
            },
            options: {
                responsive: true,
                maintainAspectRatio: false,
                plugins: {
                    legend: {
                        display: false
                    }
                }
            }
        });
    }

    initializeResourceUsageChart() {
        const canvas = document.getElementById('resource-usage-chart');
        if (!canvas) return;

        const ctx = canvas.getContext('2d');
        this.charts.resourceUsage = new Chart(ctx, {
            type: 'line',
            data: {
                labels: [],
                datasets: [
                    {
                        label: 'CPU Usage (%)',
                        data: [],
                        borderColor: '#dc3545',
                        backgroundColor: 'rgba(220, 53, 69, 0.1)',
                        tension: 0.4
                    },
                    {
                        label: 'Memory Usage (%)',
                        data: [],
                        borderColor: '#007bff',
                        backgroundColor: 'rgba(0, 123, 255, 0.1)',
                        tension: 0.4
                    }
                ]
            },
            options: {
                responsive: true,
                maintainAspectRatio: false,
                scales: {
                    y: {
                        beginAtZero: true,
                        max: 100
                    }
                }
            }
        });
    }

    // Real-time Updates
    startRealTimeUpdates() {
        if (!this.eventSource) {
            console.log('📡 Starting real-time updates...');
            this.eventSource = new EventSource(`${this.apiUrl}/events`);
            
            this.eventSource.onmessage = (event) => {
                try {
                    const data = JSON.parse(event.data);
                    this.handleRealTimeUpdate(data);
                } catch (error) {
                    console.error('Error parsing real-time update:', error);
                }
            };

            this.eventSource.onerror = (error) => {
                console.error('EventSource error:', error);
                this.showNotification('Real-time updates disconnected', 'warning');
            };
        }
    }

    handleRealTimeUpdate(data) {
        switch (data.type) {
            case 'health_update':
                this.healthMetrics = data.data;
                this.updateHealthDisplay();
                break;
            case 'error_recovery':
                this.loadErrorRecoveryMetrics();
                this.showNotification(`Error recovered: ${data.message}`, 'success');
                break;
            case 'llm_metrics':
                this.llmMetrics = data.data;
                this.updateLLMDisplay();
                break;
            case 'alert':
                this.showAlert(data.data);
                break;
            default:
                console.log('Unknown real-time update:', data);
        }
    }

    // Periodic Updates
    startPeriodicUpdates() {
        // Update metrics every 30 seconds
        setInterval(() => {
            this.checkSystemHealth();
            this.loadErrorRecoveryMetrics();
            this.loadLLMMetrics();
        }, 30000);
    }

    // Utility Methods
    switchProductionTab(tabName) {
        // Hide all tabs
        document.querySelectorAll('.prod-tab-content').forEach(tab => {
            tab.classList.remove('active');
        });

        // Show selected tab
        const selectedTab = document.getElementById(`${tabName}-tab`);
        if (selectedTab) {
            selectedTab.classList.add('active');
        }

        // Update navigation
        document.querySelectorAll('.prod-nav-item').forEach(item => {
            item.classList.remove('active');
        });
        document.querySelector(`[data-tab="${tabName}"]`)?.classList.add('active');
    }

    showNotification(message, type = 'info') {
        const notification = document.createElement('div');
        notification.className = `notification notification-${type}`;
        notification.innerHTML = `
            <i class="fas fa-${this.getNotificationIcon(type)}"></i>
            <span>${message}</span>
            <button class="notification-close" onclick="this.parentElement.remove()">
                <i class="fas fa-times"></i>
            </button>
        `;

        const container = document.getElementById('notifications') || document.body;
        container.appendChild(notification);

        // Auto-remove after 5 seconds
        setTimeout(() => {
            if (notification.parentElement) {
                notification.remove();
            }
        }, 5000);
    }

    getNotificationIcon(type) {
        const icons = {
            success: 'check-circle',
            error: 'exclamation-circle',
            warning: 'exclamation-triangle',
            info: 'info-circle'
        };
        return icons[type] || icons.info;
    }

    showAlert(alertData) {
        const alertElement = document.createElement('div');
        alertElement.className = `system-alert alert-${alertData.level}`;
        alertElement.innerHTML = `
            <div class="alert-header">
                <i class="fas fa-exclamation-triangle"></i>
                <strong>${alertData.level.toUpperCase()}</strong>
                <span class="alert-component">${alertData.component}</span>
                <button class="alert-close" onclick="this.parentElement.parentElement.remove()">
                    <i class="fas fa-times"></i>
                </button>
            </div>
            <div class="alert-message">${alertData.message}</div>
            <div class="alert-timestamp">${new Date(alertData.timestamp).toLocaleString()}</div>
        `;

        const alertContainer = document.getElementById('system-alerts');
        if (alertContainer) {
            alertContainer.insertBefore(alertElement, alertContainer.firstChild);
        }
    }

    loadSettings() {
        const settings = JSON.parse(localStorage.getItem('prodDashboardSettings') || '{}');
        
        this.apiUrl = settings.apiUrl || this.apiUrl;
        this.autoRefresh = settings.autoRefresh !== false;
        
        // Apply settings to UI
        const apiEndpointInput = document.getElementById('api-endpoint');
        if (apiEndpointInput) {
            apiEndpointInput.value = this.apiUrl;
        }
    }

    saveSettings() {
        const settings = {
            apiUrl: document.getElementById('api-endpoint')?.value || this.apiUrl,
            autoRefresh: this.autoRefresh
        };

        localStorage.setItem('prodDashboardSettings', JSON.stringify(settings));
        this.apiUrl = settings.apiUrl;
        
        this.showNotification('Settings saved successfully', 'success');
    }

    async generateHealthReport() {
        try {
            this.showNotification('Generating health report...', 'info');
            
            const response = await fetch(`${this.apiUrl}/health/report`, {
                method: 'POST'
            });
            
            const report = await response.json();
            
            if (response.ok) {
                this.downloadReport(report, 'health-report');
                this.showNotification('Health report generated successfully', 'success');
            } else {
                this.showNotification(`Failed to generate health report: ${report.error}`, 'error');
            }
            
        } catch (error) {
            console.error('Health report generation failed:', error);
            this.showNotification('Health report generation failed', 'error');
        }
    }

    downloadReport(data, filename) {
        const blob = new Blob([JSON.stringify(data, null, 2)], { type: 'application/json' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = `${filename}-${new Date().toISOString().split('T')[0]}.json`;
        document.body.appendChild(a);
        a.click();
        document.body.removeChild(a);
        URL.revokeObjectURL(url);
    }

    destroy() {
        if (this.eventSource) {
            this.eventSource.close();
        }
        
        Object.values(this.charts).forEach(chart => {
            if (chart) {
                chart.destroy();
            }
        });
    }
}

// Initialize dashboard when DOM is loaded
document.addEventListener('DOMContentLoaded', () => {
    window.productionDashboard = new ProductionDashboard();
    console.log('✅ Production Dashboard ready');
});

// Export for module usage
if (typeof module !== 'undefined' && module.exports) {
    module.exports = ProductionDashboard;
}