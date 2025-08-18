#!/usr/bin/env python3
"""
Direct OpenAI API test to diagnose quota issues
"""

import requests
import json
import os

# API Key from .env
API_KEY = "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"

def test_openai_models():
    """Test which models are available"""
    url = "https://api.openai.com/v1/models"
    headers = {
        "Authorization": f"Bearer {API_KEY}"
    }
    
    print("🔍 Testing OpenAI Models Endpoint...")
    print("-" * 50)
    
    try:
        response = requests.get(url, headers=headers, timeout=10)
        print(f"Status Code: {response.status_code}")
        
        if response.status_code == 200:
            data = response.json()
            print(f"✅ Successfully retrieved {len(data.get('data', []))} models")
            
            # Show first few models
            for model in data.get('data', [])[:5]:
                print(f"  - {model.get('id')}")
            return True
        else:
            print(f"❌ Error: {response.text}")
            return False
    except Exception as e:
        print(f"❌ Exception: {str(e)}")
        return False

def test_chat_completion():
    """Test chat completion with minimal tokens"""
    url = "https://api.openai.com/v1/chat/completions"
    headers = {
        "Content-Type": "application/json",
        "Authorization": f"Bearer {API_KEY}"
    }
    
    # Minimal request
    data = {
        "model": "gpt-3.5-turbo",
        "messages": [
            {"role": "user", "content": "Say 'hi'"}
        ],
        "max_tokens": 5,
        "temperature": 0
    }
    
    print("\n🔍 Testing Chat Completion...")
    print("-" * 50)
    print(f"Request: {json.dumps(data, indent=2)}")
    
    try:
        response = requests.post(url, headers=headers, json=data, timeout=10)
        print(f"Status Code: {response.status_code}")
        
        if response.status_code == 200:
            result = response.json()
            print("✅ Success!")
            print(f"Response: {result['choices'][0]['message']['content']}")
            print(f"Tokens used: {result.get('usage', {})}")
            return True
        else:
            error_data = response.json()
            print(f"❌ Error Response: {json.dumps(error_data, indent=2)}")
            
            # Parse specific error
            if "error" in error_data:
                error = error_data["error"]
                print(f"\nError Type: {error.get('type')}")
                print(f"Error Message: {error.get('message')}")
                print(f"Error Code: {error.get('code')}")
                
                # Check if it's a billing issue
                if "billing" in error.get('message', '').lower() or "quota" in error.get('message', '').lower():
                    print("\n⚠️  This appears to be a billing/quota issue")
                    print("Possible causes:")
                    print("1. API key belongs to an organization without payment method")
                    print("2. Free trial credits have expired")
                    print("3. Monthly spending limit has been reached")
                    print("4. API key doesn't have proper permissions")
            return False
    except Exception as e:
        print(f"❌ Exception: {str(e)}")
        return False

def test_account_info():
    """Try to get account/billing info (may not work with all keys)"""
    # Note: OpenAI doesn't provide direct billing API access via regular API keys
    # This is just to test if we can get any account-related info
    
    print("\n🔍 Testing Account Access...")
    print("-" * 50)
    
    # Try usage endpoint (this might work)
    url = "https://api.openai.com/v1/usage"
    headers = {
        "Authorization": f"Bearer {API_KEY}"
    }
    
    try:
        response = requests.get(url, headers=headers, timeout=10)
        print(f"Usage endpoint status: {response.status_code}")
        if response.status_code == 200:
            print(f"Response: {response.text[:500]}")
        else:
            print(f"Response: {response.text[:500]}")
    except Exception as e:
        print(f"Exception: {str(e)}")

def test_different_models():
    """Test different model variations"""
    models_to_test = [
        "gpt-3.5-turbo",
        "gpt-3.5-turbo-0125",
        "gpt-3.5-turbo-1106",
        "gpt-4o-mini",
        "gpt-4o-mini-2024-07-18"
    ]
    
    print("\n🔍 Testing Different Models...")
    print("-" * 50)
    
    url = "https://api.openai.com/v1/chat/completions"
    headers = {
        "Content-Type": "application/json",
        "Authorization": f"Bearer {API_KEY}"
    }
    
    for model in models_to_test:
        data = {
            "model": model,
            "messages": [{"role": "user", "content": "1"}],
            "max_tokens": 1
        }
        
        try:
            response = requests.post(url, headers=headers, json=data, timeout=5)
            if response.status_code == 200:
                print(f"✅ {model}: Works!")
                return model
            else:
                error = response.json().get('error', {})
                if "model" in error.get('message', '').lower():
                    print(f"❌ {model}: Model not available")
                else:
                    print(f"❌ {model}: {error.get('message', 'Unknown error')}")
        except Exception as e:
            print(f"❌ {model}: Exception - {str(e)}")
    
    return None

def main():
    print("=" * 70)
    print("🔬 OpenAI API Direct Testing")
    print("=" * 70)
    print(f"API Key: {API_KEY[:20]}...{API_KEY[-10:]}")
    print()
    
    # Test 1: Check if we can list models
    models_work = test_openai_models()
    
    # Test 2: Try chat completion
    chat_works = test_chat_completion()
    
    # Test 3: Try account info
    test_account_info()
    
    # Test 4: Try different models if main test failed
    if not chat_works:
        working_model = test_different_models()
        if working_model:
            print(f"\n✅ Found working model: {working_model}")
            print("Update your .env file:")
            print(f"OPENAI_MODEL={working_model}")
    
    # Summary
    print("\n" + "=" * 70)
    print("📊 Summary")
    print("=" * 70)
    
    if models_work and chat_works:
        print("✅ API key is working correctly!")
        print("The issue might be in the Rust implementation.")
    elif models_work and not chat_works:
        print("⚠️  API key can access OpenAI but cannot use chat models")
        print("This usually means a billing/quota issue.")
        print("\nTroubleshooting steps:")
        print("1. Check https://platform.openai.com/account/billing")
        print("2. Verify payment method is set up")
        print("3. Check if you're part of an organization")
        print("4. Try generating a new API key")
        print("5. Check rate limits: https://platform.openai.com/account/rate-limits")
    else:
        print("❌ API key is not working at all")
        print("The key might be invalid or revoked.")
    
    print("\n💡 Note about $5 credits:")
    print("If you see $0.00 / $5 in the usage page, this means:")
    print("- You have used $0.00 out of a $5 limit")
    print("- But the credits might be expired (they expire after 3 months)")
    print("- Or they might be for a different API key/organization")

if __name__ == "__main__":
    main()