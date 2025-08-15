#!/bin/bash

# RainbowBrowserAI - 构建发布版本脚本
# 为不同平台构建用户友好的可执行文件

echo "🌈 RainbowBrowserAI - 构建发布版本"
echo "=================================="

# 清理之前的构建
echo "🧹 清理旧版本..."
rm -rf dist
mkdir -p dist

# 构建优化版本
echo "🔨 构建优化版本..."
cargo build --release

# 复制主程序
echo "📦 打包主程序..."
cp target/release/rainbow-browser-ai dist/

# 创建Windows版本
if [[ "$OSTYPE" == "msys" || "$OSTYPE" == "cygwin" ]]; then
    echo "🪟 创建Windows版本..."
    cp target/release/rainbow-browser-ai.exe dist/RainbowBrowserAI.exe
    
    # 创建Windows启动脚本
    cat > dist/启动RainbowBrowserAI.bat << 'EOF'
@echo off
echo 🌈 RainbowBrowserAI 启动中...
echo ==============================
echo.
echo 正在启动智能浏览器助手...
echo 请不要关闭此窗口
echo.
RainbowBrowserAI.exe
pause
EOF
fi

# 创建macOS应用包
if [[ "$OSTYPE" == "darwin"* ]]; then
    echo "🍎 创建macOS应用..."
    mkdir -p "dist/RainbowBrowserAI.app/Contents/MacOS"
    mkdir -p "dist/RainbowBrowserAI.app/Contents/Resources"
    
    cp target/release/rainbow-browser-ai "dist/RainbowBrowserAI.app/Contents/MacOS/RainbowBrowserAI"
    
    # 创建Info.plist
    cat > "dist/RainbowBrowserAI.app/Contents/Info.plist" << 'EOF'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleName</key>
    <string>RainbowBrowserAI</string>
    <key>CFBundleDisplayName</key>
    <string>Rainbow Browser AI</string>
    <key>CFBundleIdentifier</key>
    <string>ai.rainbow-browser</string>
    <key>CFBundleVersion</key>
    <string>8.0.0</string>
    <key>CFBundleExecutable</key>
    <string>RainbowBrowserAI</string>
    <key>LSMinimumSystemVersion</key>
    <string>10.14</string>
</dict>
</plist>
EOF
fi

# 创建Linux AppImage
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    echo "🐧 创建Linux版本..."
    cp target/release/rainbow-browser-ai dist/RainbowBrowserAI
    chmod +x dist/RainbowBrowserAI
    
    # 创建启动脚本
    cat > dist/rainbow-browser.sh << 'EOF'
#!/bin/bash
echo "🌈 RainbowBrowserAI 启动中..."
echo "=============================="
echo ""
echo "正在启动智能浏览器助手..."
echo "请不要关闭此窗口"
echo ""
./RainbowBrowserAI
EOF
    chmod +x dist/rainbow-browser.sh
fi

# 打包浏览器扩展
echo "📦 打包浏览器扩展..."
cd src/browser_extension
zip -r ../../dist/rainbow-browser-extension.zip . -x "*.DS_Store"
cd ../..

# 创建安装说明
cat > dist/README.txt << 'EOF'
🌈 RainbowBrowserAI v8.0 - 安装说明
=====================================

Windows用户:
1. 双击运行 "启动RainbowBrowserAI.bat"
2. 保持窗口开启
3. 安装浏览器扩展 (rainbow-browser-extension.zip)

macOS用户:
1. 双击 RainbowBrowserAI.app
2. 如果提示"无法验证开发者"，右键点击选择"打开"
3. 安装浏览器扩展

Linux用户:
1. 在终端运行: ./rainbow-browser.sh
2. 或直接运行: ./RainbowBrowserAI
3. 安装浏览器扩展

浏览器扩展安装:
1. 打开浏览器扩展管理页面
   Chrome: chrome://extensions
   Edge: edge://extensions
   Firefox: about:addons
2. 开启"开发者模式"
3. 解压 rainbow-browser-extension.zip
4. 点击"加载已解压的扩展程序"
5. 选择解压后的文件夹

使用方法:
1. 点击浏览器工具栏的 🌈 图标
2. 输入自然语言命令
3. AI自动执行！

需要帮助? 访问: https://rainbow-browser.ai/help
EOF

echo ""
echo "✅ 构建完成!"
echo "📁 输出目录: ./dist/"
echo ""
echo "包含文件:"
ls -la dist/
echo ""
echo "🎉 发布版本已准备就绪!"