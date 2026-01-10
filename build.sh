#!/bin/bash
# 本地构建和测试脚本

set -e

echo "🎮 几何射击 - 构建脚本"
echo "========================"

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

FONT_URL="https://github.com/notofonts/noto-cjk/raw/refs/heads/main/Sans/OTF/SimplifiedChinese/NotoSansCJKsc-Regular.otf"
FONT_PATH="assets/NotoSansCJKsc-Regular.otf"

# 检查命令是否存在
check_command() {
    if ! command -v $1 &> /dev/null; then
        echo -e "${RED}错误: 未找到 $1，请先安装${NC}"
        exit 1
    fi
}

# 确保字体存在（若缺失则下载）
ensure_font() {
    mkdir -p assets
    if [ -s "$FONT_PATH" ]; then
        return 0
    fi

    echo -e "${YELLOW}📥 字体缺失，正在下载: ${FONT_PATH}${NC}"
    local tmp="${FONT_PATH}.download"

    if command -v curl &> /dev/null; then
        curl -L --fail --retry 3 --retry-delay 1 -o "$tmp" "$FONT_URL"
    elif command -v wget &> /dev/null; then
        wget -O "$tmp" "$FONT_URL"
    elif command -v python3 &> /dev/null; then
        python3 - <<PY
import urllib.request
url = "$FONT_URL"
out = "$tmp"
urllib.request.urlretrieve(url, out)
PY
    else
        echo -e "${RED}错误: 无法下载字体（缺少 curl/wget/python3）${NC}"
        echo -e "${RED}请手动下载并放到 ${FONT_PATH}${NC}"
        exit 1
    fi

    if [ ! -s "$tmp" ]; then
        echo -e "${RED}错误: 字体下载失败（文件为空）${NC}"
        rm -f "$tmp"
        exit 1
    fi

    mv "$tmp" "$FONT_PATH"
    echo -e "${GREEN}✓ 字体下载完成${NC}"
}

# 安装依赖
install_deps() {
    echo -e "${BLUE}📦 检查依赖...${NC}"
    
    check_command cargo
    check_command rustup
    
    # 检查 wasm32 目标
    if ! rustup target list --installed | grep -q "wasm32-unknown-unknown"; then
        echo -e "${YELLOW}安装 wasm32-unknown-unknown 目标...${NC}"
        rustup target add wasm32-unknown-unknown
    fi
    
    # 检查 wasm-bindgen-cli
    if ! command -v wasm-bindgen &> /dev/null; then
        echo -e "${YELLOW}安装 wasm-bindgen-cli...${NC}"
        cargo install wasm-bindgen-cli
    fi
    
    # 检查 basic-http-server（可选）
    if ! command -v basic-http-server &> /dev/null; then
        echo -e "${YELLOW}安装 basic-http-server（用于本地测试）...${NC}"
        cargo install basic-http-server
    fi

    ensure_font
    
    echo -e "${GREEN}✓ 依赖检查完成${NC}"
}

# 构建原生版本
build_native() {
    echo -e "${BLUE}🔨 构建原生版本...${NC}"
    ensure_font
    cargo build --release
    echo -e "${GREEN}✓ 原生版本构建完成${NC}"
}

# 构建 WASM 版本
build_wasm() {
    echo -e "${BLUE}🔨 构建 WASM 版本...${NC}"
    ensure_font
    
    # 构建 WASM
    cargo build --release --target wasm32-unknown-unknown --no-default-features --features web
    
    # 生成绑定
    echo -e "${BLUE}📎 生成 wasm-bindgen 绑定...${NC}"
    wasm-bindgen \
        --out-dir dist \
        --target web \
        --no-typescript \
        target/wasm32-unknown-unknown/release/shoot.wasm
    
    # 复制 web 资源
    echo -e "${BLUE}📁 复制 web 资源...${NC}"
    cp web/index.html dist/
    cp web/style.css dist/
    cp -r assets dist/
    
    echo -e "${GREEN}✓ WASM 版本构建完成${NC}"
    echo -e "${BLUE}输出目录: dist/${NC}"
}

# 优化 WASM
optimize_wasm() {
    echo -e "${BLUE}🔧 优化 WASM...${NC}"
    
    if command -v wasm-opt &> /dev/null; then
        wasm-opt -Oz -o dist/shoot_bg.wasm dist/shoot_bg.wasm
        echo -e "${GREEN}✓ WASM 优化完成${NC}"
    else
        echo -e "${YELLOW}⚠ wasm-opt 未安装，跳过优化${NC}"
        echo -e "${YELLOW}  安装: brew install binaryen 或 apt install binaryen${NC}"
    fi
}

# 运行本地服务器
serve() {
    echo -e "${BLUE}🌐 启动本地服务器...${NC}"
    echo -e "${GREEN}访问 http://localhost:4000${NC}"
    echo -e "${YELLOW}按 Ctrl+C 停止服务器${NC}"
    
    if command -v basic-http-server &> /dev/null; then
        basic-http-server dist -a 0.0.0.0:4000
    elif command -v python3 &> /dev/null; then
        cd dist && python3 -m http.server 4000
    else
        echo -e "${RED}错误: 未找到 HTTP 服务器${NC}"
        exit 1
    fi
}

# 运行原生版本
run_native() {
    echo -e "${BLUE}🎮 运行原生版本...${NC}"
    ensure_font
    cargo run --release
}

# 清理
clean() {
    echo -e "${BLUE}🧹 清理构建文件...${NC}"
    cargo clean
    rm -rf dist
    echo -e "${GREEN}✓ 清理完成${NC}"
}

# 显示帮助
show_help() {
    echo "用法: ./build.sh [命令]"
    echo ""
    echo "命令:"
    echo "  deps      安装/检查依赖"
    echo "  native    构建原生版本"
    echo "  wasm      构建 WASM 版本"
    echo "  optimize  优化 WASM (需要 wasm-opt)"
    echo "  serve     启动本地服务器测试 WASM"
    echo "  run       运行原生版本"
    echo "  all       构建所有版本"
    echo "  clean     清理构建文件"
    echo "  help      显示帮助"
    echo ""
    echo "示例:"
    echo "  ./build.sh deps      # 安装依赖"
    echo "  ./build.sh wasm      # 构建 WASM"
    echo "  ./build.sh serve     # 本地测试"
}

# 主函数
main() {
    cd "$(dirname "$0")"
    
    case "${1:-help}" in
        deps)
            install_deps
            ;;
        native)
            build_native
            ;;
        wasm)
            install_deps
            build_wasm
            ;;
        optimize)
            optimize_wasm
            ;;
        serve)
            if [ ! -d "dist" ]; then
                echo -e "${YELLOW}dist 目录不存在，先构建 WASM...${NC}"
                build_wasm
            fi
            serve
            ;;
        run)
            run_native
            ;;
        all)
            install_deps
            build_native
            build_wasm
            optimize_wasm
            echo -e "${GREEN}✓ 所有构建完成${NC}"
            ;;
        clean)
            clean
            ;;
        help|--help|-h)
            show_help
            ;;
        *)
            echo -e "${RED}未知命令: $1${NC}"
            show_help
            exit 1
            ;;
    esac
}

main "$@"
