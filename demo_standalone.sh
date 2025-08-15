#!/bin/bash

# RainbowBrowserAI Standalone Demo Script
# 演示独立可执行文件的功能

echo "🌈 RainbowBrowserAI 独立版本演示"
echo "=================================="
echo ""

# 检查是否存在独立可执行文件
if [ ! -f "target/standalone/rainbow-browser-standalone" ]; then
    echo "❌ 独立可执行文件不存在"
    echo "请先运行: ./build_standalone.sh"
    exit 1
fi

echo "✅ 找到独立可执行文件"
echo "📁 文件路径: target/standalone/rainbow-browser-standalone"
echo "📏 文件大小: $(du -h target/standalone/rainbow-browser-standalone | cut -f1)"
echo ""

echo "🚀 演示功能:"
echo "1. 独立运行 - 无需Rust环境"
echo "2. 内置AI服务器 - http://localhost:8888"
echo "3. 欢迎页面 - http://localhost:8889"
echo "4. 自动打开浏览器"
echo "5. 智能浏览器控制"
echo ""

echo "💡 使用方法:"
echo "  直接运行: ./target/standalone/rainbow-browser-standalone"
echo "  或使用构建脚本: ./build_standalone.sh"
echo ""

echo "🎯 特性展示:"
echo "  • 跨平台支持 (Windows/macOS/Linux)"
echo "  • 单文件分发"
echo "  • 自然语言控制浏览器"
echo "  • 智能任务执行"
echo "  • 本地AI处理"
echo ""

echo "📖 详细文档: STANDALONE.md"
echo ""

# 询问是否启动演示
read -p "是否启动演示应用？(y/n): " -n 1 -r
echo ""

if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo "🚀 启动RainbowBrowserAI独立版本..."
    echo "⚠️  按Ctrl+C停止应用"
    echo ""
    
    # 启动独立应用
    ./target/standalone/rainbow-browser-standalone
else
    echo "👋 演示结束。要运行应用，请执行:"
    echo "  ./target/standalone/rainbow-browser-standalone"
fi