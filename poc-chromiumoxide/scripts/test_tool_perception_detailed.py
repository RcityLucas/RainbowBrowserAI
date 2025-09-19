#!/usr/bin/env python3
"""
Detailed test script for perception of tool operation interfaces
Tests whether the perception system can recognize and interact with various UI elements
"""

import json
import time
import requests
import subprocess
import sys
from typing import Dict, Any, List

# ANSI color codes
RED = '\033[0;31m'
GREEN = '\033[0;32m'
YELLOW = '\033[1;33m'
BLUE = '\033[0;34m'
MAGENTA = '\033[0;35m'
CYAN = '\033[0;36m'
NC = '\033[0m'  # No Color

BASE_URL = "http://localhost:3000"

def print_colored(message: str, color: str = NC):
    """Print colored message"""
    print(f"{color}{message}{NC}")

def print_test_header(test_name: str):
    """Print a formatted test header"""
    print("\n" + "="*60)
    print_colored(f"TEST: {test_name}", CYAN)
    print("="*60)

def print_result(success: bool, message: str):
    """Print test result with appropriate color"""
    if success:
        print_colored(f"✓ {message}", GREEN)
    else:
        print_colored(f"✗ {message}", RED)

def start_server():
    """Start the Rust server"""
    print_colored("Starting server...", YELLOW)
    # Build in release mode
    subprocess.run(["cargo", "build", "--release"], check=True)
    # Start the server
    process = subprocess.Popen(["cargo", "run", "--release"])
    time.sleep(5)  # Wait for server to start
    return process

def test_tool_registry_list():
    """Test if tool registry is accessible"""
    print_test_header("Tool Registry List")
    
    try:
        response = requests.get(f"{BASE_URL}/api/tools/list")
        if response.status_code == 200:
            data = response.json()
            print_result(True, f"Tool registry accessible - {len(data.get('tools', []))} tools available")
            
            # Print tool categories
            if 'categories' in data:
                print_colored("\nTool Categories:", BLUE)
                for category, count in data['categories'].items():
                    print(f"  • {category}: {count} tools")
            
            return True
        else:
            print_result(False, f"Failed to access tool registry: {response.status_code}")
            return False
    except Exception as e:
        print_result(False, f"Error accessing tool registry: {e}")
        return False

def test_perception_modes():
    """Test different perception modes"""
    print_test_header("Perception Modes")
    
    modes = ["lightning", "quick", "standard", "deep", "adaptive"]
    results = []
    
    for mode in modes:
        try:
            response = requests.post(
                f"{BASE_URL}/api/perception/perceive",
                json={
                    "mode": mode,
                    "url": "about:blank"
                }
            )
            
            if response.status_code == 200:
                data = response.json()
                if data.get('success'):
                    print_result(True, f"Mode '{mode}' perception successful")
                    
                    # Check if tool-operable elements are detected
                    perception_data = data.get('data', {})
                    if isinstance(perception_data, dict):
                        elements = perception_data.get('elements', [])
                        interactive = [e for e in elements if e.get('interactive')]
                        print(f"    → Found {len(interactive)} interactive elements")
                    
                    results.append(True)
                else:
                    print_result(False, f"Mode '{mode}' perception failed: {data.get('error')}")
                    results.append(False)
            else:
                print_result(False, f"Mode '{mode}' request failed: {response.status_code}")
                results.append(False)
                
        except Exception as e:
            print_result(False, f"Error testing mode '{mode}': {e}")
            results.append(False)
    
    return all(results)

def test_tool_element_recognition():
    """Test recognition of tool-operable interface elements"""
    print_test_header("Tool-Operable Element Recognition")
    
    # Navigate to a test page with various UI elements
    test_urls = [
        "https://www.example.com",
        "https://www.google.com",
        "about:blank"
    ]
    
    for url in test_urls:
        print_colored(f"\nTesting URL: {url}", BLUE)
        
        try:
            # First navigate and perceive
            response = requests.post(
                f"{BASE_URL}/api/perception/navigate-and-perceive",
                json={
                    "url": url,
                    "mode": "standard"
                }
            )
            
            if response.status_code == 200:
                data = response.json()
                if data.get('success'):
                    perception = data.get('data', {}).get('perception', {})
                    
                    # Check for different types of tool-operable elements
                    element_types = {
                        'buttons': 0,
                        'links': 0,
                        'inputs': 0,
                        'forms': 0,
                        'selects': 0
                    }
                    
                    # Analyze elements
                    elements = perception.get('elements', [])
                    for elem in elements:
                        tag = elem.get('tag_name', '').lower()
                        if tag == 'button':
                            element_types['buttons'] += 1
                        elif tag == 'a':
                            element_types['links'] += 1
                        elif tag == 'input':
                            element_types['inputs'] += 1
                        elif tag == 'form':
                            element_types['forms'] += 1
                        elif tag == 'select':
                            element_types['selects'] += 1
                    
                    print_colored("    Tool-operable elements found:", MAGENTA)
                    for elem_type, count in element_types.items():
                        if count > 0:
                            print(f"      • {elem_type}: {count}")
                    
                    # Test smart element search
                    search_response = requests.post(
                        f"{BASE_URL}/api/perception/smart-search",
                        json={
                            "query": "clickable",
                            "max_results": 10
                        }
                    )
                    
                    if search_response.status_code == 200:
                        search_data = search_response.json()
                        if search_data.get('success'):
                            matches = search_data.get('data', {}).get('matches', [])
                            print_result(True, f"Smart search found {len(matches)} clickable elements")
                        else:
                            print_result(False, "Smart search failed")
                    
                    print_result(True, f"Successfully analyzed {url}")
                else:
                    print_result(False, f"Perception failed for {url}")
            else:
                print_result(False, f"Navigation failed for {url}: {response.status_code}")
                
        except Exception as e:
            print_result(False, f"Error testing {url}: {e}")

def test_form_analysis():
    """Test form analysis and auto-fill capabilities"""
    print_test_header("Form Analysis & Tool Integration")
    
    try:
        # Test form analysis
        response = requests.post(
            f"{BASE_URL}/api/perception/analyze-form",
            json={}
        )
        
        if response.status_code == 200:
            data = response.json()
            if data.get('success'):
                form_data = data.get('data', {})
                forms_found = form_data.get('forms_found', [])
                
                print_result(True, f"Form analysis successful - {len(forms_found)} forms detected")
                
                for i, form in enumerate(forms_found):
                    print_colored(f"\n  Form {i+1}:", BLUE)
                    print(f"    Selector: {form.get('selector')}")
                    print(f"    Fields: {', '.join(form.get('fields', []))}")
                    print(f"    Action: {form.get('action')}")
                
                return True
            else:
                print_result(False, f"Form analysis failed: {data.get('error')}")
                return False
        else:
            print_result(False, f"Form analysis request failed: {response.status_code}")
            return False
            
    except Exception as e:
        print_result(False, f"Error in form analysis: {e}")
        return False

def test_intelligent_actions():
    """Test intelligent action recognition"""
    print_test_header("Intelligent Action Recognition")
    
    test_commands = [
        {"action": "click", "target": "submit button"},
        {"action": "type", "target": "search box", "text": "test query"},
        {"action": "select", "target": "dropdown", "option": "first option"},
        {"action": "hover", "target": "menu"},
        {"action": "scroll", "direction": "down", "amount": 500}
    ]
    
    results = []
    for command in test_commands:
        try:
            response = requests.post(
                f"{BASE_URL}/api/perception/intelligent-command",
                json={"command": command}
            )
            
            if response.status_code == 200:
                data = response.json()
                if data.get('success'):
                    print_result(True, f"Command recognized: {command['action']} on {command.get('target', 'page')}")
                    results.append(True)
                else:
                    print_result(False, f"Command failed: {command['action']}")
                    results.append(False)
            else:
                print_result(False, f"Request failed for command: {command['action']}")
                results.append(False)
                
        except Exception as e:
            print_result(False, f"Error executing command {command['action']}: {e}")
            results.append(False)
    
    return all(results)

def test_tool_execution_with_perception():
    """Test tool execution with perception context"""
    print_test_header("Tool Execution with Perception Context")
    
    tools_to_test = [
        ("extract_text", {}),
        ("extract_links", {}),
        ("get_element_info", {"selector": "body"}),
        ("screenshot", {"full_page": False})
    ]
    
    results = []
    for tool_name, input_data in tools_to_test:
        try:
            response = requests.post(
                f"{BASE_URL}/api/tools/execute",
                json={
                    "name": tool_name,
                    "input": input_data
                }
            )
            
            if response.status_code == 200:
                data = response.json()
                if data.get('success'):
                    print_result(True, f"Tool '{tool_name}' executed successfully")
                    results.append(True)
                else:
                    print_result(False, f"Tool '{tool_name}' execution failed: {data.get('error')}")
                    results.append(False)
            else:
                print_result(False, f"Request failed for tool '{tool_name}': {response.status_code}")
                results.append(False)
                
        except Exception as e:
            print_result(False, f"Error executing tool '{tool_name}': {e}")
            results.append(False)
    
    return all(results)

def main():
    """Main test execution"""
    print_colored("\n" + "="*60, CYAN)
    print_colored("PERCEPTION TOOL INTERFACE RECOGNITION TEST SUITE", CYAN)
    print_colored("="*60 + "\n", CYAN)
    
    # Start server
    server_process = None
    try:
        server_process = start_server()
        
        # Run tests
        test_results = []
        
        test_results.append(("Tool Registry", test_tool_registry_list()))
        test_results.append(("Perception Modes", test_perception_modes()))
        test_results.append(("Element Recognition", test_tool_element_recognition()))
        test_results.append(("Form Analysis", test_form_analysis()))
        test_results.append(("Intelligent Actions", test_intelligent_actions()))
        test_results.append(("Tool Execution", test_tool_execution_with_perception()))
        
        # Print summary
        print_colored("\n" + "="*60, CYAN)
        print_colored("TEST SUMMARY", CYAN)
        print_colored("="*60, CYAN)
        
        passed = sum(1 for _, result in test_results if result)
        total = len(test_results)
        
        for test_name, result in test_results:
            status = "PASSED" if result else "FAILED"
            color = GREEN if result else RED
            print_colored(f"{test_name:.<40} {status}", color)
        
        print_colored(f"\nTotal: {passed}/{total} tests passed", 
                     GREEN if passed == total else YELLOW)
        
        # Return exit code
        return 0 if passed == total else 1
        
    except KeyboardInterrupt:
        print_colored("\nTest interrupted by user", YELLOW)
        return 1
    except Exception as e:
        print_colored(f"\nTest suite error: {e}", RED)
        return 1
    finally:
        # Clean up
        if server_process:
            print_colored("\nShutting down server...", YELLOW)
            server_process.terminate()
            server_process.wait()

if __name__ == "__main__":
    sys.exit(main())