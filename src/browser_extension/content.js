// RainbowBrowserAI - Content Script
// This runs on every web page and can interact with the DOM

class RainbowBrowserContent {
    constructor() {
        this.setupMessageListener();
        this.injectFloatingAssistant();
    }

    setupMessageListener() {
        // Listen for messages from the popup or background script
        chrome.runtime.onMessage.addListener((request, sender, sendResponse) => {
            console.log('RainbowBrowserAI: Received command', request);
            
            switch (request.action) {
                case 'search':
                    this.performSearch(request.query).then(sendResponse);
                    break;
                
                case 'click':
                    this.performClick(request.selector).then(sendResponse);
                    break;
                
                case 'fill':
                    this.performFill(request.text).then(sendResponse);
                    break;
                
                case 'extract':
                    this.performExtract().then(sendResponse);
                    break;
                
                case 'smart':
                    this.performSmartAction(request.message).then(sendResponse);
                    break;
                
                default:
                    sendResponse({ success: false, error: 'Unknown action' });
            }
            
            // Return true to indicate async response
            return true;
        });
    }

    injectFloatingAssistant() {
        // Create floating button
        const assistant = document.createElement('div');
        assistant.id = 'rainbow-assistant-button';
        assistant.innerHTML = '🌈';
        assistant.style.cssText = `
            position: fixed;
            bottom: 20px;
            right: 20px;
            width: 60px;
            height: 60px;
            border-radius: 50%;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            display: flex;
            align-items: center;
            justify-content: center;
            font-size: 30px;
            cursor: pointer;
            box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
            z-index: 999999;
            transition: all 0.3s ease;
        `;
        
        // Add hover effect
        assistant.addEventListener('mouseenter', () => {
            assistant.style.transform = 'scale(1.1)';
        });
        
        assistant.addEventListener('mouseleave', () => {
            assistant.style.transform = 'scale(1)';
        });
        
        // Add click handler
        assistant.addEventListener('click', () => {
            this.toggleAssistantPanel();
        });
        
        document.body.appendChild(assistant);
    }

    toggleAssistantPanel() {
        let panel = document.getElementById('rainbow-assistant-panel');
        
        if (panel) {
            // Toggle visibility
            panel.style.display = panel.style.display === 'none' ? 'block' : 'none';
        } else {
            // Create panel
            this.createAssistantPanel();
        }
    }

    createAssistantPanel() {
        const panel = document.createElement('div');
        panel.id = 'rainbow-assistant-panel';
        panel.style.cssText = `
            position: fixed;
            bottom: 90px;
            right: 20px;
            width: 350px;
            height: 500px;
            background: white;
            border-radius: 12px;
            box-shadow: 0 10px 40px rgba(0, 0, 0, 0.2);
            z-index: 999998;
            display: flex;
            flex-direction: column;
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
        `;
        
        panel.innerHTML = `
            <div style="padding: 15px; background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); color: white; border-radius: 12px 12px 0 0;">
                <h3 style="margin: 0; font-size: 18px;">🌈 RainbowBrowserAI</h3>
                <p style="margin: 5px 0 0 0; font-size: 12px; opacity: 0.9;">智能浏览器助手</p>
            </div>
            <div id="rainbow-chat-messages" style="flex: 1; overflow-y: auto; padding: 15px;">
                <div style="background: #f0f0f0; padding: 10px; border-radius: 8px; margin-bottom: 10px;">
                    👋 我可以帮你操作这个网页，试试这些命令：<br>
                    • "点击登录按钮"<br>
                    • "填写表单"<br>
                    • "提取所有链接"<br>
                    • "搜索关键词"
                </div>
            </div>
            <div style="padding: 15px; border-top: 1px solid #e0e0e0;">
                <input id="rainbow-chat-input" type="text" placeholder="输入命令..." style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 8px; font-size: 14px;">
            </div>
        `;
        
        document.body.appendChild(panel);
        
        // Setup input handler
        const input = document.getElementById('rainbow-chat-input');
        input.addEventListener('keypress', async (e) => {
            if (e.key === 'Enter') {
                const message = input.value.trim();
                if (message) {
                    this.addChatMessage(message, 'user');
                    input.value = '';
                    
                    const response = await this.performSmartAction(message);
                    this.addChatMessage(response.result || '完成！', 'ai');
                }
            }
        });
    }

    addChatMessage(text, sender) {
        const messagesContainer = document.getElementById('rainbow-chat-messages');
        if (!messagesContainer) return;
        
        const message = document.createElement('div');
        message.style.cssText = `
            padding: 10px;
            margin: 5px 0;
            border-radius: 8px;
            ${sender === 'user' 
                ? 'background: #667eea; color: white; text-align: right;' 
                : 'background: #f0f0f0; color: #333;'}
        `;
        message.textContent = text;
        messagesContainer.appendChild(message);
        
        // Scroll to bottom
        messagesContainer.scrollTop = messagesContainer.scrollHeight;
    }

    async performSearch(query) {
        try {
            // Find search input on the page
            const searchInputs = document.querySelectorAll('input[type="search"], input[type="text"], input[name*="search"], input[name*="q"], input[placeholder*="搜索"], input[placeholder*="Search"]');
            
            if (searchInputs.length > 0) {
                const input = searchInputs[0];
                input.value = query;
                input.dispatchEvent(new Event('input', { bubbles: true }));
                
                // Try to submit the form
                const form = input.closest('form');
                if (form) {
                    form.submit();
                } else {
                    // Trigger Enter key
                    const enterEvent = new KeyboardEvent('keypress', { key: 'Enter', keyCode: 13 });
                    input.dispatchEvent(enterEvent);
                }
                
                return { success: true, results: searchInputs.length };
            }
            
            // Fallback: Use Ctrl+F browser search
            document.execCommand('find');
            return { success: true, message: 'Opened browser search' };
            
        } catch (error) {
            return { success: false, error: error.message };
        }
    }

    async performClick(selector) {
        try {
            // Try to find element by text content first
            const elements = Array.from(document.querySelectorAll('a, button, input[type="submit"], input[type="button"], [role="button"]'));
            
            let target = elements.find(el => 
                el.textContent.toLowerCase().includes(selector.toLowerCase()) ||
                el.getAttribute('aria-label')?.toLowerCase().includes(selector.toLowerCase()) ||
                el.getAttribute('title')?.toLowerCase().includes(selector.toLowerCase())
            );
            
            // If not found by text, try CSS selector
            if (!target) {
                try {
                    target = document.querySelector(selector);
                } catch (e) {
                    // Invalid selector, ignore
                }
            }
            
            if (target) {
                // Scroll into view
                target.scrollIntoView({ behavior: 'smooth', block: 'center' });
                
                // Highlight element
                const originalStyle = target.style.cssText;
                target.style.cssText += 'outline: 3px solid #667eea !important; outline-offset: 2px !important;';
                
                // Click after a short delay
                setTimeout(() => {
                    target.click();
                    target.style.cssText = originalStyle;
                }, 500);
                
                return { success: true };
            }
            
            return { success: false, error: 'Element not found' };
            
        } catch (error) {
            return { success: false, error: error.message };
        }
    }

    async performFill(text) {
        try {
            // Find all input fields
            const inputs = document.querySelectorAll('input[type="text"], input[type="email"], input[type="tel"], textarea');
            
            if (inputs.length > 0) {
                // Smart fill based on input type
                inputs.forEach(input => {
                    const type = input.type;
                    const name = input.name?.toLowerCase() || '';
                    const placeholder = input.placeholder?.toLowerCase() || '';
                    
                    // Smart matching
                    if (name.includes('email') || placeholder.includes('email')) {
                        input.value = 'user@example.com';
                    } else if (name.includes('phone') || placeholder.includes('phone')) {
                        input.value = '13800138000';
                    } else if (name.includes('name') || placeholder.includes('name')) {
                        input.value = '张三';
                    } else {
                        input.value = text;
                    }
                    
                    input.dispatchEvent(new Event('input', { bubbles: true }));
                });
                
                return { success: true };
            }
            
            return { success: false, error: 'No input fields found' };
            
        } catch (error) {
            return { success: false, error: error.message };
        }
    }

    async performExtract() {
        try {
            const data = {
                title: document.title,
                url: window.location.href,
                headings: Array.from(document.querySelectorAll('h1, h2, h3')).map(h => h.textContent.trim()).slice(0, 5),
                links: Array.from(document.querySelectorAll('a')).map(a => ({
                    text: a.textContent.trim(),
                    href: a.href
                })).slice(0, 10),
                images: Array.from(document.querySelectorAll('img')).map(img => img.src).slice(0, 5)
            };
            
            return { success: true, data };
            
        } catch (error) {
            return { success: false, error: error.message };
        }
    }

    async performSmartAction(message) {
        const lowerMessage = message.toLowerCase();
        
        // Smart action mapping
        if (lowerMessage.includes('登录') || lowerMessage.includes('login')) {
            return await this.performClick('登录');
        }
        
        if (lowerMessage.includes('注册') || lowerMessage.includes('signup')) {
            return await this.performClick('注册');
        }
        
        if (lowerMessage.includes('下一页') || lowerMessage.includes('next')) {
            return await this.performClick('next');
        }
        
        if (lowerMessage.includes('滚动到底部') || lowerMessage.includes('scroll down')) {
            window.scrollTo(0, document.body.scrollHeight);
            return { success: true, result: '已滚动到页面底部' };
        }
        
        if (lowerMessage.includes('滚动到顶部') || lowerMessage.includes('scroll up')) {
            window.scrollTo(0, 0);
            return { success: true, result: '已滚动到页面顶部' };
        }
        
        if (lowerMessage.includes('刷新') || lowerMessage.includes('refresh')) {
            window.location.reload();
            return { success: true, result: '页面刷新中...' };
        }
        
        if (lowerMessage.includes('后退') || lowerMessage.includes('back')) {
            window.history.back();
            return { success: true, result: '返回上一页' };
        }
        
        // Default: try to understand and execute
        return { 
            success: true, 
            result: `正在尝试执行: "${message}"\n如果需要更精确的操作，请使用具体的命令。` 
        };
    }
}

// Initialize content script
const rainbowContent = new RainbowBrowserContent();
console.log('🌈 RainbowBrowserAI content script loaded');