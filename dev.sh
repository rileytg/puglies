#!/bin/bash

# PLGUI Development Script

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

usage() {
    echo "Usage: ./dev.sh <command>"
    echo ""
    echo "Commands:"
    echo "  start       Start the Tauri dev server"
    echo "  stop        Stop any running dev processes"
    echo "  build       Build the app for production"
    echo "  build-debug Build the app in debug mode"
    echo "  check       Run cargo check on Rust code"
    echo "  lint        Run TypeScript type checking"
    echo "  clean       Clean build artifacts"
    echo "  logs        Show recent dev logs"
    echo ""
}

start() {
    echo -e "${GREEN}Starting PLGUI dev server...${NC}"

    # Kill any existing processes on our ports
    lsof -ti:1420 | xargs kill -9 2>/dev/null || true

    # Start tauri dev
    pnpm tauri dev
}

stop() {
    echo -e "${YELLOW}Stopping dev processes...${NC}"

    # Kill vite dev server
    lsof -ti:1420 | xargs kill -9 2>/dev/null || true

    # Kill any plgui processes
    pkill -f "plgui" 2>/dev/null || true

    echo -e "${GREEN}Done${NC}"
}

build() {
    echo -e "${GREEN}Building PLGUI for production...${NC}"
    pnpm tauri build
}

build_debug() {
    echo -e "${GREEN}Building PLGUI in debug mode...${NC}"
    pnpm tauri build --debug
}

check() {
    echo -e "${GREEN}Running cargo check...${NC}"
    cd src-tauri && cargo check
}

lint() {
    echo -e "${GREEN}Running TypeScript check...${NC}"
    pnpm exec tsc --noEmit
}

clean() {
    echo -e "${YELLOW}Cleaning build artifacts...${NC}"
    rm -rf dist
    rm -rf src-tauri/target
    rm -rf node_modules/.vite
    echo -e "${GREEN}Done${NC}"
}

logs() {
    echo -e "${GREEN}Recent build output:${NC}"
    if [ -d "src-tauri/target" ]; then
        ls -lt src-tauri/target/debug/bundle/macos/*.app 2>/dev/null || echo "No app bundle found"
    else
        echo "No target directory found. Run build first."
    fi
}

# Main
case "${1:-}" in
    start)
        start
        ;;
    stop)
        stop
        ;;
    build)
        build
        ;;
    build-debug)
        build_debug
        ;;
    check)
        check
        ;;
    lint)
        lint
        ;;
    clean)
        clean
        ;;
    logs)
        logs
        ;;
    *)
        usage
        exit 1
        ;;
esac
