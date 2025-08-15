// RainbowBrowserAI - Browser Extension Popup Controller
// This runs in the extension popup and communicates with the content script

class RainbowBrowserAI {
    constructor() {
        this.chatContainer = document.getElementById('chatContainer');
        this.chatInput = document.getElementById('chatInput');
        this.sendBtn = document.getElementById('sendBtn');
        this.initializeEventListeners();
        this.connectToBackend();
    }

    initializeEventListeners() {
        // Send message on button click
        this.sendBtn.addEventListener('click', () => this.sendMessage());
        
        // Send message on Enter key
        this.chatInput.addEventListener('keypress', (e) => {
            if (e.key === 'Enter') {
                this.sendMessage();
            }
        });

        // Quick action buttons
        document.querySelectorAll('.quick-btn').forEach(btn => {
            btn.addEventListener('click', (e) => {
                const action = e.target.dataset.action;
                this.handleQuickAction(action);
            });
        });
    }

    async connectToBackend() {
        // Try to connect to local Rust server
        try {
            const response = await fetch('http://localhost:8888/status');
            if (response.ok) {
                console.log('Connected to RainbowBrowserAI backend');
                this.updateStatus('connected');
            }
        } catch (error) {
            console.log('Using browser-only mode');
            this.updateStatus('browser-only');
        }
    }

    async sendMessage() {
        const message = this.chatInput.value.trim();
        if (!message) return;

        // Add user message to chat
        this.addMessage(message, 'user');
        this.chatInput.value = '';

        // Process the message
        const response = await this.processCommand(message);
        
        // Add AI response to chat
        this.addMessage(response, 'ai');
    }

    async processCommand(message) {
        // Get current tab
        const [tab] = await chrome.tabs.query({ active: true, currentWindow: true });
        
        // Parse intent from message
        const intent = this.parseIntent(message);
        
        // Execute action based on intent
        switch (intent.action) {
            case 'search':
                return await this.executeSearch(tab.id, intent.params);
            
            case 'click':
                return await this.executeClick(tab.id, intent.params);
            
            case 'fill':
                return await this.executeFill(tab.id, intent.params);
            
            case 'extract':
                return await this.executeExtract(tab.id, intent.params);
            
            case 'navigate':
                return await this.executeNavigate(tab.id, intent.params);
            
            case 'screenshot':
                return await this.executeScreenshot(tab.id);
            
            default:
                return await this.executeSmartAction(tab.id, message);
        }
    }

    parseIntent(message) {
        const lowerMessage = message.toLowerCase();
        
        // Search patterns
        if (lowerMessage.includes('搜索') || lowerMessage.includes('search')) {
            const query = message.replace(/搜索|search/gi, '').trim();
            return { action: 'search', params: { query } };
        }
        
        // Click patterns
        if (lowerMessage.includes('点击') || lowerMessage.includes('click')) {
            const target = message.replace(/点击|click/gi, '').trim();
            return { action: 'click', params: { target } };
        }
        
        // Fill patterns
        if (lowerMessage.includes('填写') || lowerMessage.includes('输入') || lowerMessage.includes('fill')) {
            return { action: 'fill', params: { text: message } };
        }
        
        // Extract patterns
        if (lowerMessage.includes('提取') || lowerMessage.includes('获取') || lowerMessage.includes('extract')) {
            return { action: 'extract', params: {} };
        }
        
        // Navigation patterns
        if (lowerMessage.includes('打开') || lowerMessage.includes('访问') || lowerMessage.includes('go to')) {
            const url = message.replace(/打开|访问|go to/gi, '').trim();
            return { action: 'navigate', params: { url } };
        }
        
        // Screenshot patterns
        if (lowerMessage.includes('截图') || lowerMessage.includes('screenshot')) {
            return { action: 'screenshot', params: {} };
        }
        
        // Default: smart action
        return { action: 'smart', params: { message } };
    }

    async executeSearch(tabId, params) {
        // Send message to content script to perform search
        const response = await chrome.tabs.sendMessage(tabId, {
            action: 'search',
            query: params.query
        });
        
        if (response && response.success) {
            return `✅ 搜索 "${params.query}" 完成！找到 ${response.results || 0} 个结果。`;
        } else {
            return `❌ 搜索失败，请重试。`;
        }
    }

    async executeClick(tabId, params) {
        const response = await chrome.tabs.sendMessage(tabId, {
            action: 'click',
            selector: params.target
        });
        
        if (response && response.success) {
            return `✅ 已点击 "${params.target}"`;
        } else {
            return `❌ 未找到可点击的元素 "${params.target}"`;
        }
    }

    async executeFill(tabId, params) {
        const response = await chrome.tabs.sendMessage(tabId, {
            action: 'fill',
            text: params.text
        });
        
        if (response && response.success) {
            return `✅ 表单填写完成`;
        } else {
            return `❌ 未找到可填写的表单`;
        }
    }

    async executeExtract(tabId, params) {
        const response = await chrome.tabs.sendMessage(tabId, {
            action: 'extract'
        });
        
        if (response && response.data) {
            return `✅ 已提取数据：\n${JSON.stringify(response.data, null, 2)}`;
        } else {
            return `❌ 无法提取数据`;
        }
    }

    async executeNavigate(tabId, params) {
        let url = params.url;
        
        // Add protocol if missing
        if (!url.startsWith('http://') && !url.startsWith('https://')) {
            url = 'https://' + url;
        }
        
        await chrome.tabs.update(tabId, { url });
        return `✅ 正在导航到 ${url}`;
    }

    async executeScreenshot(tabId) {
        try {
            const dataUrl = await chrome.tabs.captureVisibleTab();
            
            // Create download link
            const link = document.createElement('a');
            link.href = dataUrl;
            link.download = `rainbow-screenshot-${Date.now()}.png`;
            link.click();
            
            return `✅ 截图已保存`;
        } catch (error) {
            return `❌ 截图失败: ${error.message}`;
        }
    }

    async executeSmartAction(tabId, message) {
        // Send to content script for smart processing
        const response = await chrome.tabs.sendMessage(tabId, {
            action: 'smart',
            message: message
        });
        
        if (response && response.result) {
            return response.result;
        } else {
            // Fallback to simple interpretation
            return this.interpretMessage(message);
        }
    }

    interpretMessage(message) {
        // Simple rule-based responses for common requests
        const responses = {
            '你好': '👋 你好！有什么可以帮助你的吗？',
            'hello': '👋 Hello! How can I help you?',
            '帮助': '我可以帮你：\n• 🔍 搜索信息\n• 📋 提取数据\n• ✍️ 填写表单\n• 🖱️ 点击元素\n• 📸 截图保存',
            'help': 'I can help you:\n• 🔍 Search\n• 📋 Extract data\n• ✍️ Fill forms\n• 🖱️ Click elements\n• 📸 Take screenshots',
        };
        
        // Check for matching response
        for (const [key, value] of Object.entries(responses)) {
            if (message.toLowerCase().includes(key)) {
                return value;
            }
        }
        
        // Default response
        return `🤖 正在处理: "${message}"\n请稍候...`;
    }

    addMessage(text, sender) {
        const messageDiv = document.createElement('div');
        messageDiv.className = `message ${sender}-message`;
        messageDiv.textContent = text;
        this.chatContainer.appendChild(messageDiv);
        
        // Scroll to bottom
        this.chatContainer.scrollTop = this.chatContainer.scrollHeight;
    }

    handleQuickAction(action) {
        const actions = {
            'search': '搜索网页内容',
            'extract': '提取页面数据',
            'fill': '自动填写表单',
            'screenshot': '截图当前页面'
        };
        
        const message = actions[action] || action;
        this.chatInput.value = message;
        this.sendMessage();
    }

    updateStatus(status) {
        const statusText = document.getElementById('modeText');
        if (status === 'connected') {
            statusText.textContent = 'AI云端模式';
        } else {
            statusText.textContent = '本地智能模式';
        }
    }
}

// Initialize when popup loads
document.addEventListener('DOMContentLoaded', () => {
    new RainbowBrowserAI();
});