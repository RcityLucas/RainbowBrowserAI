#!/usr/bin/env python3
"""
RainbowBrowserAI V2 API Client Examples (Python)
Demonstrates how to use the new coordinated endpoints
"""

import json
import time
import asyncio
import aiohttp
from typing import Optional, Dict, Any, List
from dataclasses import dataclass
from datetime import datetime


@dataclass
class NavigationResult:
    """Result from navigation operation"""
    url: str
    load_time_ms: int
    success: bool
    analysis: Optional[Dict[str, Any]] = None


@dataclass
class IntelligentActionResult:
    """Result from intelligent action"""
    success: bool
    duration_ms: int
    confidence: float
    steps: List[str]
    verification_success: bool
    learning_applied: bool


class V2ApiClient:
    """Client for RainbowBrowserAI V2 Coordinated API"""
    
    def __init__(self, base_url: str = "http://localhost:3000"):
        self.base_url = base_url
        self.session_id: Optional[str] = None
        self.session: Optional[aiohttp.ClientSession] = None
    
    async def __aenter__(self):
        """Async context manager entry"""
        self.session = aiohttp.ClientSession()
        return self
    
    async def __aexit__(self, exc_type, exc_val, exc_tb):
        """Async context manager exit"""
        if self.session_id:
            await self.delete_session()
        if self.session:
            await self.session.close()
    
    async def create_session(self) -> str:
        """Create a new coordinated session"""
        url = f"{self.base_url}/api/v2/session/create"
        
        async with self.session.post(url) as response:
            data = await response.json()
            
            if data["success"]:
                self.session_id = data["data"]["session_id"]
                print(f"✅ Created session: {self.session_id}")
                print(f"   Modules: {data['data']['modules']}")
                return self.session_id
            else:
                raise Exception(f"Failed to create session: {data.get('error')}")
    
    async def navigate(self, url: str, analyze: bool = False) -> NavigationResult:
        """Navigate to a URL with optional page analysis"""
        api_url = f"{self.base_url}/api/v2/navigate"
        
        request_data = {
            "session_id": self.session_id,
            "data": {
                "url": url,
                "wait_for_load": True,
                "analyze_page": analyze
            }
        }
        
        async with self.session.post(api_url, json=request_data) as response:
            data = await response.json()
            
            if data["success"]:
                result_data = data["data"]
                print(f"✅ Navigated to: {result_data['url']}")
                print(f"   Load time: {result_data['load_time_ms']}ms")
                
                if "metrics" in data and data["metrics"]:
                    metrics = data["metrics"]
                    print(f"   Operation took: {metrics['duration_ms']}ms")
                    print(f"   Cache hits: {metrics['cache_hits']}, misses: {metrics['cache_misses']}")
                
                if "analysis" in result_data and result_data["analysis"]:
                    analysis = result_data["analysis"]
                    print(f"   Page title: {analysis['title']}")
                    print(f"   Elements: {analysis['element_count']}")
                    print(f"   Interactive: {len(analysis['interactive_elements'])} elements")
                
                return NavigationResult(
                    url=result_data["url"],
                    load_time_ms=result_data["load_time_ms"],
                    success=result_data["success"],
                    analysis=result_data.get("analysis")
                )
            else:
                raise Exception(f"Navigation failed: {data.get('error')}")
    
    async def intelligent_action(self, action_type: str, target: str, 
                                parameters: Optional[Dict] = None) -> IntelligentActionResult:
        """Execute an intelligent action"""
        url = f"{self.base_url}/api/v2/intelligent-action"
        
        request_data = {
            "session_id": self.session_id,
            "data": {
                "action_type": action_type,
                "target": target,
                "parameters": parameters or {}
            }
        }
        
        async with self.session.post(url, json=request_data) as response:
            data = await response.json()
            
            if data["success"]:
                result = data["data"]
                print(f"✅ Intelligent action completed: {action_type}")
                print(f"   Target: {target}")
                print(f"   Success: {result['success']}")
                print(f"   Duration: {result['duration_ms']}ms")
                print(f"   Confidence: {result['analysis']['confidence'] * 100:.1f}%")
                print(f"   Plan steps: {len(result['plan']['steps'])}")
                print(f"   Verification: {result['verification']['success']}")
                print(f"   Learning applied: {result['learning_applied']}")
                
                return IntelligentActionResult(
                    success=result["success"],
                    duration_ms=result["duration_ms"],
                    confidence=result["analysis"]["confidence"],
                    steps=result["plan"]["steps"],
                    verification_success=result["verification"]["success"],
                    learning_applied=result["learning_applied"]
                )
            else:
                raise Exception(f"Intelligent action failed: {data.get('error')}")
    
    async def analyze_page(self, analysis_type: str = "standard") -> Dict[str, Any]:
        """Analyze current page with perception"""
        url = f"{self.base_url}/api/v2/perception/analyze"
        
        request_data = {
            "session_id": self.session_id,
            "data": {
                "analysis_type": analysis_type,
                "target": "current_page"
            }
        }
        
        async with self.session.post(url, json=request_data) as response:
            data = await response.json()
            
            if data["success"]:
                analysis = data["data"]
                print(f"✅ Page analysis complete")
                print(f"   Title: {analysis['title']}")
                print(f"   Total elements: {analysis['element_count']}")
                print(f"   Interactive elements: {len(analysis['interactive_elements'])}")
                return analysis
            else:
                raise Exception(f"Page analysis failed: {data.get('error')}")
    
    async def execute_tool(self, tool_name: str, parameters: Dict[str, Any]) -> Any:
        """Execute a tool"""
        url = f"{self.base_url}/api/v2/tool/execute"
        
        request_data = {
            "session_id": self.session_id,
            "data": {
                "tool_name": tool_name,
                "parameters": parameters
            }
        }
        
        async with self.session.post(url, json=request_data) as response:
            data = await response.json()
            
            if data["success"]:
                print(f"✅ Tool executed: {tool_name}")
                if "metrics" in data and data["metrics"]:
                    print(f"   Duration: {data['metrics']['duration_ms']}ms")
                return data["data"]
            else:
                raise Exception(f"Tool execution failed: {data.get('error')}")
    
    async def get_session_health(self) -> Dict[str, Any]:
        """Get session health information"""
        if not self.session_id:
            raise Exception("No active session")
        
        url = f"{self.base_url}/api/v2/session/{self.session_id}"
        
        async with self.session.get(url) as response:
            data = await response.json()
            
            if data["success"]:
                health = data["data"]
                print(f"✅ Session health retrieved")
                print(f"   {json.dumps(health, indent=2)}")
                return health
            else:
                raise Exception(f"Failed to get session health: {data.get('error')}")
    
    async def get_system_health(self) -> Dict[str, Any]:
        """Get system health information"""
        url = f"{self.base_url}/api/v2/health"
        
        async with self.session.get(url) as response:
            data = await response.json()
            
            if data["success"]:
                health = data["data"]
                print(f"✅ System health:")
                print(f"   Healthy: {health['healthy']}")
                print(f"   Sessions: {health['total_sessions']} total, {health['healthy_sessions']} healthy")
                print(f"   Memory: {health['resource_usage']['memory_mb']}MB")
                print(f"   CPU: {health['resource_usage']['cpu_percent']}%")
                return health
            else:
                raise Exception(f"Failed to get system health: {data.get('error')}")
    
    async def list_sessions(self) -> List[Dict[str, Any]]:
        """List all active sessions"""
        url = f"{self.base_url}/api/v2/sessions"
        
        async with self.session.get(url) as response:
            data = await response.json()
            
            if data["success"]:
                sessions = data["data"]["sessions"]
                print(f"✅ Active sessions: {len(sessions)}")
                for session in sessions:
                    print(f"   - {session['session_id']}: active={session['is_active']}")
                return sessions
            else:
                raise Exception(f"Failed to list sessions: {data.get('error')}")
    
    async def delete_session(self) -> None:
        """Delete current session"""
        if not self.session_id:
            return
        
        url = f"{self.base_url}/api/v2/session/{self.session_id}"
        
        async with self.session.delete(url) as response:
            data = await response.json()
            
            if data["success"]:
                print(f"✅ Session deleted: {self.session_id}")
                self.session_id = None
            else:
                raise Exception(f"Failed to delete session: {data.get('error')}")


# Example 1: Basic navigation and analysis
async def example_basic_navigation():
    """Demonstrate basic navigation and page analysis"""
    print("\n🚀 Example 1: Basic Navigation and Analysis")
    print("=" * 50)
    
    async with V2ApiClient() as client:
        # Create a session
        await client.create_session()
        
        # Navigate to a website with analysis
        await client.navigate("https://example.com", analyze=True)
        
        # Wait a bit
        await asyncio.sleep(1)
        
        # Analyze the page
        await client.analyze_page("standard")


# Example 2: Intelligent actions workflow
async def example_intelligent_actions():
    """Demonstrate intelligent action capabilities"""
    print("\n🤖 Example 2: Intelligent Actions Workflow")
    print("=" * 50)
    
    async with V2ApiClient() as client:
        # Create a session
        await client.create_session()
        
        # Navigate to a form page
        await client.navigate("https://example.com/form", analyze=True)
        
        # Fill in a form field intelligently
        await client.intelligent_action("type", "email input field")
        
        # Click submit button intelligently
        await client.intelligent_action("click", "submit button")
        
        # Wait for navigation
        await asyncio.sleep(2)
        
        # Analyze result page
        await client.analyze_page("deep")


# Example 3: Tool execution with coordination
async def example_tool_execution():
    """Demonstrate coordinated tool execution"""
    print("\n🔧 Example 3: Coordinated Tool Execution")
    print("=" * 50)
    
    async with V2ApiClient() as client:
        # Create a session
        await client.create_session()
        
        # Navigate
        await client.navigate("https://example.com")
        
        # Execute click tool
        await client.execute_tool("click", {"selector": "#main-button"})
        
        # Execute extract_text tool
        text = await client.execute_tool("extract_text", {"selector": "h1"})
        print(f"   Extracted text: {text}")
        
        # Execute screenshot tool
        await client.execute_tool("take_screenshot", {"full_page": False})


# Example 4: Health monitoring
async def example_health_monitoring():
    """Demonstrate health monitoring capabilities"""
    print("\n💚 Example 4: Health Monitoring")
    print("=" * 50)
    
    async with V2ApiClient() as client:
        # Check system health before creating session
        await client.get_system_health()
        
        # Create a session
        await client.create_session()
        
        # Perform some operations
        await client.navigate("https://example.com", analyze=True)
        
        # Check session health
        await client.get_session_health()
        
        # More operations
        await client.intelligent_action("click", "navigation link")
        
        # Check health again
        await client.get_session_health()
        
        # Check system health with active session
        await client.get_system_health()


# Example 5: Multi-session coordination
async def example_multi_session():
    """Demonstrate multi-session capabilities"""
    print("\n👥 Example 5: Multi-Session Coordination")
    print("=" * 50)
    
    async with V2ApiClient() as client1, V2ApiClient() as client2:
        # Create sessions
        session1 = await client1.create_session()
        print(f"Session 1: {session1}")
        
        session2 = await client2.create_session()
        print(f"Session 2: {session2}")
        
        # Parallel operations
        results = await asyncio.gather(
            client1.navigate("https://example.com", analyze=True),
            client2.navigate("https://google.com", analyze=True)
        )
        
        print(f"Both navigations completed")
        
        # List all sessions
        await client1.list_sessions()
        
        # Check system health with multiple sessions
        await client1.get_system_health()


# Example 6: Error handling and recovery
async def example_error_handling():
    """Demonstrate error handling and recovery"""
    print("\n⚠️ Example 6: Error Handling and Recovery")
    print("=" * 50)
    
    async with V2ApiClient() as client:
        # Create a session
        await client.create_session()
        
        # Try to navigate to an invalid URL
        try:
            await client.navigate("not-a-valid-url")
        except Exception as e:
            print(f"Expected error: {e}")
        
        # Session should still be valid, try a valid operation
        await client.navigate("https://example.com")
        
        # Try an intelligent action that might fail
        try:
            result = await client.intelligent_action("click", "non-existent-element")
            if not result.success:
                print("Action failed as expected")
        except Exception as e:
            print(f"Request error: {e}")
        
        # Session should still be healthy
        await client.get_session_health()


# Example 7: Advanced workflow with retries
async def example_advanced_workflow():
    """Demonstrate an advanced workflow with retries and error handling"""
    print("\n🎯 Example 7: Advanced Workflow")
    print("=" * 50)
    
    async with V2ApiClient() as client:
        await client.create_session()
        
        # Navigate with retry logic
        max_retries = 3
        for attempt in range(max_retries):
            try:
                await client.navigate("https://example.com", analyze=True)
                break
            except Exception as e:
                if attempt < max_retries - 1:
                    print(f"   Retry {attempt + 1}/{max_retries} after error: {e}")
                    await asyncio.sleep(1)
                else:
                    raise
        
        # Complex action sequence
        actions = [
            ("click", "cookie accept button"),
            ("type", "search input"),
            ("click", "search button")
        ]
        
        for action_type, target in actions:
            try:
                result = await client.intelligent_action(action_type, target)
                if result.success:
                    print(f"   ✓ {action_type} on '{target}' succeeded")
                else:
                    print(f"   ✗ {action_type} on '{target}' failed")
            except Exception as e:
                print(f"   ✗ {action_type} on '{target}' error: {e}")
        
        # Final health check
        health = await client.get_session_health()
        print(f"\nFinal session state: {health.get('health', {}).get('overall', 'unknown')}")


# Main function to run all examples
async def main():
    """Run all examples"""
    print("🌈 RainbowBrowserAI V2 API Client Examples (Python)")
    print("=" * 60)
    
    examples = [
        example_basic_navigation,
        example_intelligent_actions,
        example_tool_execution,
        example_health_monitoring,
        example_multi_session,
        example_error_handling,
        example_advanced_workflow
    ]
    
    for example in examples:
        try:
            await example()
        except Exception as e:
            print(f"❌ Example failed: {e}")
        
        # Small delay between examples
        await asyncio.sleep(1)
    
    print("\n✅ All examples completed!")


if __name__ == "__main__":
    # Run the examples
    asyncio.run(main())