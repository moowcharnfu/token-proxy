# Token Proxy

跨平台桌面应用，用于管理多个 AI API Token 提供商。支持一键切换激活提供商、API 连通性测试、环境变量快速复制、Claude Code 无缝集成。

## 功能

- ✅ 添加/编辑/删除 Token 提供商
- ✅ API 连通性测试（自动识别 Anthropic / Gemini / Ollama / OpenAI 兼容四种格式）
- ✅ 一键切换当前激活的提供商
- ✅ 环境变量（`ANTHROPIC_API_KEY` / `ANTHROPIC_BASE_URL`）复制
- ✅ 一键应用到 Claude Code（自动写 `~/.claude/settings.json` + `config.json`）
- ✅ 跨平台：macOS / Linux / Windows
- ✅ 激活项自动滚动定位

## 预置提供商

首次启动时自动生成默认配置（按优先级排序）：

| 提供商 | API Base | 默认模型 |
|--------|----------|----------|
| DeepSeek | https://api.deepseek.com | deepseek-chat |
| 阿里通义 (DashScope) | https://dashscope.aliyuncs.com/compatible-mode/v1 | qwen-plus |
| 月之暗面 (Moonshot) | https://api.moonshot.cn/v1 | moonshot-v1-32k |
| 智谱AI (Zhipu) | https://open.bigmodel.cn/api/paas/v4 | glm-4-flash |
| 商汤日日新 (SenseNova) | https://token.sensenova.cn | senxin-1 |
| 百川智能 (Baichuan) | https://api.baichuan-ai.com/v1 | Baichuan3-Turbo |
| MiniMax | https://api.minimaxi.com/v1 | abab6.5 |
| Ollama (本地) | http://localhost:11434 | llama3.2 |
| OpenAI | https://api.openai.com/v1 | gpt-4o |
| Google Gemini | https://generativelanguage.googleapis.com/v1beta | gemini-2.0-flash |

> 配置文件位于 `~/.token-proxy/config.yaml`，首次运行后自动生成，修改后持久化保存。

## 开发

### 前置条件

- Node.js 18+
- Rust 1.70+
- Tauri CLI（`cargo install tauri-cli --version "^2"`）
- ```
  #中国源
  export RUSTUP_DIST_SERVER=https://mirrors.ustc.edu.cn/rust-static && export RUSTUP_UPDATE_ROOT=https://mirrors.ustc.edu.cn/rust-static/rustup && curl --proto '=https' --tlsv1.2 -sSf   https://sh.rustup.rs | sh
  ```

### 安装依赖

```bash
npm install
```

### 开发模式

```bash
npm run tauri dev
```

前端开发服务器：`http://localhost:1420/`

### 构建发布版本

```bash
npm run tauri build
```

### 运行 Rust 测试

```bash
cd src-tauri && cargo test
```

## 技术栈

- **前端**: Vue 3 + TypeScript + TailwindCSS
- **后端**: Tauri v2 (Rust)
- **配置**: YAML（`serde_yaml`）
- **HTTP 客户端**: reqwest（rustls-tls）
