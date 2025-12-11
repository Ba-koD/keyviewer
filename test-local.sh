#!/bin/bash

# Local Build Test Script for KeyQueueViewer (Linux/macOS)
# This script allows testing builds locally before pushing to GitHub Actions

set -e

PLATFORM="all"
CLEAN=false

# Color codes
CYAN='\033[0;36m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
GRAY='\033[0;37m'
NC='\033[0m' # No Color

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
        *)
            echo "Usage: $0 [--platform windows|linux|macos|all] [--clean]"
            exit 1
            ;;
    esac
done

echo -e "${CYAN}========================================${NC}"
echo -e "${CYAN}  KeyQueueViewer - Local Build Test    ${NC}"
echo -e "${CYAN}========================================${NC}"
echo ""

# Detect OS
OS=$(uname -s)
echo "Detected OS: $OS"

# Check Docker installation
DOCKER_INSTALLED=false
if command -v docker &> /dev/null; then
    DOCKER_VERSION=$(docker --version)
    DOCKER_INSTALLED=true
    echo -e "${GREEN}✓ Docker found: $DOCKER_VERSION${NC}"
else
    echo -e "${YELLOW}⚠ Docker not found - Linux builds will be skipped${NC}"
fi

echo ""

# Function to test native build
test_native_build() {
    echo -e "${CYAN}========================================${NC}"
    echo -e "${CYAN}  Testing Native Build ($OS)           ${NC}"
    echo -e "${CYAN}========================================${NC}"
    echo ""
    
    # Check Rust installation
    if ! command -v cargo &> /dev/null; then
        echo -e "${RED}✗ Rust not installed!${NC}"
        echo -e "${YELLOW}  Install from https://rustup.rs/${NC}"
        return 1
    fi
    
    RUST_VERSION=$(cargo --version)
    echo -e "${GREEN}✓ Rust found: $RUST_VERSION${NC}"
    
    # Check Tauri CLI
    if ! command -v cargo-tauri &> /dev/null; then
        echo -e "${YELLOW}Installing Tauri CLI...${NC}"
        cargo install tauri-cli
    fi
    
    echo ""
    echo -e "${YELLOW}Running build...${NC}"
    echo -e "${GRAY}This may take several minutes on first build...${NC}"
    
    START_TIME=$(date +%s)
    
    if cargo tauri build; then
        END_TIME=$(date +%s)
        DURATION=$((END_TIME - START_TIME))
        
        echo ""
        echo -e "${GREEN}✓ Build completed successfully in ${DURATION} seconds${NC}"
        
        # Check for build artifacts
        BUNDLE_PATH="src-tauri/target/release/bundle"
        
        if [ "$OS" == "Darwin" ]; then
            # macOS
            if [ -d "$BUNDLE_PATH/dmg" ]; then
                echo -e "${GREEN}✓ DMG files found:${NC}"
                find "$BUNDLE_PATH/dmg" -name "*.dmg" -exec bash -c 'SIZE=$(du -h "$1" | cut -f1); echo -e "  - $(basename \"$1\") ($SIZE)"' _ {} \;
            fi
            
            if [ -d "$BUNDLE_PATH/macos" ]; then
                echo -e "${GREEN}✓ App bundles found:${NC}"
                find "$BUNDLE_PATH/macos" -name "*.app" -maxdepth 1 -exec bash -c 'SIZE=$(du -sh "$1" | cut -f1); echo -e "  - $(basename \"$1\") ($SIZE)"' _ {} \;
            fi
        elif [ "$OS" == "Linux" ]; then
            # Linux
            if [ -d "$BUNDLE_PATH/deb" ]; then
                echo -e "${GREEN}✓ DEB packages found:${NC}"
                find "$BUNDLE_PATH/deb" -name "*.deb" -exec bash -c 'SIZE=$(du -h "$1" | cut -f1); echo -e "  - $(basename \"$1\") ($SIZE)"' _ {} \;
            fi
            
            if [ -d "$BUNDLE_PATH/appimage" ]; then
                echo -e "${GREEN}✓ AppImage files found:${NC}"
                find "$BUNDLE_PATH/appimage" -name "*.AppImage" -exec bash -c 'SIZE=$(du -h "$1" | cut -f1); echo -e "  - $(basename \"$1\") ($SIZE)"' _ {} \;
            fi
        fi
        
        echo ""
        return 0
    else
        echo ""
        echo -e "${RED}✗ Build failed!${NC}"
        return 1
    fi
}

# Function to test Linux build in Docker
test_linux_docker_build() {
    if [ "$DOCKER_INSTALLED" = false ]; then
        echo -e "${YELLOW}⚠ Skipping Linux Docker build - Docker not installed${NC}"
        echo -e "${GRAY}   Install Docker to enable Linux build testing${NC}"
        return 0
    fi
    
    echo -e "${CYAN}========================================${NC}"
    echo -e "${CYAN}  Testing Linux Build (Docker)         ${NC}"
    echo -e "${CYAN}========================================${NC}"
    echo ""
    
    echo -e "${YELLOW}Building Docker image...${NC}"
    docker build -f Dockerfile.linux -t keyviewer-linux-build .
    
    echo ""
    echo -e "${YELLOW}Running build in Docker container...${NC}"
    echo -e "${GRAY}This may take several minutes on first run...${NC}"
    
    START_TIME=$(date +%s)
    
    if docker run --rm \
        -v "$(pwd):/app" \
        -v "keyviewer-cargo-cache:/root/.cargo/registry" \
        -v "keyviewer-target-cache:/app/src-tauri/target" \
        keyviewer-linux-build \
        bash -c "cd /app && cargo tauri build"; then
        
        END_TIME=$(date +%s)
        DURATION=$((END_TIME - START_TIME))
        
        echo ""
        echo -e "${GREEN}✓ Linux build completed successfully in ${DURATION} seconds${NC}"
        
        # Check for build artifacts
        BUNDLE_PATH="src-tauri/target/release/bundle"
        
        if [ -d "$BUNDLE_PATH/deb" ]; then
            echo -e "${GREEN}✓ DEB packages found:${NC}"
            find "$BUNDLE_PATH/deb" -name "*.deb" -exec bash -c 'SIZE=$(du -h "$1" | cut -f1); echo -e "  - $(basename \"$1\") ($SIZE)"' _ {} \;
        fi
        
        if [ -d "$BUNDLE_PATH/appimage" ]; then
            echo -e "${GREEN}✓ AppImage files found:${NC}"
            find "$BUNDLE_PATH/appimage" -name "*.AppImage" -exec bash -c 'SIZE=$(du -h "$1" | cut -f1); echo -e "  - $(basename \"$1\") ($SIZE)"' _ {} \;
        fi
        
        echo ""
        return 0
    else
        echo ""
        echo -e "${RED}✗ Linux build failed!${NC}"
        return 1
    fi
}

# Clean previous builds if requested
if [ "$CLEAN" = true ]; then
    echo -e "${YELLOW}Cleaning previous builds...${NC}"
    
    if [ -d "dist" ]; then
        rm -rf dist
        echo -e "${GREEN}✓ Cleaned dist/${NC}"
    fi
    
    if [ -d "src-tauri/target" ]; then
        rm -rf src-tauri/target
        echo -e "${GREEN}✓ Cleaned src-tauri/target/${NC}"
    fi
    
    if [ "$DOCKER_INSTALLED" = true ]; then
        echo -e "${YELLOW}Cleaning Docker volumes...${NC}"
        docker volume rm keyviewer-cargo-cache keyviewer-target-cache 2>/dev/null || true
        echo -e "${GREEN}✓ Cleaned Docker volumes${NC}"
    fi
    
    echo ""
fi

# Run tests based on platform parameter
ALL_SUCCESS=true

if [ "$PLATFORM" == "all" ] || [ "$PLATFORM" == "$(echo $OS | tr '[:upper:]' '[:lower:]')" ]; then
    if ! test_native_build; then
        ALL_SUCCESS=false
    fi
fi

if [ "$PLATFORM" == "all" ] || [ "$PLATFORM" == "linux" ]; then
    if [ "$OS" != "Linux" ]; then
        if ! test_linux_docker_build; then
            ALL_SUCCESS=false
        fi
    fi
fi

# Summary
echo -e "${CYAN}========================================${NC}"
echo -e "${CYAN}  Build Test Summary                    ${NC}"
echo -e "${CYAN}========================================${NC}"
echo ""

if [ "$ALL_SUCCESS" = true ]; then
    echo -e "${GREEN}✓ All builds completed successfully!${NC}"
    echo ""
    echo -e "${YELLOW}Next steps:${NC}"
    echo -e "${GRAY}  1. Review the build artifacts${NC}"
    echo -e "${GRAY}  2. Test the executables${NC}"
    echo -e "${GRAY}  3. Push to GitHub to trigger Actions${NC}"
else
    echo -e "${RED}✗ Some builds failed - please check the errors above${NC}"
    echo ""
    echo -e "${YELLOW}Troubleshooting tips:${NC}"
    echo -e "${GRAY}  1. Check Rust installation: cargo --version${NC}"
    echo -e "${GRAY}  2. Check Tauri CLI: cargo tauri --version${NC}"
    echo -e "${GRAY}  3. Review build logs above for specific errors${NC}"
    echo -e "${GRAY}  4. Try running with --clean to start fresh${NC}"
fi

echo ""
echo -e "${GREEN}✓ Done!${NC}"
echo ""

# Exit with appropriate code
if [ "$ALL_SUCCESS" = true ]; then
    exit 0
else
    exit 1
fi

