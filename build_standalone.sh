#!/bin/bash

# RainbowBrowserAI Standalone Build Script
# 构建独立可执行文件

echo "🌈 构建RainbowBrowserAI独立可执行文件..."

# 检查Rust环境
if ! command -v cargo &> /dev/null; then
    echo "❌ 错误: 未找到Cargo。请安装Rust: https://rustup.rs/"
    exit 1
fi

# 设置构建选项
BUILD_TARGET=""
BUILD_FEATURES="standalone,web-server,webdriver"
OUTPUT_DIR="target/standalone"

# 解析命令行参数
while [[ $# -gt 0 ]]; do
    case $1 in
        --target)
            BUILD_TARGET="$2"
            shift 2
            ;;
        --features)
            BUILD_FEATURES="$2"
            shift 2
            ;;
        --output)
            OUTPUT_DIR="$2"
            shift 2
            ;;
        --help|-h)
            echo "用法: $0 [选项]"
            echo ""
            echo "选项:"
            echo "  --target TRIPLE    目标平台 (例如: x86_64-pc-windows-gnu)"
            echo "  --features LIST    启用的功能特性 (默认: standalone,web-server,webdriver)"
            echo "  --output DIR       输出目录 (默认: target/standalone)"
            echo "  --help            显示此帮助信息"
            echo ""
            echo "示例:"
            echo "  $0                                    # 构建当前平台"
            echo "  $0 --target x86_64-pc-windows-gnu    # 构建Windows版本"
            echo "  $0 --target x86_64-apple-darwin      # 构建macOS版本"
            exit 0
            ;;
        *)
            echo "❌ 未知选项: $1"
            echo "使用 --help 查看帮助信息"
            exit 1
            ;;
    esac
done

# 创建输出目录
mkdir -p "$OUTPUT_DIR"

echo "📦 构建配置:"
echo "  • 功能特性: $BUILD_FEATURES"
echo "  • 目标平台: ${BUILD_TARGET:-当前平台}"
echo "  • 输出目录: $OUTPUT_DIR"
echo ""

# 构建命令
BUILD_CMD="cargo build --release --bin rainbow-browser-standalone --features $BUILD_FEATURES"

if [ -n "$BUILD_TARGET" ]; then
    BUILD_CMD="$BUILD_CMD --target $BUILD_TARGET"
fi

echo "🔨 执行构建..."
echo "命令: $BUILD_CMD"
echo ""

# 执行构建
if eval $BUILD_CMD; then
    echo ""
    echo "✅ 构建成功!"
    
    # 确定可执行文件路径
    if [ -n "$BUILD_TARGET" ]; then
        EXECUTABLE_PATH="target/$BUILD_TARGET/release/rainbow-browser-standalone"
        if [[ "$BUILD_TARGET" == *"windows"* ]]; then
            EXECUTABLE_PATH="${EXECUTABLE_PATH}.exe"
        fi
    else
        EXECUTABLE_PATH="target/release/rainbow-browser-standalone"
        if [[ "$OSTYPE" == "msys" || "$OSTYPE" == "cygwin" ]]; then
            EXECUTABLE_PATH="${EXECUTABLE_PATH}.exe"
        fi
    fi
    
    # 复制到输出目录
    if [ -f "$EXECUTABLE_PATH" ]; then
        FILENAME=$(basename "$EXECUTABLE_PATH")
        cp "$EXECUTABLE_PATH" "$OUTPUT_DIR/$FILENAME"
        
        # 获取文件大小
        FILE_SIZE=$(ls -lh "$OUTPUT_DIR/$FILENAME" | awk '{print $5}')
        
        echo ""
        echo "📁 输出文件:"
        echo "  • 路径: $OUTPUT_DIR/$FILENAME"
        echo "  • 大小: $FILE_SIZE"
        echo ""
        echo "🚀 运行方式:"
        echo "  ./$OUTPUT_DIR/$FILENAME"
        
        # 如果是Windows平台，提供额外说明
        if [[ "$FILENAME" == *.exe ]]; then
            echo ""
            echo "💡 Windows用户注意:"
            echo "  • 首次运行可能被杀毒软件拦截，请添加信任"
            echo "  • 需要安装Visual C++ Redistributable"
            echo "  • 确保防火墙允许网络访问"
        fi
    else
        echo "❌ 错误: 未找到构建的可执行文件: $EXECUTABLE_PATH"
        exit 1
    fi
else
    echo ""
    echo "❌ 构建失败!"
    echo ""
    echo "常见问题解决方案:"
    echo "1. 确保所有依赖已安装:"
    echo "   sudo apt-get install build-essential pkg-config libssl-dev libgtk-3-dev libwebkit2gtk-4.0-dev"
    echo ""
    echo "2. 如果使用Windows，请安装:"
    echo "   • Visual Studio Build Tools"
    echo "   • Windows SDK"
    echo ""
    echo "3. 清理并重试:"
    echo "   cargo clean"
    echo "   $BUILD_CMD"
    exit 1
fi

echo ""
echo "✨ 独立可执行文件构建完成!"
echo "🎉 现在你可以将文件分发给其他用户，无需安装Rust环境即可运行。"