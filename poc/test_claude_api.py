#!/usr/bin/env python3
"""
Test Claude API key to diagnose specific issues
"""

import requests
import json
import sys

API_KEY = "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"

def test_anthropic_api(model_name, api_version="2023-06-01"):
    """Test a specific model with the API key"""
    
    url = "https://api.anthropic.com/v1/messages"
    
    headers = {
        "Content-Type": "application/json",
        "x-api-key": API_KEY,
        "anthropic-version": api_version
    }
    
    data = {
        "model": model_name,
        "max_tokens": 10,  # Use minimal tokens
        "messages": [
            {
                "role": "user",
                "content": "Hi"
            }
        ]
    }
    
    print(f"\n🔍 Testing model: {model_name}")
    print(f"   API Version: {api_version}")
    print(f"   Request URL: {url}")
    
    try:
        response = requests.post(url, headers=headers, json=data, timeout=10)
        
        print(f"   Status Code: {response.status_code}")
        
        if response.status_code == 200:
            result = response.json()
            print(f"   ✅ SUCCESS! Response: {json.dumps(result, indent=2)[:200]}...")
            return True
        else:
            error_data = response.json()
            print(f"   ❌ ERROR: {json.dumps(error_data, indent=2)}")
            
            # Parse specific error details
            if "error" in error_data:
                error = error_data["error"]
                if "type" in error:
                    print(f"   Error Type: {error['type']}")
                if "message" in error:
                    print(f"   Error Message: {error['message']}")
            
            return False
            
    except requests.exceptions.Timeout:
        print(f"   ⏱️ Request timed out")
        return False
    except requests.exceptions.RequestException as e:
        print(f"   🔥 Request failed: {str(e)}")
        return False
    except json.JSONDecodeError:
        print(f"   🔥 Invalid JSON response: {response.text[:200]}")
        return False

def main():
    print("=" * 70)
    print("🧪 Claude API Key Diagnostic Test")
    print("=" * 70)
    print(f"Testing API Key: {API_KEY[:20]}...{API_KEY[-10:]}")
    
    # Test different model variations
    models_to_test = [
        # Claude 3 Haiku variations
        "claude-3-haiku-20240307",
        "claude-3-haiku",
        "claude-3-haiku-20241022",
        
        # Claude 3.5 Haiku
        "claude-3-5-haiku-20241022",
        "claude-3-5-haiku-latest",
        
        # Claude 3 Sonnet
        "claude-3-sonnet-20240229",
        "claude-3-sonnet",
        
        # Claude 3.5 Sonnet
        "claude-3-5-sonnet-20241022",
        "claude-3-5-sonnet-20240620",
        
        # Claude Instant
        "claude-instant-1.2",
        "claude-instant-1",
        
        # Try simplified names
        "claude-3",
        "claude-2",
        "claude-instant"
    ]
    
    success_count = 0
    
    # Test with different API versions
    api_versions = ["2023-06-01", "2024-01-01", "2024-10-01"]
    
    for model in models_to_test[:5]:  # Test first 5 models to save time
        if test_anthropic_api(model, api_versions[0]):
            success_count += 1
            break  # If one works, we found a valid model
    
    # If no model works with default version, try other versions
    if success_count == 0:
        print("\n📝 No models worked with API version 2023-06-01")
        print("🔄 Trying alternative API versions...")
        
        for version in api_versions[1:]:
            print(f"\n🔍 Testing with API version: {version}")
            if test_anthropic_api("claude-3-haiku-20240307", version):
                success_count += 1
                break
    
    # Summary
    print("\n" + "=" * 70)
    print("📊 Test Summary")
    print("=" * 70)
    
    if success_count > 0:
        print("✅ API key is VALID! At least one model worked.")
    else:
        print("❌ API key appears to have issues. Possible causes:")
        print("   1. Insufficient credit balance")
        print("   2. API key is invalid or expired")
        print("   3. Account restrictions or permissions")
        print("   4. Region restrictions")
        print("   5. Rate limiting")
    
    # Test with minimal request to get more details
    print("\n" + "=" * 70)
    print("🔬 Detailed Error Analysis")
    print("=" * 70)
    
    # Try with the most basic request
    url = "https://api.anthropic.com/v1/messages"
    headers = {
        "x-api-key": API_KEY,
        "anthropic-version": "2023-06-01",
        "Content-Type": "application/json"
    }
    
    # Minimal valid request
    data = {
        "model": "claude-3-haiku-20240307",
        "messages": [{"role": "user", "content": "1"}],
        "max_tokens": 1
    }
    
    print("Sending minimal request to get detailed error...")
    response = requests.post(url, headers=headers, json=data, timeout=10)
    
    print(f"Response Status: {response.status_code}")
    print(f"Response Headers: {dict(response.headers)}")
    print(f"Response Body: {response.text}")
    
    return success_count > 0

if __name__ == "__main__":
    success = main()
    sys.exit(0 if success else 1)