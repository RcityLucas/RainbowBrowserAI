// RainbowBrowserAI - Chromiumoxide Edition
// JavaScript for tools interface

// Use the current origin and port (makes it work regardless of port)
const API_BASE = window.location.origin;

// Initialize the application
document.addEventListener('DOMContentLoaded', function() {
    console.log('RainbowBrowserAI Tools Interface Loaded');
    updateStatus('Ready');
    loadAvailableTools();
});

// Update connection status
function updateStatus(status, isError = false) {
    const statusElement = document.getElementById('connection-status');
    if (statusElement) {
        statusElement.textContent = status;
        statusElement.className = isError ? 'status-error' : 'status-ok';
    }
}

// Show notification
function showNotification(message, type = 'info') {
    const notificationsContainer = document.querySelector('.notifications') || createNotificationsContainer();
    
    const notification = document.createElement('div');
    notification.className = `notification ${type}`;
    notification.innerHTML = `
        <i class="fas fa-${getIconForType(type)}"></i>
        <span>${message}</span>
    `;
    
    notificationsContainer.appendChild(notification);
    
    // Auto-remove after 5 seconds
    setTimeout(() => {
        notification.remove();
    }, 5000);
}

function createNotificationsContainer() {
    const container = document.createElement('div');
    container.className = 'notifications';
    document.body.appendChild(container);
    return container;
}

function getIconForType(type) {
    switch(type) {
        case 'success': return 'check-circle';
        case 'error': return 'exclamation-circle';
        case 'warning': return 'exclamation-triangle';
        default: return 'info-circle';
    }
}

// Display result in the output container
function displayResult(result, containerId = 'tools-output') {
    const container = document.getElementById(containerId);
    if (!container) return;
    
    const resultDiv = container.querySelector('.result-container');
    if (resultDiv) {
        if (typeof result === 'object') {
            resultDiv.textContent = JSON.stringify(result, null, 2);
        } else {
            resultDiv.textContent = result;
        }
    }
}

// Load available tools from API
async function loadAvailableTools() {
    try {
        const response = await fetch(`${API_BASE}/api/tools`);
        const data = await response.json();
        
        if (data.success && data.data) {
            console.log('Available tools loaded:', data.data);
            updateStatus('Connected');
        }
    } catch (error) {
        console.error('Failed to load tools:', error);
        updateStatus('Disconnected', true);
    }
}

// Main tool execution function
async function executeTool(toolName, parameters) {
    try {
        updateStatus('Executing...', false);
        showNotification(`Executing ${toolName}...`, 'info');
        
        const response = await fetch(`${API_BASE}/api/tools/execute`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify({
                tool_name: toolName,
                parameters: parameters || {}
            })
        });
        
        // Check if response is ok
        if (!response.ok) {
            const errorText = await response.text();
            throw new Error(`HTTP ${response.status}: ${errorText || response.statusText}`);
        }
        
        // Try to parse JSON
        let result;
        const contentType = response.headers.get('content-type');
        if (contentType && contentType.includes('application/json')) {
            result = await response.json();
        } else {
            const text = await response.text();
            throw new Error(`Expected JSON but got: ${text}`);
        }
        
        if (result && result.success) {
            showNotification(`${toolName} executed successfully`, 'success');
            displayResult(result.data);
        } else {
            const errorMsg = result?.error || 'Unknown error occurred';
            showNotification(`Error: ${errorMsg}`, 'error');
            displayResult(`Error: ${errorMsg}`);
        }
        
        updateStatus('Ready');
        return result;
    } catch (error) {
        console.error('Tool execution failed:', error);
        showNotification(`Failed to execute ${toolName}: ${error.message}`, 'error');
        displayResult(`Error: ${error.message}`);
        updateStatus('Error', true);
        throw error;
    }
}

// Navigation tools
function executeNavigateTool() {
    const url = document.getElementById('nav-url').value;
    if (!url) {
        showNotification('Please enter a URL', 'warning');
        return;
    }
    executeTool('navigate_to_url', { url });
}

// Quick navigation helper
function quickNavigate(url) {
    executeTool('navigate_to_url', { url });
}

// Refresh page helper
function refreshPage() {
    executeTool('refresh', {});
}

// Quick screenshot helper
function quickScreenshot() {
    executeTool('screenshot', { full_page: false }).then(result => {
        if (result && result.data && result.data.screenshot) {
            // Display screenshot preview
            const preview = document.getElementById('screenshot-preview');
            if (preview) {
                preview.innerHTML = `<img src="data:image/png;base64,${result.data.screenshot}" style="max-width: 100%; height: auto;">`;
            }
        }
    });
}

function executeScrollTool() {
    const x = parseInt(document.getElementById('scroll-x').value) || 0;
    const y = parseInt(document.getElementById('scroll-y').value) || 0;
    executeTool('scroll', { x, y });
}

// Interaction tools
function executeClickTool() {
    const selector = document.getElementById('click-selector').value;
    if (!selector) {
        showNotification('Please enter a CSS selector', 'warning');
        return;
    }
    executeTool('click', { selector });
}

function executeTypeTool() {
    const selector = document.getElementById('type-selector').value;
    const text = document.getElementById('type-text').value;
    if (!selector || !text) {
        showNotification('Please enter both selector and text', 'warning');
        return;
    }
    executeTool('type_text', { selector, text });
}

function executeHoverTool() {
    const selector = document.getElementById('hover-selector').value;
    if (!selector) {
        showNotification('Please enter a CSS selector', 'warning');
        return;
    }
    executeTool('hover', { selector });
}

function executeFocusTool() {
    const selector = document.getElementById('hover-selector').value;
    if (!selector) {
        showNotification('Please enter a CSS selector', 'warning');
        return;
    }
    executeTool('focus', { selector });
}

// Data extraction tools
function executeExtractTextTool() {
    const selector = document.getElementById('extract-text-selector').value;
    if (!selector) {
        showNotification('Please enter a CSS selector', 'warning');
        return;
    }
    executeTool('extract_text', { selector });
}

function executeExtractLinksTool() {
    const selector = document.getElementById('extract-links-selector').value || 'a';
    executeTool('extract_links', { selector });
}

function executeExtractDataTool() {
    const selector = document.getElementById('extract-data-selector').value;
    const attributes = document.getElementById('extract-data-attributes').value;
    if (!selector) {
        showNotification('Please enter a CSS selector', 'warning');
        return;
    }
    
    const attrArray = attributes ? attributes.split(',').map(s => s.trim()) : [];
    executeTool('extract_data', { selector, attributes: attrArray });
}

// Synchronization tools
function executeWaitForElementTool() {
    const selector = document.getElementById('wait-selector').value;
    const timeout = parseInt(document.getElementById('wait-timeout').value) || 5000;
    if (!selector) {
        showNotification('Please enter a CSS selector', 'warning');
        return;
    }
    executeTool('wait_for_element', { selector, timeout });
}

// Memory tools
function executeScreenshotTool() {
    const fullPage = document.getElementById('screenshot-full').checked;
    executeTool('screenshot', { full_page: fullPage });
}

function executeSessionMemoryTool() {
    const action = document.getElementById('session-action').value;
    const key = document.getElementById('session-key').value;
    const value = document.getElementById('session-value').value;
    
    const params = { action };
    if (key) params.key = key;
    if (value) params.value = value;
    
    executeTool('session_memory', params);
}

// Tab switching
function switchTab(tabName) {
    // Hide all tab contents
    const tabContents = document.querySelectorAll('.tab-content');
    tabContents.forEach(content => {
        content.classList.remove('active');
    });
    
    // Remove active class from all nav items
    const navItems = document.querySelectorAll('.nav-item');
    navItems.forEach(item => {
        item.classList.remove('active');
    });
    
    // Show selected tab content
    const selectedTab = document.getElementById(`${tabName}-tab`);
    if (selectedTab) {
        selectedTab.classList.add('active');
    }
    
    // Add active class to selected nav item
    const selectedNavItem = document.querySelector(`.nav-item[onclick*="${tabName}"]`);
    if (selectedNavItem) {
        selectedNavItem.classList.add('active');
    }
}

// Additional tool functions for new interface
function executeSelectTool() {
    const selector = document.getElementById('select-selector').value;
    const value = document.getElementById('select-value').value;
    if (!selector || !value) {
        showNotification('Please enter selector and value', 'warning');
        return;
    }
    executeTool('select_option', { selector, value });
}

function executeExtractTableTool() {
    const selector = document.getElementById('extract-table-selector').value || 'table';
    executeTool('extract_table', { selector });
}

function executeExtractFormTool() {
    const selector = document.getElementById('extract-form-selector').value || 'form';
    executeTool('extract_form', { selector });
}

function executeWaitForConditionTool() {
    const condition = document.getElementById('wait-condition').value;
    const timeout = parseInt(document.getElementById('condition-timeout').value) || 5000;
    if (!condition) {
        showNotification('Please enter a condition', 'warning');
        return;
    }
    executeTool('wait_for_condition', { condition, timeout });
}

function executeElementInfoTool() {
    const selector = document.getElementById('element-info-selector').value;
    if (!selector) {
        showNotification('Please enter a selector', 'warning');
        return;
    }
    executeTool('get_element_info', { selector });
}

function executeHistoryTool() {
    const action = document.getElementById('history-action').value;
    executeTool('history_tracker', { action });
}

function executeCacheTool() {
    const action = document.getElementById('cache-action').value;
    const key = document.getElementById('cache-key').value;
    const value = document.getElementById('cache-value').value;
    
    const params = { action };
    if (key) params.key = key;
    if (value) params.value = value;
    
    executeTool('persistent_cache', params);
}

// Clear output
function clearOutput() {
    const output = document.querySelector('.result-container');
    if (output) {
        output.textContent = 'Output cleared.';
    }
}

// Session management
async function createSession() {
    try {
        const response = await fetch(`${API_BASE}/api/session/create`, {
            method: 'POST'
        });
        const result = await response.json();
        
        if (result.success) {
            showNotification(`Session created: ${result.data.session_id}`, 'success');
            displayResult(result.data);
        } else {
            showNotification(`Failed to create session: ${result.error}`, 'error');
        }
    } catch (error) {
        console.error('Failed to create session:', error);
        showNotification(`Error: ${error.message}`, 'error');
    }
}

async function listSessions() {
    try {
        const response = await fetch(`${API_BASE}/api/sessions`);
        const result = await response.json();
        
        if (result.success) {
            displayResult(result.data);
        } else {
            showNotification(`Failed to list sessions: ${result.error}`, 'error');
        }
    } catch (error) {
        console.error('Failed to list sessions:', error);
        showNotification(`Error: ${error.message}`, 'error');
    }
}

// Settings management
function saveSettings() {
    const settings = {
        apiEndpoint: document.getElementById('api-endpoint')?.value || API_BASE,
        timeout: parseInt(document.getElementById('timeout')?.value) || 30000,
        headless: document.getElementById('headless-mode')?.checked || true
    };
    
    localStorage.setItem('rainbow-settings', JSON.stringify(settings));
    showNotification('Settings saved successfully', 'success');
}

function loadSettings() {
    const saved = localStorage.getItem('rainbow-settings');
    if (saved) {
        const settings = JSON.parse(saved);
        
        if (document.getElementById('api-endpoint')) {
            document.getElementById('api-endpoint').value = settings.apiEndpoint || API_BASE;
        }
        if (document.getElementById('timeout')) {
            document.getElementById('timeout').value = settings.timeout || 30000;
        }
        if (document.getElementById('headless-mode')) {
            document.getElementById('headless-mode').checked = settings.headless !== false;
        }
    }
}

// Initialize settings on load
loadSettings();

// Test all tools function
async function testAllTools() {
    showNotification('Starting comprehensive test...', 'info');
    const testSummary = document.getElementById('test-summary');
    if (testSummary) {
        testSummary.style.display = 'block';
    }
    
    let totalTests = 0;
    let passedTests = 0;
    let failedTests = 0;
    
    const tools = [
        // Navigation Tools (5)
        { name: 'navigate_to_url', params: { url: 'https://example.com' }, category: 'Navigation' },
        { name: 'scroll', params: { x: 0, y: 100 }, category: 'Navigation' },
        { name: 'refresh', params: {}, category: 'Navigation' },
        { name: 'go_back', params: {}, category: 'Navigation' },
        { name: 'go_forward', params: {}, category: 'Navigation' },
        
        // Interaction Tools (5)  
        { name: 'click', params: { selector: 'a' }, category: 'Interaction' },
        { name: 'type_text', params: { selector: 'input', text: 'test' }, category: 'Interaction' },
        { name: 'hover', params: { selector: 'body' }, category: 'Interaction' },
        { name: 'focus', params: { selector: 'body' }, category: 'Interaction' },
        { name: 'select_option', params: { selector: 'select', value: 'test' }, category: 'Interaction' },
        
        // Extraction Tools (5)
        { name: 'extract_text', params: { selector: 'h1' }, category: 'Extraction' },
        { name: 'extract_links', params: { selector: 'a' }, category: 'Extraction' },
        { name: 'extract_data', params: { selector: 'img', attributes: ['src', 'alt'] }, category: 'Extraction' },
        { name: 'extract_table', params: { selector: 'table' }, category: 'Extraction' },
        { name: 'extract_form', params: { selector: 'form' }, category: 'Extraction' },
        
        // Synchronization Tools (2)
        { name: 'wait_for_element', params: { selector: 'body', timeout: 1000 }, category: 'Synchronization' },
        { name: 'wait_for_condition', params: { condition: 'document.readyState === "complete"', timeout: 1000 }, category: 'Synchronization' },
        
        // Memory & Sync Tools (5)
        { name: 'screenshot', params: {}, category: 'Memory' },
        { name: 'session_memory', params: { action: 'get' }, category: 'Memory' },
        { name: 'get_element_info', params: { selector: 'body' }, category: 'Memory' },
        { name: 'history_tracker', params: { action: 'get' }, category: 'Memory' },
        { name: 'persistent_cache', params: { action: 'get' }, category: 'Memory' }
    ];
    
    for (const tool of tools) {
        totalTests++;
        try {
            console.log(`🧪 Testing ${tool.category}: ${tool.name}`);
            const result = await executeTool(tool.name, tool.params);
            if (result && result.success) {
                passedTests++;
                console.log(`✅ ${tool.category}/${tool.name} passed`);
            } else {
                failedTests++;
                console.log(`❌ ${tool.category}/${tool.name} failed:`, result?.error || 'Unknown error');
            }
        } catch (error) {
            failedTests++;
            console.error(`❌ ${tool.category}/${tool.name} error:`, error);
        }
        
        // Update test stats and show progress
        if (document.getElementById('total-tests')) {
            document.getElementById('total-tests').textContent = totalTests;
            document.getElementById('passed-tests').textContent = passedTests;
            document.getElementById('failed-tests').textContent = failedTests;
        }
        
        // Add a small delay to avoid overwhelming the browser
        await new Promise(resolve => setTimeout(resolve, 500));
    }
    
    const message = `Test complete: ${passedTests}/${totalTests} passed`;
    showNotification(message, failedTests === 0 ? 'success' : 'warning');
}

// Test scenarios
async function runScenario(scenario) {
    showNotification(`Running ${scenario} scenario...`, 'info');
    
    switch(scenario) {
        case 'search':
            await executeTool('navigate_to_url', { url: 'https://google.com' });
            await executeTool('wait_for_element', { selector: 'input[name="q"]', timeout: 5000 });
            await executeTool('type_text', { selector: 'input[name="q"]', text: 'RainbowBrowserAI test' });
            showNotification('Search scenario complete', 'success');
            break;
            
        case 'form':
            await executeTool('navigate_to_url', { url: 'https://example.com' });
            await executeTool('extract_form', { selector: 'form' });
            showNotification('Form scenario complete', 'success');
            break;
            
        case 'navigation':
            await executeTool('navigate_to_url', { url: 'https://example.com' });
            await executeTool('navigate_to_url', { url: 'https://example.org' });
            await executeTool('go_back', {});
            await executeTool('go_forward', {});
            await executeTool('refresh', {});
            showNotification('Navigation scenario complete', 'success');
            break;
            
        case 'extraction':
            await executeTool('navigate_to_url', { url: 'https://example.com' });
            await executeTool('extract_text', { selector: 'h1' });
            await executeTool('extract_links', { selector: 'a' });
            await executeTool('get_element_info', { selector: 'body' });
            showNotification('Extraction scenario complete', 'success');
            break;
            
        default:
            showNotification('Unknown scenario', 'error');
    }
}

// Export for use in HTML
window.executeTool = executeTool;
window.executeNavigateTool = executeNavigateTool;
window.executeScrollTool = executeScrollTool;
window.executeClickTool = executeClickTool;
window.executeTypeTool = executeTypeTool;
window.executeHoverTool = executeHoverTool;
window.executeFocusTool = executeFocusTool;
window.executeExtractTextTool = executeExtractTextTool;
window.executeExtractLinksTool = executeExtractLinksTool;
window.executeExtractDataTool = executeExtractDataTool;
window.executeWaitForElementTool = executeWaitForElementTool;
window.executeScreenshotTool = executeScreenshotTool;
window.executeSessionMemoryTool = executeSessionMemoryTool;
window.switchTab = switchTab;
window.createSession = createSession;
window.listSessions = listSessions;
window.saveSettings = saveSettings;
window.loadSettings = loadSettings;
window.quickNavigate = quickNavigate;
window.refreshPage = refreshPage;
window.quickScreenshot = quickScreenshot;
window.executeSelectTool = executeSelectTool;
window.executeExtractTableTool = executeExtractTableTool;
window.executeExtractFormTool = executeExtractFormTool;
window.executeWaitForConditionTool = executeWaitForConditionTool;
window.executeElementInfoTool = executeElementInfoTool;
window.executeHistoryTool = executeHistoryTool;
window.executeCacheTool = executeCacheTool;
window.clearOutput = clearOutput;
window.testAllTools = testAllTools;
window.runScenario = runScenario;