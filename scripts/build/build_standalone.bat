@echo off
REM RainbowBrowserAI Standalone Build Script for Windows
REM 构建独立可执行文件

echo 🌈 构建RainbowBrowserAI独立可执行文件...

REM 检查Rust环境
where cargo >nul 2>nul
if %ERRORLEVEL% neq 0 (
    echo ❌ 错误: 未找到Cargo。请安装Rust: https://rustup.rs/
    pause
    exit /b 1
)

REM 设置构建选项
set BUILD_FEATURES=standalone,web-server,webdriver
set OUTPUT_DIR=target\standalone

REM 解析命令行参数
:parse_args
if "%1"=="--features" (
    set BUILD_FEATURES=%2
    shift
    shift
    goto parse_args
)
if "%1"=="--output" (
    set OUTPUT_DIR=%2
    shift
    shift
    goto parse_args
)
if "%1"=="--help" goto show_help
if "%1"=="-h" goto show_help
if "%1"=="" goto start_build

echo ❌ 未知选项: %1
echo 使用 --help 查看帮助信息
pause
exit /b 1

:show_help
echo 用法: %0 [选项]
echo.
echo 选项:
echo   --features LIST    启用的功能特性 (默认: standalone,web-server,webdriver)
echo   --output DIR       输出目录 (默认: target\standalone)
echo   --help            显示此帮助信息
echo.
echo 示例:
echo   %0                                    # 构建当前平台
echo   %0 --features standalone              # 只启用独立浏览器功能
pause
exit /b 0

:start_build
REM 创建输出目录
if not exist "%OUTPUT_DIR%" mkdir "%OUTPUT_DIR%"

echo 📦 构建配置:
echo   • 功能特性: %BUILD_FEATURES%
echo   • 输出目录: %OUTPUT_DIR%
echo.

echo 🔨 执行构建...
echo 命令: cargo build --release --bin rainbow-browser-standalone --features %BUILD_FEATURES%
echo.

REM 执行构建
cargo build --release --bin rainbow-browser-standalone --features %BUILD_FEATURES%

if %ERRORLEVEL% equ 0 (
    echo.
    echo ✅ 构建成功!
    
    REM 复制到输出目录
    set EXECUTABLE_PATH=target\release\rainbow-browser-standalone.exe
    if exist "%EXECUTABLE_PATH%" (
        copy "%EXECUTABLE_PATH%" "%OUTPUT_DIR%\rainbow-browser-standalone.exe"
        
        echo.
        echo 📁 输出文件:
        echo   • 路径: %OUTPUT_DIR%\rainbow-browser-standalone.exe
        
        REM 获取文件大小
        for %%I in ("%OUTPUT_DIR%\rainbow-browser-standalone.exe") do echo   • 大小: %%~zI bytes
        
        echo.
        echo 🚀 运行方式:
        echo   %OUTPUT_DIR%\rainbow-browser-standalone.exe
        echo.
        echo 💡 Windows用户注意:
        echo   • 首次运行可能被杀毒软件拦截，请添加信任
        echo   • 需要安装Visual C++ Redistributable
        echo   • 确保防火墙允许网络访问
        echo.
        echo ✨ 独立可执行文件构建完成!
        echo 🎉 现在你可以将文件分发给其他用户，无需安装Rust环境即可运行。
    ) else (
        echo ❌ 错误: 未找到构建的可执行文件: %EXECUTABLE_PATH%
        goto build_failed
    )
) else (
    :build_failed
    echo.
    echo ❌ 构建失败!
    echo.
    echo 常见问题解决方案:
    echo 1. 确保Visual Studio Build Tools已安装
    echo 2. 确保Windows SDK已安装
    echo 3. 尝试清理并重试:
    echo    cargo clean
    echo    cargo build --release --bin rainbow-browser-standalone --features %BUILD_FEATURES%
    echo.
    echo 如果问题持续存在，请检查依赖项或联系支持。
)

pause