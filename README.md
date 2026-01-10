# 几何射击 - Geometry Shooter 🎮

一款基于 Bevy 引擎的几何风格射击游戏，支持编译到 WebAssembly 在浏览器中运行。

## 🎯 游戏特性

- **几何风格**: 所有实体（飞机、敌人、子弹、护甲）都由多边形和弧形组成
- **自动卷轴**: 由下到上的自动滚动视角
- **跨平台**: 支持原生桌面和 Web 浏览器
- **数据持久化**: 使用浏览器 LocalStorage 保存游戏进度
- **充值系统**: 支持 HTTP 请求的充值功能

## 🎮 操作方式

| 按键 | 功能 |
|------|------|
| WASD / 方向键 | 移动飞机 |
| 空格 / Z | 射击 |
| ESC | 暂停游戏 |

## 🚀 快速开始

### 前置条件

- [Rust](https://rustup.rs/) (1.75+)
- [wasm-bindgen-cli](https://rustwasm.github.io/wasm-bindgen/)

### 安装依赖

```bash
# 添加 WASM 目标
rustup target add wasm32-unknown-unknown

# 安装 wasm-bindgen
cargo install wasm-bindgen-cli

# (可选) 安装本地服务器
cargo install basic-http-server
```

### 构建和运行

#### 原生版本
```bash
cargo run --release
```

#### Web 版本
```bash
# 使用构建脚本
./build.sh wasm   # macOS/Linux
.\build.ps1 wasm  # Windows

# 启动本地服务器
./build.sh serve
```

然后访问 http://localhost:4000

> 注意：字体文件 `assets/NotoSansCJKsc-Regular.otf` 被 `.gitignore` 忽略；构建脚本会在本地缺失时自动从官方仓库下载完整字体到 `assets/NotoSansCJKsc-Regular.full.otf`，并尝试根据 `src/ui` 实际使用到的文字生成子集字体输出到 `assets/NotoSansCJKsc-Regular.otf`（如本机未安装 `fontTools` 会退化为直接使用完整字体）。

## 📁 项目结构

```
shoot/
├── src/
│   ├── main.rs          # 原生入口
│   ├── lib.rs           # 库入口 & WASM 入口
│   ├── game/            # 游戏核心系统
│   │   ├── states.rs    # 游戏状态机
│   │   ├── scroll.rs    # 卷轴系统
│   │   └── collision.rs # 碰撞检测
│   ├── geometry/        # 几何系统
│   │   ├── shapes.rs    # 形状定义
│   │   └── renderer.rs  # 渲染器
│   ├── entities/        # 游戏实体
│   │   ├── player.rs    # 玩家
│   │   ├── enemy.rs     # 敌人
│   │   ├── bullet.rs    # 子弹
│   │   └── shield.rs    # 护盾
│   ├── storage/         # 存储系统
│   │   ├── web_storage.rs # LocalStorage
│   │   └── recharge.rs  # 充值系统
│   └── ui/              # 用户界面
│       ├── menu.rs      # 菜单
│       ├── hud.rs       # HUD
│       └── input.rs     # 输入处理
├── web/
│   ├── index.html       # Web 页面
│   └── style.css        # 样式
├── assets/              # 游戏资源
├── build.sh             # 构建脚本 (Unix)
├── build.ps1            # 构建脚本 (Windows)
└── Cargo.toml           # 项目配置
```

## 🔧 几何形状数据格式

游戏使用统一的 JSON 格式存储几何实体：

```rust
// 形状类型
enum GeometryShape {
    Polygon { vertices, color, fill, stroke_width },
    Arc { center, radius, start_angle, end_angle, color, stroke_width },
    Circle { center, radius, color, fill, stroke_width },
    Line { start, end, color, stroke_width },
}

// 实体蓝图
struct GeometryBlueprint {
    name: String,
    shapes: Vec<GeometryShape>,
    collision: CollisionShape,
    scale: f32,
}
```

## 🌐 WASM 文字输入

对于充值码输入（100字符），采用 HTML 覆盖层方案：

1. 在 WASM 中触发显示输入框
2. 使用浏览器原生 `<input>` 元素
3. 通过 JS 桥接将数据传回 Rust
4. 完美支持中文输入法

## 📦 GitHub Pages 部署

项目已配置 GitHub Actions，推送到 `main` 分支后会自动：

1. 构建 WASM 版本
2. 优化 WASM 大小
3. 部署到 GitHub Pages

访问: `https://<username>.github.io/shoot/`

### 手动部署

```bash
./build.sh all
# 将 dist/ 目录部署到任意静态托管服务
```

## 🛠 开发

```bash
# 检查代码
cargo clippy

# 格式化
cargo fmt

# 运行测试
cargo test
```

## 📄 许可证

MIT License

## 🙏 致谢

- [Bevy Engine](https://bevyengine.org/)
- [bevy_prototype_lyon](https://github.com/Nilirad/bevy_prototype_lyon)
- [wasm-bindgen](https://rustwasm.github.io/wasm-bindgen/)
