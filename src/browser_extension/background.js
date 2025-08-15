// RainbowBrowserAI - Background Service Worker
// Handles extension lifecycle, commands, and communication

// Configuration
const SERVER_URL = 'http://localhost:8888';
let isServerConnected = false;
let browserOnlyMode = true;

// Check server connection on startup
async function checkServerConnection() {
    try {
        const response = await fetch(`${SERVER_URL}/health`, {
            method: 'GET',
            mode: 'cors'
        });
        
        if (response.ok) {
            const data = await response.json();
            isServerConnected = data.status === 'healthy';
            browserOnlyMode = !isServerConnected;
            console.log('🌈 Server connected:', isServerConnected);
        }
    } catch (error) {
        console.log('🌈 Running in browser-only mode');
        isServerConnected = false;
        browserOnlyMode = true;
    }
}

// Initialize on install
chrome.runtime.onInstalled.addListener(async (details) => {
    console.log('🌈 RainbowBrowserAI installed:', details);
    
    // Check server connection
    await checkServerConnection();
    
    // Create context menu items
    chrome.contextMenus.create({
        id: 'rainbow-extract',
        title: '🌈 Extract with AI',
        contexts: ['selection', 'page']
    });
    
    chrome.contextMenus.create({
        id: 'rainbow-analyze',
        title: '🌈 Analyze with AI',
        contexts: ['image', 'link', 'selection']
    });
    
    // Set initial badge
    chrome.action.setBadgeText({ text: 'AI' });
    chrome.action.setBadgeBackgroundColor({ color: '#667eea' });
});

// Handle keyboard shortcut
chrome.commands.onCommand.addListener((command) => {
    if (command === 'activate-ai') {
        // Get current tab and inject UI
        chrome.tabs.query({ active: true, currentWindow: true }, (tabs) => {
            if (tabs[0]) {
                chrome.tabs.sendMessage(tabs[0].id, {
                    action: 'toggle-assistant'
                });
            }
        });
    }
});

// Handle context menu clicks
chrome.contextMenus.onClicked.addListener((info, tab) => {
    switch (info.menuItemId) {
        case 'rainbow-extract':
            handleExtraction(info, tab);
            break;
        case 'rainbow-analyze':
            handleAnalysis(info, tab);
            break;
    }
});

// Handle messages from content scripts and popup
chrome.runtime.onMessage.addListener((request, sender, sendResponse) => {
    console.log('🌈 Background received:', request);
    
    switch (request.type) {
        case 'process-ai':
            processAIRequest(request.data).then(sendResponse);
            return true; // Will respond asynchronously
            
        case 'check-status':
            sendResponse({
                serverConnected: isServerConnected,
                browserOnly: browserOnlyMode
            });
            break;
            
        case 'execute-on-tab':
            executeOnActiveTab(request.action, request.data).then(sendResponse);
            return true;
            
        default:
            sendResponse({ error: 'Unknown request type' });
    }
});

// Process AI requests
async function processAIRequest(data) {
    if (isServerConnected) {
        // Send to server
        try {
            const response = await fetch(`${SERVER_URL}/ai`, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify(data)
            });
            
            if (response.ok) {
                return await response.json();
            }
        } catch (error) {
            console.error('Server error:', error);
        }
    }
    
    // Fallback to browser-only processing
    return processBrowserOnly(data);
}

// Browser-only AI simulation
function processBrowserOnly(data) {
    const { action, message } = data;
    
    // Simulate intelligent processing
    const responses = {
        search: {
            success: true,
            result: `Searching for: ${message}`,
            instructions: [
                { type: 'find', selector: 'input[type="search"]' },
                { type: 'fill', value: message },
                { type: 'submit' }
            ]
        },
        navigate: {
            success: true,
            result: `Navigating to: ${message}`,
            instructions: [
                { type: 'navigate', url: message }
            ]
        },
        click: {
            success: true,
            result: `Clicking: ${message}`,
            instructions: [
                { type: 'click', text: message }
            ]
        },
        extract: {
            success: true,
            result: 'Extracting page data',
            instructions: [
                { type: 'extract', target: 'all' }
            ]
        },
        fill: {
            success: true,
            result: `Filling form with: ${message}`,
            instructions: [
                { type: 'fill', data: message }
            ]
        }
    };
    
    // Smart pattern matching
    const lowerMessage = message.toLowerCase();
    
    if (lowerMessage.includes('search') || lowerMessage.includes('搜索')) {
        return responses.search;
    } else if (lowerMessage.includes('go to') || lowerMessage.includes('打开')) {
        return responses.navigate;
    } else if (lowerMessage.includes('click') || lowerMessage.includes('点击')) {
        return responses.click;
    } else if (lowerMessage.includes('extract') || lowerMessage.includes('提取')) {
        return responses.extract;
    } else if (lowerMessage.includes('fill') || lowerMessage.includes('填写')) {
        return responses.fill;
    }
    
    // Default response
    return {
        success: true,
        result: `Processing: ${message}`,
        instructions: [
            { type: 'smart', action: message }
        ]
    };
}

// Execute action on active tab
async function executeOnActiveTab(action, data) {
    const [tab] = await chrome.tabs.query({ active: true, currentWindow: true });
    
    if (!tab) {
        return { success: false, error: 'No active tab' };
    }
    
    // Send message to content script
    try {
        const response = await chrome.tabs.sendMessage(tab.id, {
            action: action,
            ...data
        });
        return response;
    } catch (error) {
        // Content script might not be loaded, inject it
        await chrome.scripting.executeScript({
            target: { tabId: tab.id },
            files: ['content.js']
        });
        
        // Try again
        return await chrome.tabs.sendMessage(tab.id, {
            action: action,
            ...data
        });
    }
}

// Handle extraction from context menu
async function handleExtraction(info, tab) {
    const message = {
        action: 'extract',
        context: {
            selectionText: info.selectionText,
            pageUrl: info.pageUrl
        }
    };
    
    chrome.tabs.sendMessage(tab.id, message);
}

// Handle analysis from context menu
async function handleAnalysis(info, tab) {
    const message = {
        action: 'analyze',
        context: {
            selectionText: info.selectionText,
            linkUrl: info.linkUrl,
            srcUrl: info.srcUrl,
            pageUrl: info.pageUrl
        }
    };
    
    chrome.tabs.sendMessage(tab.id, message);
}

// Periodic server health check
setInterval(checkServerConnection, 30000); // Check every 30 seconds

// Initial setup
checkServerConnection();

console.log('🌈 RainbowBrowserAI background service worker loaded');