// RainbowBrowserAI - Chromiumoxide Edition
// JavaScript for tools interface

// Resolve API base dynamically; default to current origin
let API_BASE = window.location.origin;

// Initialize the application
document.addEventListener('DOMContentLoaded', function() {
    console.log('RainbowBrowserAI Tools Interface Loaded');
    updateStatus('Ready');
    loadAvailableTools();
    
    // Setup navigation menu event listeners
    const navItems = document.querySelectorAll('.nav-item[data-tab]');
    navItems.forEach(item => {
        item.addEventListener('click', function(e) {
            e.preventDefault();
            const tabName = this.getAttribute('data-tab');
            switchTab(tabName);
        });
    });
    
    // Initialize perception statistics on load
    if (typeof updatePerceptionStats === 'function') {
        updatePerceptionStats();
    }

    // Ensure session + binding for visual tests launched via start.sh
    ensureSessionBinding();
});

// Helper: pick the best lightning-like view from various result shapes
function pickLightningView(result) {
    const r = result || {};
    return (
        r.lightning || r.Lightning || r.perception ||
        (r.quick ? (r.quick.lightning || r.quick) : null) ||
        (r.Quick ? (r.Quick.lightning || r.Quick) : null) ||
        (r.standard ? (r.standard.quick?.lightning || r.standard.quick || r.standard) : null) ||
        (r.Standard ? (r.Standard.quick?.lightning || r.Standard.quick || r.Standard) : null) ||
        (r.deep ? (r.deep.standard?.quick?.lightning || r.deep.standard?.quick || r.deep.standard || r.deep) : null) ||
        (r.Deep ? (r.Deep.standard?.quick?.lightning || r.Deep.standard?.quick || r.Deep.standard || r.Deep) : null) ||
        r
    );
}

// Global session tracking (kept very simple)
window.currentSessionId = window.currentSessionId || null;
window.bindToSession = window.bindToSession !== undefined ? window.bindToSession : true;
function setCurrentSession(sessionId) {
    window.currentSessionId = sessionId || null;
    const el = document.getElementById('current-session');
    if (el) {
        el.textContent = window.currentSessionId || 'None';
    }
    updateServerBanner();
}

// Ensure we have a session and binding enabled for visual tests
async function ensureSessionBinding() {
    try {
        // Respect saved settings; default to binding ON
        const saved = localStorage.getItem('rainbow-settings');
        if (saved) {
            const s = JSON.parse(saved);
            window.bindToSession = (s.bindToSession !== false);
            API_BASE = s.apiEndpoint || window.location.origin;
        } else {
            window.bindToSession = true;
            API_BASE = window.location.origin;
        }
        // Update Settings UI if present
        const bindToggle = document.getElementById('bind-session-toggle');
        if (bindToggle) bindToggle.checked = window.bindToSession;
        const apiInput = document.getElementById('api-endpoint');
        if (apiInput && !apiInput.value) apiInput.value = API_BASE;

        if (!window.currentSessionId && window.bindToSession) {
            const res = await fetch(`${API_BASE}/api/session/create`, { method: 'POST' });
            const json = await res.json();
            if (json && json.success && json.data && json.data.session_id) {
                setCurrentSession(json.data.session_id);
                showNotification(`Auto session created: ${json.data.session_id}`, 'success');
            } else {
                showNotification('Failed to auto-create session for visual test', 'warning');
            }
        }
        updateServerBanner();
    } catch (e) {
        console.warn('ensureSessionBinding error:', e);
    }
}

// Update small header banner with API endpoint and session
function updateServerBanner() {
    const ep = document.getElementById('server-endpoint');
    if (ep) ep.textContent = API_BASE;
    const sb = document.getElementById('session-badge');
    if (sb) sb.textContent = `Session: ${window.currentSessionId || 'None'}${window.bindToSession ? '' : ' (not bound)'}`;
}

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
            
            // Update tool count dynamically
            const toolCount = (data.data.summary && data.data.summary.total_tools) || 0;
            const toolsCountElement = document.getElementById('tools-count');
            if (toolsCountElement) {
                toolsCountElement.textContent = `${toolCount} Tools Available`;
            }
            
            updateStatus('Connected');
        }
    } catch (error) {
        console.error('Failed to load tools:', error);
        updateStatus('Disconnected', true);
        
        // Show error state for tool count
        const toolsCountElement = document.getElementById('tools-count');
        if (toolsCountElement) {
            toolsCountElement.textContent = 'Tools Unavailable';
        }
    }
}

// Main tool execution function
async function executeTool(toolName, parameters) {
    try {
        updateStatus('Executing...', false);
        showNotification(`Executing ${toolName}...`, 'info');
        
        const payload = {
            tool_name: toolName,
            parameters: parameters || {}
        };
        // Attach session if selected so tools operate on the same page as perception
        if (window.bindToSession && window.currentSessionId) {
            payload.session_id = window.currentSessionId;
        }

        const controller = new AbortController();
        const timer = setTimeout(() => controller.abort(), 35000);
        const response = await fetch(`${API_BASE}/api/tools/execute`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify(payload),
            signal: controller.signal
        });
        clearTimeout(timer);
        
        // Check if response is ok
        if (!response.ok) {
            const errorText = await response.text();
            let errorData;
            try {
                errorData = JSON.parse(errorText);
                throw new Error(errorData.error || `HTTP ${response.status} error`);
            } catch (parseError) {
                throw new Error(`HTTP ${response.status}: ${errorText || response.statusText}`);
            }
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

            // Track last navigated URL to help perception align with tools
            try {
                if (toolName === 'navigate_to_url' && parameters && parameters.url) {
                    window.lastNavigatedUrl = parameters.url;
                    updateServerBanner();
                }
            } catch (e) { /* noop */ }
        } else {
            const errorMsg = result?.error || 'Unknown error occurred';
            let helpMessage = '';
            
            // Provide helpful suggestions based on common errors
            if (errorMsg.includes('Element not found') || errorMsg.includes('Could not find node')) {
                helpMessage = '\n💡 Tip: Check if the element exists on the current page. Use browser dev tools (F12) to find the correct selector.';
            } else if (errorMsg.includes('timeout') || errorMsg.includes('Timeout')) {
                helpMessage = '\n💡 Tip: Try increasing timeout value or wait for page to load completely.';
            } else if (errorMsg.includes('not visible') || errorMsg.includes('not an HTMLElement')) {
                helpMessage = '\n💡 Tip: Use a more specific selector or wait for the element to become visible.';
            }
            
            showNotification(`Error: ${errorMsg}${helpMessage}`, 'error');
            displayResult(`Error: ${errorMsg}${helpMessage}`);
        }
        
        updateStatus('Ready');
        return result;
    } catch (error) {
        console.error('Tool execution failed:', error);
        
        let helpMessage = '';
        if (error.name === 'AbortError') {
            helpMessage = '\n💡 Tip: The operation exceeded 35s. Try simpler pages or increase server-side timeout via RAINBOW_TOOL_TIMEOUT_SECS.';
        } else if (error.message.includes('Element not found') || error.message.includes('Could not find node')) {
            helpMessage = '\n💡 Tip: Navigate to a webpage first (use Browse tab), then verify the element exists using F12 dev tools.';
        } else if (error.message.includes('Failed to fetch') || error.message.includes('NetworkError')) {
            helpMessage = '\n💡 Tip: Check if the server is running on the correct port.';
        }
        
        showNotification(`Failed to execute ${toolName}: ${error.message}${helpMessage}`, 'error');
        displayResult(`Error: ${error.message}${helpMessage}`);
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

async function navigateAndPerceive() {
    const url = document.getElementById('nav-url').value;
    if (!url) {
        showNotification('Please enter a URL', 'warning');
        return;
    }
    const body = { url, mode: 'lightning' };
    if (window.bindToSession && window.currentSessionId) {
        body.session_id = window.currentSessionId;
    }
    try {
        const res = await fetch(`${API_BASE}/api/navigate-perceive`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(body)
        });
        if (res.status === 404) {
            // Fallback for older servers: use /api/perception/analyze which can navigate when url is provided
            const fb = await fetch(`${API_BASE}/api/perception/analyze`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ url, session_id: body.session_id })
            });
            const fbData = await fb.json();
            if (fbData && fbData.success && fbData.data) {
                window.lastNavigatedUrl = url;
                // Render minimal lightning-like view
                renderPerceptionResult('lightning', { data: {
                    url: fbData.data.url || url,
                    title: fbData.data.title || '(unknown)',
                    ready_state: 'complete',
                    clickable_count: 0,
                    input_count: 0,
                    link_count: 0,
                    form_count: 0,
                    perception_time_ms: 0
                }});
                showNotification('Navigate + Perceive (fallback) completed', 'success');
                return;
            } else {
                showNotification(`Fallback analyze failed: ${fbData?.error || 'Unknown error'}`, 'error');
                return;
            }
        }
        const data = await res.json();
        if (data && data.success && data.data) {
            window.lastNavigatedUrl = url;
            // Reuse perception renderer for consistency
            renderPerceptionResult('lightning', { data: data.data.perception || data.data });
            showNotification('Navigate + Perceive completed', 'success');
        } else {
            showNotification(`Navigate + Perceive failed: ${data?.error || 'Unknown error'}`, 'error');
        }
    } catch (e) {
        showNotification(`Network error: ${e.message}`, 'error');
    }
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

function executeSelectOptionTool() {
    const selector = document.getElementById('select-selector').value;
    const value = document.getElementById('select-value').value;
    if (!selector || !value) {
        showNotification('Please enter both selector and option value', 'warning');
        return;
    }
    executeTool('select_option', { selector, value });
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
    const selectedNavItem = document.querySelector(`.nav-item[data-tab="${tabName}"]`);
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
            const sid = result.data.session_id;
            setCurrentSession(sid);
            showNotification(`Session created: ${sid}`, 'success');
            // Reflect in sessions UI
            const countEl = document.getElementById('session-count');
            if (countEl && !isNaN(parseInt(countEl.textContent))) {
                countEl.textContent = (parseInt(countEl.textContent) + 1).toString();
            }
            displayResult({ current_session: sid, created: true });
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
            // Update session count
            const countEl = document.getElementById('session-count');
            if (countEl) {
                countEl.textContent = Array.isArray(result.data) ? result.data.length : 0;
            }
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
        headless: document.getElementById('headless-mode')?.checked || true,
        bindToSession: document.getElementById('bind-session-toggle')?.checked || false
    };
    
    localStorage.setItem('rainbow-settings', JSON.stringify(settings));
    // Apply new API base + binding immediately
    API_BASE = settings.apiEndpoint || window.location.origin;
    window.bindToSession = settings.bindToSession === true;
    updateServerBanner();
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
        if (document.getElementById('bind-session-toggle')) {
            document.getElementById('bind-session-toggle').checked = settings.bindToSession === true;
        }
        window.bindToSession = settings.bindToSession === true;
        // Apply saved API endpoint for cross-origin setups
        API_BASE = settings.apiEndpoint || window.location.origin;
    } else {
        // Defaults if nothing saved yet
        if (document.getElementById('api-endpoint')) {
            document.getElementById('api-endpoint').value = window.location.origin;
        }
        if (document.getElementById('bind-session-toggle')) {
            document.getElementById('bind-session-toggle').checked = true;
        }
        window.bindToSession = true;
        API_BASE = window.location.origin;
    }
}

// Initialize settings on load
loadSettings();
updateServerBanner();

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
window.executeSelectOptionTool = executeSelectOptionTool;
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

// ========== Perception Functions ==========

// Perception statistics tracking
let perceptionStats = {
    elementsFound: 0,
    commandsExecuted: 0,
    formsAnalyzed: 0,
    pagesAnalyzed: 0
};

// Update perception statistics display
function updatePerceptionStats() {
    document.getElementById('elements-found').textContent = perceptionStats.elementsFound;
    document.getElementById('commands-executed').textContent = perceptionStats.commandsExecuted;
    document.getElementById('forms-analyzed').textContent = perceptionStats.formsAnalyzed;
    document.getElementById('pages-analyzed').textContent = perceptionStats.pagesAnalyzed;
}

// Analyze current page using perception
async function analyzePage() {
    const resultDiv = document.getElementById('analysis-result');
    resultDiv.innerHTML = '<div class="loading">Analyzing page...</div>';
    
    try {
        const body = {};
        if (window.bindToSession && window.currentSessionId) {
            body.session_id = window.currentSessionId;
        }
        const response = await fetch(`${API_BASE}/api/perception/analyze`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(body)
        });
        
        const data = await response.json();
        
        if (data.success && data.data) {
            perceptionStats.pagesAnalyzed++;
            updatePerceptionStats();
            
            resultDiv.innerHTML = `
                <div class="success-result">
                    <h4>Page Analysis Complete</h4>
                    <div class="analysis-details">
                        <div class="detail-item">
                            <strong>URL:</strong> ${data.data.url || 'Unknown'}
                        </div>
                        <div class="detail-item">
                            <strong>Title:</strong> ${data.data.title || 'Unknown'}
                        </div>
                        <div class="detail-item">
                            <strong>Page Type:</strong> <span class="tag">${data.data.page_type || 'Unknown'}</span>
                        </div>
                        <div class="detail-item">
                            <strong>Analysis Time:</strong> ${data.data.timestamp ? new Date(data.data.timestamp).toLocaleString() : 'Unknown'}
                        </div>
                        <div class="detail-item">
                            <strong>Semantic Intent:</strong> <span class="tag">${data.data.semantic_analysis?.intent || 'Unknown'}</span>
                        </div>
                        <div class="detail-item">
                            <strong>Entities Found:</strong> ${data.data.semantic_analysis?.entities?.length || 0}
                        </div>
                    </div>
                </div>
            `;
            showNotification('Page analysis completed successfully', 'success');
        } else {
            resultDiv.innerHTML = `<div class="error-result">Error: ${data.error || 'Analysis failed'}</div>`;
            showNotification('Page analysis failed', 'error');
        }
    } catch (error) {
        resultDiv.innerHTML = `<div class="error-result">Network error: ${error.message}</div>`;
        showNotification('Network error during page analysis', 'error');
    }
}

// Classify page type
async function classifyPage() {
    // This is part of analyzePage, so we'll just call that
    await analyzePage();
}

// Find element using natural language description
async function findElement() {
    const description = document.getElementById('element-description').value.trim();
    const resultDiv = document.getElementById('element-result');
    
    if (!description) {
        showNotification('Please enter an element description', 'warning');
        return;
    }
    
    resultDiv.innerHTML = '<div class="loading">Finding element...</div>';
    
    try {
        const body = { description };
        if (window.currentSessionId) {
            body.session_id = window.currentSessionId;
        }
        const response = await fetch(`${API_BASE}/api/perception/find`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(body)
        });
        
        const data = await response.json();
        
        if (data.success && data.data) {
            perceptionStats.elementsFound++;
            updatePerceptionStats();
            
            resultDiv.innerHTML = `
                <div class="success-result">
                    <h4>Element Found</h4>
                    <div class="element-details">
                        <div class="detail-item">
                            <strong>Selector:</strong> <code>${data.data.selector}</code>
                        </div>
                        <div class="detail-item">
                            <strong>Text:</strong> "${data.data.text || 'N/A'}"
                        </div>
                        <div class="detail-item">
                            <strong>Element Type:</strong> <span class="tag">${data.data.element_type || 'Unknown'}</span>
                        </div>
                        <div class="detail-item">
                            <strong>Confidence:</strong> <span class="confidence">${Math.round(data.data.confidence * 100)}%</span>
                        </div>
                    </div>
                </div>
            `;
            showNotification('Element found successfully', 'success');
        } else {
            resultDiv.innerHTML = `<div class="error-result">Element not found: ${data.error || 'No matching element'}</div>`;
            showNotification('Element not found', 'warning');
        }
    } catch (error) {
        resultDiv.innerHTML = `<div class="error-result">Network error: ${error.message}</div>`;
        showNotification('Network error during element search', 'error');
    }
}

// Highlight found element (placeholder - would need browser integration)
function highlightElement() {
    showNotification('Element highlighting would be implemented with browser integration', 'info');
}

// Execute intelligent command
async function executeIntelligentCommand() {
    const command = document.getElementById('intelligent-command').value.trim();
    const resultDiv = document.getElementById('command-result');
    
    if (!command) {
        showNotification('Please enter a command', 'warning');
        return;
    }
    
    resultDiv.innerHTML = '<div class="loading">Executing command...</div>';
    
    try {
        const payload = { 
            command: {
                action: 'execute',
                description: command,
                parameters: {}
            }
        };
        if (window.currentSessionId) {
            payload.session_id = window.currentSessionId;
        }
        const response = await fetch(`${API_BASE}/api/perception/command`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(payload)
        });
        
        const data = await response.json();
        
        if (data.success && data.data) {
            perceptionStats.commandsExecuted++;
            updatePerceptionStats();
            
            resultDiv.innerHTML = `
                <div class="success-result">
                    <h4>Command Executed</h4>
                    <div class="command-details">
                        <div class="detail-item">
                            <strong>Command:</strong> "${command}"
                        </div>
                        <div class="detail-item">
                            <strong>Status:</strong> <span class="tag success">${data.data.status || 'Completed'}</span>
                        </div>
                        <div class="detail-item">
                            <strong>Result:</strong> ${data.data.result || 'Success'}
                        </div>
                    </div>
                </div>
            `;
            showNotification('Command executed successfully', 'success');
        } else {
            resultDiv.innerHTML = `<div class="error-result">Command failed: ${data.error || 'Execution error'}</div>`;
            showNotification('Command execution failed', 'error');
        }
    } catch (error) {
        resultDiv.innerHTML = `<div class="error-result">Network error: ${error.message}</div>`;
        showNotification('Network error during command execution', 'error');
    }
}

// NEW: Layered Perception Functions
async function perceiveWithMode(mode) {
    const resultDiv = document.getElementById('layered-perception-result');
    resultDiv.innerHTML = `<div class="loading">Running ${mode} perception...</div>`;
    
    try {
        const body = { mode: mode };
        if (window.bindToSession && window.currentSessionId) {
            body.session_id = window.currentSessionId;
        }
        // If we know the last navigated URL, include it to pre-align browser state
        if (window.lastNavigatedUrl) {
            body.url = window.lastNavigatedUrl;
        }
        const response = await fetch(`${API_BASE}/api/perceive-mode`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(body)
        });
        
        const data = await response.json();

        // Helper to detect blank/placeholder payloads
        const payloadLooksBlank = (res) => {
            try {
                const d = res && res.data ? res.data : res;
                const u = (d.url || d.lightning?.url || d.Lightning?.url || '').toLowerCase();
                const t = (d.title || d.lightning?.title || d.Lightning?.title || '').toLowerCase();
                return u === 'about:blank' || t.includes('current session page');
            } catch { return false; }
        };

        if (data.success && data.data && payloadLooksBlank(data) && window.lastNavigatedUrl) {
            // Fallback alignment: align tool-registry browser and retry without session binding
            showNotification('Perception saw blank page; retrying alignment...', 'warning');
            try {
                await fetch(`${API_BASE}/api/tools/execute`, {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({
                        tool_name: 'navigate_to_url',
                        parameters: { url: window.lastNavigatedUrl }
                    })
                });
                const fbRes = await fetch(`${API_BASE}/api/perceive-mode`, {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({ mode, url: window.lastNavigatedUrl })
                });
                const fbJson = await fbRes.json();
                if (fbJson && fbJson.success && fbJson.data && !payloadLooksBlank(fbJson)) {
                    renderPerceptionResult(mode, fbJson);
                    return;
                }
            } catch (e) {
                console.warn('Alignment retry failed:', e);
            }
        }

        if (data.success && data.data) {
            const result = data.data;
            let html = `
                <div class="success-result">
                    <h4>${mode.charAt(0).toUpperCase() + mode.slice(1)} Perception Results</h4>
            `;
            
            // Display results based on mode
            if (mode === 'lightning' || result.lightning || result.Lightning || result.perception || result.quick || result.Quick || result.standard || result.Standard || result.deep || result.Deep) {
                const lightning = pickLightningView(result);
                html += `
                    <div class="perception-layer">
                        <h5><i class="fas fa-bolt"></i> Lightning Layer (${lightning.perception_time_ms || 0}ms)</h5>
                        <div class="detail-grid">
                            <div class="detail-item"><strong>URL:</strong> ${lightning.url}</div>
                            <div class="detail-item"><strong>Title:</strong> ${lightning.title}</div>
                            <div class="detail-item"><strong>Ready State:</strong> ${lightning.ready_state}</div>
                            <div class="detail-item"><strong>Clickable:</strong> ${lightning.clickable_count}</div>
                            <div class="detail-item"><strong>Inputs:</strong> ${lightning.input_count}</div>
                            <div class="detail-item"><strong>Links:</strong> ${lightning.link_count}</div>
                            <div class="detail-item"><strong>Forms:</strong> ${lightning.form_count}</div>
                        </div>
                    </div>
                `;
            }
            
            if (result.quick || result.interactive_elements) {
                html += `
                    <div class="perception-layer">
                        <h5><i class="fas fa-tachometer-alt"></i> Quick Layer</h5>
                        <div class="detail-item">
                            <strong>Interactive Elements:</strong> ${result.interactive_elements?.length || 0}
                        </div>
                        <div class="detail-item">
                            <strong>Text Blocks:</strong> ${result.visible_text_blocks?.length || 0}
                        </div>
                        <div class="detail-item">
                            <strong>Form Fields:</strong> ${result.form_fields?.length || 0}
                        </div>
                    </div>
                `;
            }
            
            if (result.standard || result.semantic_structure) {
                html += `
                    <div class="perception-layer">
                        <h5><i class="fas fa-brain"></i> Standard Layer</h5>
                        <div class="detail-item">
                            <strong>Semantic Analysis:</strong> Complete
                        </div>
                        <div class="detail-item">
                            <strong>Accessibility Info:</strong> Available
                        </div>
                        <div class="detail-item">
                            <strong>Performance Metrics:</strong> Collected
                        </div>
                    </div>
                `;
            }
            
            if (result.deep || result.ai_insights) {
                html += `
                    <div class="perception-layer">
                        <h5><i class="fas fa-microscope"></i> Deep Layer</h5>
                        <div class="detail-item">
                            <strong>DOM Analysis:</strong> Complete
                        </div>
                        <div class="detail-item">
                            <strong>Visual Analysis:</strong> Complete
                        </div>
                        <div class="detail-item">
                            <strong>AI Insights:</strong> Generated
                        </div>
                    </div>
                `;
            }
            
            html += '</div>';
            resultDiv.innerHTML = html;
            showNotification(`${mode} perception completed`, 'success');
        } else {
            resultDiv.innerHTML = `<div class="error-result">Perception failed: ${data.error || 'Unknown error'}</div>`;
            showNotification('Perception failed', 'error');
        }
    } catch (error) {
        resultDiv.innerHTML = `<div class="error-result">Network error: ${error.message}</div>`;
        showNotification('Network error during perception', 'error');
    }
}

function renderPerceptionResult(mode, data) {
    const resultDiv = document.getElementById('layered-perception-result');
    const result = data.data || data;
    let html = `
        <div class="success-result">
            <h4>${mode.charAt(0).toUpperCase() + mode.slice(1)} Perception Results</h4>
    `;
    if (mode === 'lightning' || result.lightning || result.Lightning || result.perception || result.quick || result.Quick || result.standard || result.Standard || result.deep || result.Deep) {
        const lightning = pickLightningView(result);
        html += `
            <div class="perception-layer">
                <h5><i class="fas fa-bolt"></i> Lightning Layer (${lightning.perception_time_ms || 0}ms)</h5>
                <div class="detail-grid">
                    <div class="detail-item"><strong>URL:</strong> ${lightning.url}</div>
                    <div class="detail-item"><strong>Title:</strong> ${lightning.title}</div>
                    <div class="detail-item"><strong>Ready State:</strong> ${lightning.ready_state}</div>
                    <div class="detail-item"><strong>Clickable:</strong> ${lightning.clickable_count}</div>
                    <div class="detail-item"><strong>Inputs:</strong> ${lightning.input_count}</div>
                    <div class="detail-item"><strong>Links:</strong> ${lightning.link_count}</div>
                    <div class="detail-item"><strong>Forms:</strong> ${lightning.form_count}</div>
                </div>
            </div>
        `;
    }
    html += '</div>';
    resultDiv.innerHTML = html;
    showNotification(`${mode} perception completed`, 'success');
}

// Quick Scan function
async function quickScan() {
    const resultDiv = document.getElementById('quick-scan-result');
    resultDiv.innerHTML = '<div class="loading">Performing quick scan...</div>';
    
    try {
        const body = {};
        if (window.bindToSession && window.currentSessionId) {
            body.session_id = window.currentSessionId;
        }
        const response = await fetch(`${API_BASE}/api/quick-scan`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(body)
        });
        
        const data = await response.json();
        
        if (data.success && data.data) {
            const scan = data.data;
            resultDiv.innerHTML = `
                <div class="success-result">
                    <h4>Quick Scan Results</h4>
                    <div class="scan-summary">
                        <div class="summary-item">
                            <i class="fas fa-mouse-pointer"></i>
                            <span>${scan.interactive_count || 0} Interactive Elements</span>
                        </div>
                        <div class="summary-item">
                            <i class="fas fa-font"></i>
                            <span>${scan.text_blocks || 0} Text Blocks</span>
                        </div>
                        <div class="summary-item">
                            <i class="fas fa-wpforms"></i>
                            <span>${scan.forms || 0} Forms</span>
                        </div>
                        <div class="summary-item">
                            <i class="fas fa-images"></i>
                            <span>${scan.images || 0} Images</span>
                        </div>
                    </div>
                    ${scan.key_elements ? `
                        <div class="key-elements">
                            <h5>Key Elements Found:</h5>
                            ${scan.key_elements.map(el => `
                                <div class="element-item">
                                    <code>${el.selector}</code> - ${el.type}
                                </div>
                            `).join('')}
                        </div>
                    ` : ''}
                </div>
            `;
            showNotification('Quick scan completed', 'success');
        } else {
            resultDiv.innerHTML = `<div class="error-result">Scan failed: ${data.error || 'Unknown error'}</div>`;
            showNotification('Quick scan failed', 'error');
        }
    } catch (error) {
        resultDiv.innerHTML = `<div class="error-result">Network error: ${error.message}</div>`;
        showNotification('Network error during scan', 'error');
    }
}

// Deep Analysis function
async function deepAnalysis() {
    const resultDiv = document.getElementById('quick-scan-result');
    resultDiv.innerHTML = '<div class="loading">Performing deep analysis (this may take a few seconds)...</div>';
    
    // Use the deep perception mode
    await perceiveWithMode('deep');
    // Copy results to quick-scan-result div
    const deepResults = document.getElementById('layered-perception-result').innerHTML;
    resultDiv.innerHTML = deepResults;
}

// Smart Element Search function
async function smartElementSearch() {
    const query = document.getElementById('smart-search-query').value.trim();
    const maxResults = parseInt(document.getElementById('smart-search-max').value) || 5;
    const resultDiv = document.getElementById('smart-search-result');
    
    if (!query) {
        showNotification('Please enter a search query', 'warning');
        return;
    }
    
    resultDiv.innerHTML = '<div class="loading">Searching for elements...</div>';
    
    try {
        const body = { 
            query: query,
            max_results: maxResults
        };
        if (window.bindToSession && window.currentSessionId) {
            body.session_id = window.currentSessionId;
        }
        const response = await fetch(`${API_BASE}/api/smart-element-search`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(body)
        });
        
        const data = await response.json();
        
        if (data.success && data.data) {
            const results = data.data.elements || [];
            resultDiv.innerHTML = `
                <div class="success-result">
                    <h4>Smart Search Results</h4>
                    <div class="search-summary">
                        Found ${results.length} element(s) matching "${query}"
                    </div>
                    ${results.length > 0 ? `
                        <div class="search-results">
                            ${results.map((el, idx) => `
                                <div class="search-result-item">
                                    <div class="result-header">
                                        <span class="result-number">#${idx + 1}</span>
                                        <span class="confidence">Confidence: ${Math.round((el.confidence || 0) * 100)}%</span>
                                    </div>
                                    <div class="result-details">
                                        <div class="detail-item">
                                            <strong>Selector:</strong> <code>${el.selector}</code>
                                        </div>
                                        <div class="detail-item">
                                            <strong>Type:</strong> ${el.element_type || 'unknown'}
                                        </div>
                                        <div class="detail-item">
                                            <strong>Text:</strong> ${el.text || '(no text)'}
                                        </div>
                                        ${el.attributes ? `
                                            <div class="detail-item">
                                                <strong>Attributes:</strong> ${JSON.stringify(el.attributes)}
                                            </div>
                                        ` : ''}
                                    </div>
                                    <button class="btn btn-sm btn-secondary" onclick="highlightElement('${el.selector}')">
                                        <i class="fas fa-highlighter"></i> Highlight
                                    </button>
                                </div>
                            `).join('')}
                        </div>
                    ` : '<p>No elements found matching your query.</p>'}
                </div>
            `;
            
            if (results.length > 0) {
                perceptionStats.elementsFound += results.length;
                updatePerceptionStats();
                showNotification(`Found ${results.length} element(s)`, 'success');
            } else {
                showNotification('No elements found', 'info');
            }
        } else {
            resultDiv.innerHTML = `<div class="error-result">Search failed: ${data.error || 'Unknown error'}</div>`;
            showNotification('Smart search failed', 'error');
        }
    } catch (error) {
        resultDiv.innerHTML = `<div class="error-result">Network error: ${error.message}</div>`;
        showNotification('Network error during search', 'error');
    }
}

// Verification: ensure perception sees the page navigated by tools using the same session
async function verifyPerceptionSeesTools() {
    const out = document.getElementById('verify-perception-result');
    out.innerHTML = '<div class="loading">Running verification...</div>';

    try {
        // Make sure binding is enabled and we have a session
        if (!window.bindToSession) {
            showNotification('Binding is OFF. Enabling it for this test.', 'info');
            window.bindToSession = true;
            const toggle = document.getElementById('bind-session-toggle');
            if (toggle) toggle.checked = true;
        }
        if (!window.currentSessionId) {
            const res = await fetch(`${API_BASE}/api/session/create`, { method: 'POST' });
            const json = await res.json();
            if (!json.success) throw new Error('Failed to create session');
            setCurrentSession(json.data.session_id);
        }

        // 1) Navigate via tools with session
        const sid = window.currentSessionId;
        const navRes = await fetch(`${API_BASE}/api/tools/execute`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
                tool_name: 'navigate_to_url',
                parameters: { url: 'https://example.com' },
                session_id: sid
            })
        });
        const navJson = await navRes.json();
        if (!navJson.success) throw new Error('Navigation tool failed: ' + (navJson.error || 'unknown'));

        // 1.1) Wait for document readyState complete (use tool to ensure same session)
        try {
            await fetch(`${API_BASE}/api/tools/execute`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                    tool_name: 'wait_for_condition',
                    parameters: { condition: 'document.readyState === "complete"', timeout: 5000 },
                    session_id: sid
                })
            });
        } catch (e) {
            // Non-fatal; continue
        }

        // Small extra wait to let page stabilize
        await new Promise(r => setTimeout(r, 1200));

        // 2) Perception with same session
        const perRes = await fetch(`${API_BASE}/api/perceive-mode`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ mode: 'quick', session_id: sid })
        });
        const perJson = await perRes.json();
        if (!perJson.success) throw new Error('Perception failed: ' + (perJson.error || 'unknown'));

        // 3) Evaluate
        const data = perJson.data || {};
        const url = data.url || '';
        const source = data.source || '';
        const pass = url.startsWith('https://example.com') && source !== 'browser_pool_fallback';

        out.innerHTML = pass
            ? `<div class="success-result"><strong>PASS</strong> — Perception observed tools page.<br>URL: ${url}<br>Source: ${source || '(session-bound)'}</div>`
            : `<div class="error-result"><strong>FAIL</strong> — Perception did not observe tools page.<br>Observed URL: ${url || '(none)'}<br>Source: ${source || '(none)'}<br>Expected URL to start with https://example.com and source != browser_pool_fallback.</div>`;

        showNotification(pass ? 'Verification PASS' : 'Verification FAIL', pass ? 'success' : 'error');
    } catch (e) {
        out.innerHTML = `<div class="error-result">Verification error: ${e.message}</div>`;
        showNotification('Verification error', 'error');
    }
}

// Analyze form
async function analyzeForm() {
    const formSelector = document.getElementById('form-selector').value.trim();
    const resultDiv = document.getElementById('form-result');
    
    resultDiv.innerHTML = '<div class="loading">Analyzing form...</div>';
    
    try {
        const payload = { 
            form_selector: formSelector || null 
        };
        if (window.currentSessionId) {
            payload.session_id = window.currentSessionId;
        }
        const response = await fetch(`${API_BASE}/api/perception/forms/analyze`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(payload)
        });
        
        const data = await response.json();
        
        if (data.success && data.data) {
            perceptionStats.formsAnalyzed++;
            updatePerceptionStats();
            
            resultDiv.innerHTML = `
                <div class="success-result">
                    <h4>Form Analysis Complete</h4>
                    <div class="form-details">
                        <div class="detail-item">
                            <strong>Form Type:</strong> <span class="tag">${data.data.form_type || 'Unknown'}</span>
                        </div>
                        <div class="detail-item">
                            <strong>Fields Found:</strong> ${data.data.fields?.length || 0}
                        </div>
                        <div class="detail-item">
                            <strong>Required Fields:</strong> ${data.data.required_fields?.length || 0}
                        </div>
                        <div class="detail-item">
                            <strong>Submit Elements:</strong> ${data.data.submit_elements?.length || 0}
                        </div>
                        <div class="detail-item">
                            <strong>Confidence:</strong> <span class="confidence">${Math.round((data.data.confidence || 0) * 100)}%</span>
                        </div>
                    </div>
                    ${data.data.fields && data.data.fields.length > 0 ? `
                        <div class="fields-list">
                            <h5>Form Fields:</h5>
                            ${data.data.fields.map(field => `
                                <div class="field-item">
                                    <code>${field.selector}</code> - ${field.field_type} 
                                    ${field.required ? '<span class="required">*</span>' : ''}
                                    ${field.label ? `(${field.label})` : ''}
                                </div>
                            `).join('')}
                        </div>
                    ` : ''}
                </div>
            `;
            showNotification('Form analysis completed successfully', 'success');
        } else {
            resultDiv.innerHTML = `<div class="error-result">Form analysis failed: ${data.error || 'No form found'}</div>`;
            showNotification('Form analysis failed', 'error');
        }
    } catch (error) {
        resultDiv.innerHTML = `<div class="error-result">Network error: ${error.message}</div>`;
        showNotification('Network error during form analysis', 'error');
    }
}

// Auto-fill form (placeholder implementation)
async function autoFillForm() {
    showNotification('Auto-fill functionality requires user profile configuration', 'info');
    const resultDiv = document.getElementById('form-result');
    resultDiv.innerHTML = `
        <div class="info-result">
            <h4>Auto-Fill Configuration Needed</h4>
            <p>To use auto-fill functionality, you need to:</p>
            <ul>
                <li>Configure user profiles with personal information</li>
                <li>Specify which profile to use for filling</li>
                <li>Analyze the form first to understand its structure</li>
            </ul>
            <p>This feature is available through the API with proper configuration.</p>
        </div>
    `;
}

// Make perception functions available globally
window.analyzePage = analyzePage;
window.classifyPage = classifyPage;
window.findElement = findElement;
window.highlightElement = highlightElement;
window.executeIntelligentCommand = executeIntelligentCommand;
window.analyzeForm = analyzeForm;
window.autoFillForm = autoFillForm;
