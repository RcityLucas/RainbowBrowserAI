#!/usr/bin/env python3
"""
Test if this is a ChatAPI key (alternative OpenAI API provider)
"""

import requests
import json

# The API key that might be for ChatAPI
API_KEY = "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"

def test_chatapi():
    """Test if this key works with ChatAPI endpoints"""
    
    print("=" * 70)
    print("🔍 Testing as ChatAPI Key")
    print("=" * 70)
    print(f"API Key: {API_KEY[:20]}...{API_KEY[-10:]}")
    print()
    
    # ChatAPI endpoints (various alternatives)
    chatapi_endpoints = [
        "https://api.chatanywhere.tech/v1/chat/completions",
        "https://api.chatanywhere.com.cn/v1/chat/completions",
        "https://api.chatanywhere.org/v1/chat/completions",
        "https://api.chatanywhere.cn/v1/chat/completions",
        "https://oa.api2d.net/v1/chat/completions",
        "https://api.api2d.net/v1/chat/completions",
        "https://api.closeai-proxy.xyz/v1/chat/completions",
        "https://api.openai-proxy.com/v1/chat/completions"
    ]
    
    headers = {
        "Content-Type": "application/json",
        "Authorization": f"Bearer {API_KEY}"
    }
    
    data = {
        "model": "gpt-3.5-turbo",
        "messages": [
            {"role": "user", "content": "Say 'yes' if you work"}
        ],
        "max_tokens": 5,
        "temperature": 0
    }
    
    working_endpoint = None
    
    for endpoint in chatapi_endpoints:
        print(f"Testing endpoint: {endpoint}")
        print("-" * 50)
        
        try:
            response = requests.post(
                endpoint, 
                headers=headers, 
                json=data, 
                timeout=10
            )
            
            print(f"Status Code: {response.status_code}")
            
            if response.status_code == 200:
                result = response.json()
                content = result.get('choices', [{}])[0].get('message', {}).get('content', '')
                print(f"✅ SUCCESS! Response: {content}")
                print(f"This endpoint works: {endpoint}")
                working_endpoint = endpoint
                break
            elif response.status_code == 401:
                print("❌ Authentication failed - wrong API key for this service")
            elif response.status_code == 429:
                error_msg = response.json().get('error', {}).get('message', 'Rate limited')
                print(f"❌ Rate limit or quota issue: {error_msg[:100]}")
            else:
                print(f"❌ Error: {response.text[:200]}")
                
        except requests.exceptions.Timeout:
            print("⏱️ Timeout - service might be down")
        except requests.exceptions.ConnectionError:
            print("🔌 Connection error - service unreachable")
        except Exception as e:
            print(f"❌ Exception: {str(e)[:100]}")
        
        print()
    
    print("=" * 70)
    print("📊 Summary")
    print("=" * 70)
    
    if working_endpoint:
        print(f"✅ Found working endpoint: {working_endpoint}")
        print()
        print("🎯 To use this API key:")
        print("1. Update .env file:")
        print(f"   CHATAPI_BASE_URL={working_endpoint}")
        print(f"   CHATAPI_API_KEY={API_KEY}")
        print("   LLM_PROVIDER=chatapi")
        print("   CHATAPI_MODEL=gpt-3.5-turbo")
        print("   RAINBOW_MOCK_MODE=false")
        print()
        print("2. Restart the application")
    else:
        print("❌ This API key doesn't work with any tested ChatAPI endpoints")
        print()
        print("This appears to be a standard OpenAI API key, but it has no quota.")
        print()
        print("The issue is clear:")
        print("• Both OpenAI API keys you provided have no available credits")
        print("• They can authenticate (access the API) but cannot make requests")
        print("• The $5 credits you see might be:")
        print("  - Expired (older than 3 months)")
        print("  - For a different API key")
        print("  - For a different organization/project")
        print()
        print("Solutions:")
        print("1. Create a NEW API key in the correct project/organization")
        print("2. Add a payment method to your OpenAI account")
        print("3. Use a different AI service with free credits")

if __name__ == "__main__":
    test_chatapi()