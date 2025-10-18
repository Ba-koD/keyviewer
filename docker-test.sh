#!/bin/bash
# Docker-based Local Testing for GitHub Actions (Linux/macOS version)
# This script simulates the exact build environment used in GitHub Actions

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Default values
PLATFORM="all"
CLEAN=false
REBUILD=false
SHELL_MODE=false

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --platform)
            PLATFORM="$2"
            shift 2
            ;;
        --clean)
            CLEAN=true
            shift
            ;;
        --rebuild)
            REBUILD=true
            shift
            ;;
        --shell)
            SHELL_MODE=true
            shift
            ;;
        --help)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --platform <linux|macos-check|all>  Select platform (default: all)"
            echo "  --clean                              Clean builds and caches"
            echo "  --rebuild                            Rebuild Docker images"
            echo "  --shell                              Open interactive shell"
            echo "  --help                               Show this help"
            exit 0
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            exit 1
            ;;
    esac
done

echo -e "\n${CYAN}========================================${NC}"
echo -e "${CYAN}  Docker-based GitHub Actions Test     ${NC}"
echo -e "${CYAN}========================================${NC}\n"

# Check Docker
if ! command -v docker &> /dev/null; then
    echo -e "${RED}✗ Docker is not installed!${NC}"
    echo -e "${YELLOW}  Please install Docker first${NC}"
    exit 1
fi

echo -e "${GREEN}✓ Docker: $(docker --version)${NC}"
echo -e "${GREEN}✓ Docker Compose: $(docker compose version)${NC}"
echo ""

# Read version
VERSION=$(cat version.txt | tr -d '\n\r')
echo -e "${YELLOW}Version: $VERSION${NC}\n"

# Function to ensure icon.png exists
ensure_icon_png() {
    echo -e "${YELLOW}Checking for icon.png...${NC}"
    
    if [ ! -f "src-tauri/icons/icon.png" ]; then
        echo -e "${YELLOW}icon.png not found, creating placeholder...${NC}"
        
        # Create placeholder using ImageMagick in Docker
        docker run --rm -v "$(pwd):/work" alpine/imagemagick:latest \
            convert -size 256x256 xc:blue -fill white -gravity center \
            -pointsize 48 -annotate +0+0 "KV" /work/src-tauri/icons/icon.png 2>/dev/null || true
        
        if [ -f "src-tauri/icons/icon.png" ]; then
            echo -e "${GREEN}✓ Placeholder icon.png created${NC}"
        else
            echo -e "${RED}✗ Could not create icon.png${NC}"
            return 1
        fi
    else
        echo -e "${GREEN}✓ icon.png exists${NC}"
    fi
    
    echo ""
    return 0
}

# Clean if requested
if [ "$CLEAN" = true ]; then
    echo -e "${YELLOW}Cleaning previous builds and caches...${NC}"
    
    rm -rf dist src-tauri/target 2>/dev/null || true
    echo -e "${GREEN}✓ Cleaned local directories${NC}"
    
    docker volume rm keyviewer-linux-cargo-cache 2>/dev/null || true
    docker volume rm keyviewer-linux-target-cache 2>/dev/null || true
    docker volume rm keyviewer-macos-cargo-cache 2>/dev/null || true
    echo -e "${GREEN}✓ Cleaned Docker volumes${NC}\n"
fi

# Rebuild images if requested
if [ "$REBUILD" = true ]; then
    echo -e "${YELLOW}Rebuilding Docker images...${NC}"
    docker compose build --no-cache
    echo -e "${GREEN}✓ Docker images rebuilt${NC}\n"
fi

# Ensure icon.png
if ! ensure_icon_png; then
    echo -e "${RED}✗ Cannot proceed without icon.png${NC}"
    exit 1
fi

# Function to test Linux build
test_linux_build() {
    echo -e "${CYAN}========================================${NC}"
    echo -e "${CYAN}  Linux Build (Ubuntu 22.04 + Docker)  ${NC}"
    echo -e "${CYAN}========================================${NC}\n"
    
    if [ "$SHELL_MODE" = true ]; then
        echo -e "${YELLOW}Opening interactive shell...${NC}"
        docker compose run --rm linux-build bash
        return 0
    fi
    
    echo -e "${YELLOW}Building Docker image...${NC}"
    docker compose build linux-build
    
    echo -e "${GREEN}✓ Docker image built${NC}\n"
    echo -e "${YELLOW}Running Tauri build in container...${NC}\n"
    
    START_TIME=$(date +%s)
    
    if docker compose run --rm linux-build bash -c "
set -e
echo '========================================'
echo '  Simulating GitHub Actions Linux Build'
echo '========================================'
echo ''
echo 'Environment Info:'
rustc --version
cargo --version
cargo tauri --version
echo ''
echo 'Building Tauri app...'
cd /app/src-tauri
cargo tauri build --verbose
"; then
        END_TIME=$(date +%s)
        DURATION=$((END_TIME - START_TIME))
        
        echo -e "\n${GREEN}✓ Linux build completed in ${DURATION}s${NC}"
        
        # Check artifacts
        echo -e "\n${YELLOW}Build Artifacts:${NC}"
        
        if ls src-tauri/target/release/bundle/deb/*.deb 1> /dev/null 2>&1; then
            for file in src-tauri/target/release/bundle/deb/*.deb; do
                size=$(du -h "$file" | cut -f1)
                echo -e "  ${GREEN}✓ DEB: $(basename "$file") ($size)${NC}"
            done
        fi
        
        if ls src-tauri/target/release/bundle/appimage/*.AppImage 1> /dev/null 2>&1; then
            for file in src-tauri/target/release/bundle/appimage/*.AppImage; do
                size=$(du -h "$file" | cut -f1)
                echo -e "  ${GREEN}✓ AppImage: $(basename "$file") ($size)${NC}"
            done
        fi
        
        return 0
    else
        END_TIME=$(date +%s)
        DURATION=$((END_TIME - START_TIME))
        
        echo -e "\n${RED}✗ Linux build failed after ${DURATION}s${NC}"
        return 1
    fi
}

# Function to check macOS compilation
test_macos_check() {
    echo -e "${CYAN}========================================${NC}"
    echo -e "${CYAN}  macOS Compilation Check              ${NC}"
    echo -e "${CYAN}========================================${NC}\n"
    echo -e "${YELLOW}Note: This only checks compilation${NC}\n"
    
    if [ "$SHELL_MODE" = true ]; then
        docker compose run --rm macos-check bash
        return 0
    fi
    
    echo -e "${YELLOW}Building Docker image...${NC}"
    docker compose build macos-check
    
    echo -e "${GREEN}✓ Docker image built${NC}\n"
    echo -e "${YELLOW}Checking macOS target compilation...${NC}\n"
    
    START_TIME=$(date +%s)
    
    if docker compose run --rm macos-check bash -c "
set -e
echo '========================================'
echo '  macOS Compilation Check'
echo '========================================'
echo ''
rustc --version
cargo --version
rustup target list --installed
echo ''
echo 'Checking compilation for x86_64-apple-darwin...'
cd /app/src-tauri
cargo check --target x86_64-apple-darwin --verbose
"; then
        END_TIME=$(date +%s)
        DURATION=$((END_TIME - START_TIME))
        
        echo -e "\n${GREEN}✓ macOS compilation check passed in ${DURATION}s${NC}"
        return 0
    else
        END_TIME=$(date +%s)
        DURATION=$((END_TIME - START_TIME))
        
        echo -e "\n${RED}✗ macOS compilation check failed after ${DURATION}s${NC}"
        return 1
    fi
}

# Run tests
declare -A RESULTS

if [ "$PLATFORM" = "linux" ] || [ "$PLATFORM" = "all" ]; then
    if test_linux_build; then
        RESULTS["Linux"]=true
    else
        RESULTS["Linux"]=false
    fi
    echo ""
fi

if [ "$PLATFORM" = "macos-check" ] || [ "$PLATFORM" = "all" ]; then
    if test_macos_check; then
        RESULTS["macOS Check"]=true
    else
        RESULTS["macOS Check"]=false
    fi
    echo ""
fi

# Summary
echo -e "${CYAN}========================================${NC}"
echo -e "${CYAN}  Test Summary                          ${NC}"
echo -e "${CYAN}========================================${NC}\n"

ALL_PASSED=true
for test in "${!RESULTS[@]}"; do
    if [ "${RESULTS[$test]}" = true ]; then
        echo -e "  ${GREEN}✓ $test${NC}"
    else
        echo -e "  ${RED}✗ $test${NC}"
        ALL_PASSED=false
    fi
done

echo ""

if [ "$ALL_PASSED" = true ]; then
    echo -e "${GREEN}✓ All tests passed!${NC}"
    exit 0
else
    echo -e "${RED}✗ Some tests failed!${NC}"
    exit 1
fi

