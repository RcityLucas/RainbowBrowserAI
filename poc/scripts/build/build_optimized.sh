#!/bin/bash

# Optimized Build Script for RainbowBrowserAI
# Produces the smallest, fastest binary possible

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
echo -e "${CYAN}       🚀 RainbowBrowserAI Optimized Build Script${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
echo ""

# Function to print section headers
print_section() {
    echo -e "\n${YELLOW}▶ $1${NC}"
    echo -e "${BLUE}───────────────────────────────────────────────${NC}"
}

# Function to check command existence
check_command() {
    if ! command -v $1 &> /dev/null; then
        echo -e "${RED}✗ $1 is not installed${NC}"
        return 1
    else
        echo -e "${GREEN}✓ $1 is installed${NC}"
        return 0
    fi
}

# Parse arguments
BUILD_TYPE=${1:-release}
TARGET=${2:-native}
FEATURES=${3:-default}

print_section "Build Configuration"
echo "Build Type: $BUILD_TYPE"
echo "Target: $TARGET"
echo "Features: $FEATURES"

# Check dependencies
print_section "Checking Dependencies"
check_command cargo
check_command rustc
check_command strip || echo -e "${YELLOW}⚠ strip not found - binary size won't be optimized${NC}"

# Show Rust version
echo -e "\n${CYAN}Rust Version:${NC}"
rustc --version
cargo --version

# Clean previous builds
print_section "Cleaning Previous Builds"
if [ "$BUILD_TYPE" = "clean" ]; then
    cargo clean
    echo -e "${GREEN}✓ Removed all build artifacts${NC}"
    BUILD_TYPE="release"
fi

# Set optimization flags
print_section "Setting Optimization Flags"

# CPU-specific optimizations
if [ "$TARGET" = "native" ]; then
    export RUSTFLAGS="-C target-cpu=native -C opt-level=3 -C lto=fat -C embed-bitcode=yes -C codegen-units=1"
    echo "Using native CPU optimizations"
elif [ "$TARGET" = "generic" ]; then
    export RUSTFLAGS="-C opt-level=3 -C lto=thin -C codegen-units=16"
    echo "Using generic optimizations"
else
    export RUSTFLAGS="-C target-feature=+crt-static -C opt-level=3 -C lto=fat"
    echo "Using custom target: $TARGET"
fi

# Additional flags for release builds
if [ "$BUILD_TYPE" = "release" ]; then
    export RUSTFLAGS="$RUSTFLAGS -C strip=symbols -C panic=abort"
    echo -e "${GREEN}✓ Release optimizations enabled${NC}"
fi

# Show the flags being used
echo -e "\n${CYAN}RUSTFLAGS:${NC} $RUSTFLAGS"

# Build the project
print_section "Building Project"

BUILD_CMD="cargo build"

if [ "$BUILD_TYPE" = "release" ]; then
    BUILD_CMD="$BUILD_CMD --release"
fi

if [ "$FEATURES" != "default" ]; then
    if [ "$FEATURES" = "minimal" ]; then
        BUILD_CMD="$BUILD_CMD --no-default-features"
    else
        BUILD_CMD="$BUILD_CMD --features $FEATURES"
    fi
fi

echo "Executing: $BUILD_CMD"
echo ""

# Measure build time
BUILD_START=$(date +%s)

# Run the build with progress
$BUILD_CMD 2>&1 | while IFS= read -r line; do
    if [[ $line == *"Compiling"* ]]; then
        echo -ne "\r${YELLOW}⚙ ${line:0:60}...${NC}"
    elif [[ $line == *"Finished"* ]]; then
        echo -e "\r${GREEN}✓ Build completed successfully${NC}                    "
    elif [[ $line == *"error"* ]]; then
        echo -e "\r${RED}✗ Build error: $line${NC}"
    fi
done

BUILD_END=$(date +%s)
BUILD_TIME=$((BUILD_END - BUILD_START))

# Get binary path
if [ "$BUILD_TYPE" = "release" ]; then
    BINARY_PATH="target/release/rainbow-poc"
else
    BINARY_PATH="target/debug/rainbow-poc"
fi

# Check if build succeeded
if [ ! -f "$BINARY_PATH" ]; then
    echo -e "${RED}✗ Build failed - binary not found${NC}"
    exit 1
fi

# Strip the binary (release only)
if [ "$BUILD_TYPE" = "release" ] && command -v strip &> /dev/null; then
    print_section "Optimizing Binary Size"
    SIZE_BEFORE=$(stat -f%z "$BINARY_PATH" 2>/dev/null || stat -c%s "$BINARY_PATH" 2>/dev/null || echo "0")
    strip "$BINARY_PATH"
    SIZE_AFTER=$(stat -f%z "$BINARY_PATH" 2>/dev/null || stat -c%s "$BINARY_PATH" 2>/dev/null || echo "0")
    
    if [ "$SIZE_BEFORE" -gt 0 ] && [ "$SIZE_AFTER" -gt 0 ]; then
        REDUCTION=$((SIZE_BEFORE - SIZE_AFTER))
        PERCENT=$((REDUCTION * 100 / SIZE_BEFORE))
        echo -e "${GREEN}✓ Reduced binary size by ${PERCENT}% ($(numfmt --to=iec $REDUCTION))${NC}"
    fi
fi

# Analyze the binary
print_section "Binary Analysis"

# Get file size
if command -v numfmt &> /dev/null; then
    SIZE=$(stat -f%z "$BINARY_PATH" 2>/dev/null || stat -c%s "$BINARY_PATH" 2>/dev/null)
    SIZE_HUMAN=$(numfmt --to=iec $SIZE)
else
    SIZE_HUMAN=$(du -h "$BINARY_PATH" | cut -f1)
fi

echo "Binary: $BINARY_PATH"
echo "Size: $SIZE_HUMAN"
echo "Build Time: ${BUILD_TIME}s"

# Check dependencies
if command -v ldd &> /dev/null; then
    echo -e "\n${CYAN}Dynamic Dependencies:${NC}"
    ldd "$BINARY_PATH" 2>/dev/null | head -5 || echo "  Binary is statically linked or not a Linux binary"
fi

# Generate build report
print_section "Generating Build Report"

REPORT_FILE="build_report_$(date +%Y%m%d_%H%M%S).json"
cat > "$REPORT_FILE" << EOF
{
  "timestamp": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
  "build_type": "$BUILD_TYPE",
  "target": "$TARGET",
  "features": "$FEATURES",
  "rust_version": "$(rustc --version | cut -d' ' -f2)",
  "binary_size": "$SIZE_HUMAN",
  "build_time_seconds": $BUILD_TIME,
  "optimization_flags": "$RUSTFLAGS"
}
EOF

echo -e "${GREEN}✓ Build report saved to $REPORT_FILE${NC}"

# Optional: Run tests
if [ "$4" = "--test" ]; then
    print_section "Running Tests"
    cargo test --release
fi

# Optional: Run benchmarks
if [ "$4" = "--bench" ]; then
    print_section "Running Benchmarks"
    cargo bench --no-fail-fast
fi

# Summary
print_section "Build Summary"
echo -e "${GREEN}✅ Build completed successfully!${NC}"
echo ""
echo "Binary location: ${CYAN}$BINARY_PATH${NC}"
echo "Size: ${CYAN}$SIZE_HUMAN${NC}"
echo "Build time: ${CYAN}${BUILD_TIME}s${NC}"
echo ""
echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}🎉 Optimized build ready for deployment!${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"

# Provide next steps
echo -e "\n${YELLOW}Next steps:${NC}"
echo "1. Test the binary: ./$BINARY_PATH --help"
echo "2. Run the service: ./$BINARY_PATH serve --port 3001"
echo "3. Deploy to production: See DEPLOYMENT_GUIDE.md"
echo ""

# Exit successfully
exit 0