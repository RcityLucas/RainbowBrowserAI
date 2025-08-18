#!/usr/bin/env python3
"""
Test the new OpenAI API key that should have credits
"""

import requests
import json

# New API Key that should have credits
API_KEY = "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"

def test_api_key():
    """Test if this API key works"""
    
    print("=" * 70)
    print("🔍 Testing New OpenAI API Key")
    print("=" * 70)
    print(f"API Key: {API_KEY[:20]}...{API_KEY[-10:]}")
    print()
    
    # Test 1: Check models endpoint
    print("Test 1: Checking Models Endpoint...")
    print("-" * 50)
    
    url = "https://api.openai.com/v1/models"
    headers = {
        "Authorization": f"Bearer {API_KEY}"
    }
    
    try:
        response = requests.get(url, headers=headers, timeout=10)
        print(f"Status Code: {response.status_code}")
        
        if response.status_code == 200:
            data = response.json()
            print(f"✅ Can access API - Found {len(data.get('data', []))} models")
            # Show some available models
            models = [m['id'] for m in data.get('data', [])[:5]]
            for model in models:
                print(f"  - {model}")
        else:
            print(f"❌ Error: {response.text}")
            return False
    except Exception as e:
        print(f"❌ Exception: {str(e)}")
        return False
    
    print()
    
    # Test 2: Try chat completion with minimal tokens
    print("Test 2: Chat Completion Test...")
    print("-" * 50)
    
    url = "https://api.openai.com/v1/chat/completions"
    headers = {
        "Content-Type": "application/json",
        "Authorization": f"Bearer {API_KEY}"
    }
    
    # Try different models
    models_to_try = [
        "gpt-3.5-turbo",
        "gpt-3.5-turbo-0125", 
        "gpt-4o-mini",
        "gpt-4o-mini-2024-07-18"
    ]
    
    working_model = None
    
    for model in models_to_try:
        print(f"\nTrying model: {model}")
        
        data = {
            "model": model,
            "messages": [
                {"role": "user", "content": "Say 'yes' if you work"}
            ],
            "max_tokens": 5,
            "temperature": 0
        }
        
        try:
            response = requests.post(url, headers=headers, json=data, timeout=10)
            print(f"  Status: {response.status_code}")
            
            if response.status_code == 200:
                result = response.json()
                content = result['choices'][0]['message']['content']
                tokens = result.get('usage', {})
                print(f"  ✅ SUCCESS! Response: {content}")
                print(f"  Tokens used: {tokens}")
                working_model = model
                break
            else:
                error_data = response.json()
                error_msg = error_data.get('error', {}).get('message', 'Unknown error')
                error_type = error_data.get('error', {}).get('type', 'Unknown')
                
                if "quota" in error_msg.lower():
                    print(f"  ❌ Quota issue: {error_msg[:100]}")
                elif "model" in error_msg.lower():
                    print(f"  ❌ Model not available")
                else:
                    print(f"  ❌ Error ({error_type}): {error_msg[:100]}")
                    
        except Exception as e:
            print(f"  ❌ Exception: {str(e)}")
    
    print()
    print("=" * 70)
    print("📊 Results Summary")
    print("=" * 70)
    
    if working_model:
        print(f"✅ API Key is WORKING!")
        print(f"✅ Working model: {working_model}")
        print()
        print("🎯 Next Steps:")
        print("1. Update .env file with this API key")
        print("2. Set OPENAI_API_KEY to this key")
        print(f"3. Set OPENAI_MODEL={working_model}")
        print("4. Set RAINBOW_MOCK_MODE=false")
        print("5. Restart the application")
        return True
    else:
        print("❌ API Key is NOT working with any tested model")
        print()
        print("Possible issues:")
        print("1. API key has no credits/quota")
        print("2. API key is invalid or revoked")
        print("3. API key belongs to an organization without billing")
        print("4. Free trial credits have expired (3 month limit)")
        print()
        print("Please check:")
        print("- https://platform.openai.com/account/billing")
        print("- https://platform.openai.com/api-keys")
        return False

if __name__ == "__main__":
    test_api_key()