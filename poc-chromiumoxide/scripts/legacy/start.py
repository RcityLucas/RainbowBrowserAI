#!/usr/bin/env python3
"""
RainbowBrowserAI Chromiumoxide - Cross-platform Start Script
Usage: python start.py [--port 3002] [--headless] [--debug] [--open-browser]
"""

import os
import sys
import time
import signal
import subprocess
import platform
import argparse
import webbrowser
from pathlib import Path

# ANSI color codes
class Colors:
    BLUE = '\033[94m'
    GREEN = '\033[92m'
    YELLOW = '\033[93m'
    RED = '\033[91m'
    CYAN = '\033[96m'
    END = '\033[0m'
    BOLD = '\033[1m'

def print_colored(color, message):
    """Print colored message"""
    print(f"{color}{message}{Colors.END}")

def print_header():
    """Print application header"""
    print_colored(Colors.CYAN, "═" * 63)
    print_colored(Colors.GREEN + Colors.BOLD, "     🌈 RainbowBrowserAI - Chromiumoxide Edition 🌈")
    print_colored(Colors.CYAN, "═" * 63)
    print()

def check_rust():
    """Check if Rust is installed"""
    try:
        subprocess.run(["cargo", "--version"], capture_output=True, check=True)
        return True
    except (subprocess.SubprocessError, FileNotFoundError):
        print_colored(Colors.RED, "❌ Cargo not found. Please install Rust from https://rustup.rs/")
        return False

def is_port_open(port):
    """Check if a port is available"""
    import socket
    sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    result = sock.connect_ex(('localhost', port))
    sock.close()
    return result == 0

def find_free_port(base_port, max_tries=10):
    """Find an available port starting from base_port"""
    for i in range(max_tries):
        port = base_port + i
        if not is_port_open(port):
            return port
        print_colored(Colors.YELLOW, f"Port {port} is in use, trying {port + 1}...")
    raise Exception(f"Could not find free port after {max_tries} attempts")

def kill_existing_processes():
    """Kill any existing rainbow-poc-chromiumoxide processes"""
    print_colored(Colors.YELLOW, "🔄 Cleaning up old processes...")
    
    if platform.system() == "Windows":
        subprocess.run(["taskkill", "/F", "/IM", "rainbow-poc-chromiumoxide.exe"], 
                      capture_output=True, stderr=subprocess.DEVNULL)
    else:
        subprocess.run(["pkill", "-f", "rainbow-poc-chromiumoxide"], 
                      capture_output=True, stderr=subprocess.DEVNULL)
    
    print_colored(Colors.GREEN, "  ✓ Cleanup complete")

def build_project(debug=False):
    """Build the Rust project"""
    mode = "debug" if debug else "release"
    print_colored(Colors.BLUE, f"\n🔨 Building project in {mode} mode...")
    
    cmd = ["cargo", "build"]
    if not debug:
        cmd.append("--release")
    
    result = subprocess.run(cmd, capture_output=True, text=True)
    if result.returncode != 0:
        print_colored(Colors.RED, "❌ Build failed")
        print(result.stderr)
        return None
    
    print_colored(Colors.GREEN, "  ✓ Build completed")
    
    # Return path to binary
    if platform.system() == "Windows":
        binary = f"target\\{mode}\\rainbow-poc-chromiumoxide.exe"
    else:
        binary = f"target/{mode}/rainbow-poc-chromiumoxide"
    
    return binary

def test_browser(binary, headless=False):
    """Test browser connection"""
    print_colored(Colors.BLUE, "\n🧪 Testing browser connection...")
    
    cmd = [binary, "test"]
    if headless:
        cmd.append("--headless")
    
    result = subprocess.run(cmd, capture_output=True, text=True)
    if "All tests passed" in result.stdout:
        print_colored(Colors.GREEN, "  ✓ Browser test passed")
        return True
    else:
        print_colored(Colors.YELLOW, "  ⚠ Browser test had issues, but continuing...")
        return False

def wait_for_server(port, timeout=30):
    """Wait for server to start"""
    import urllib.request
    import urllib.error
    
    print_colored(Colors.BLUE, "⏳ Waiting for server to start...")
    
    start_time = time.time()
    while time.time() - start_time < timeout:
        try:
            response = urllib.request.urlopen(f"http://localhost:{port}/api/health", timeout=2)
            if response.getcode() == 200:
                return True
        except (urllib.error.URLError, urllib.error.HTTPError):
            pass
        time.sleep(2)
    
    return False

def start_service(port=3002, headless=False, debug=False, open_browser=False):
    """Main function to start the service"""
    print_header()
    
    # Check prerequisites
    if not check_rust():
        return 1
    
    # Kill existing processes
    kill_existing_processes()
    
    # Find available port
    print_colored(Colors.BLUE, "\n🔍 Finding available port...")
    try:
        actual_port = find_free_port(port)
        print_colored(Colors.GREEN, f"  ✓ Using port {actual_port}")
    except Exception as e:
        print_colored(Colors.RED, f"❌ {e}")
        return 1
    
    # Build project
    binary = build_project(debug)
    if not binary:
        return 1
    
    # Test browser
    test_browser(binary, headless)
    
    # Start server
    print_colored(Colors.CYAN, "\n" + "═" * 63)
    print_colored(Colors.GREEN + Colors.BOLD, "           🚀 Starting RainbowBrowserAI Server 🚀")
    print_colored(Colors.CYAN, "═" * 63)
    
    mode = "HEADLESS" if headless else "HEADED (Browser Visible)"
    print_colored(Colors.GREEN, f"  📍 Dashboard: http://localhost:{actual_port}")
    print_colored(Colors.GREEN, f"  📊 API Health: http://localhost:{actual_port}/api/health")
    print_colored(Colors.GREEN, f"  🔧 Tools API: http://localhost:{actual_port}/api/tools")
    print_colored(Colors.YELLOW, f"  🎯 Mode: {mode}")
    print_colored(Colors.CYAN, "═" * 63)
    print()
    
    # Prepare server command
    cmd = [binary, "serve", "--port", str(actual_port)]
    if headless:
        cmd.append("--headless")
    
    # Start server process
    process = subprocess.Popen(cmd)
    
    # Wait for server to start
    if wait_for_server(actual_port):
        print_colored(Colors.GREEN, "\n✅ Service started successfully!")
        
        if open_browser:
            print_colored(Colors.BLUE, "🌐 Opening dashboard in browser...")
            webbrowser.open(f"http://localhost:{actual_port}")
        
        print_colored(Colors.YELLOW, "\nPress Ctrl+C to stop the server")
        print_colored(Colors.CYAN, "═" * 63 + "\n")
        
        # Wait for process or interrupt
        try:
            process.wait()
        except KeyboardInterrupt:
            print_colored(Colors.YELLOW, "\n\n🛑 Shutting down...")
            process.terminate()
            try:
                process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                process.kill()
            print_colored(Colors.GREEN, "✓ Shutdown complete")
    else:
        print_colored(Colors.RED, "❌ Service failed to start")
        process.terminate()
        return 1
    
    return 0

def main():
    """Main entry point"""
    parser = argparse.ArgumentParser(description='Start RainbowBrowserAI Chromiumoxide Service')
    parser.add_argument('--port', type=int, default=3002, help='Port to run server on (default: 3002)')
    parser.add_argument('--headless', action='store_true', help='Run browser in headless mode')
    parser.add_argument('--debug', action='store_true', help='Build in debug mode')
    parser.add_argument('--open-browser', action='store_true', help='Open dashboard in browser after start')
    
    args = parser.parse_args()
    
    # Change to script directory
    script_dir = Path(__file__).parent
    os.chdir(script_dir)
    
    # Start service
    return start_service(
        port=args.port,
        headless=args.headless,
        debug=args.debug,
        open_browser=args.open_browser
    )

if __name__ == "__main__":
    sys.exit(main())