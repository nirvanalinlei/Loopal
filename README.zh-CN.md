<p align="center">
  <pre align="center">
  _        ___    ___   ____     _     _
 | |      / _ \  / _ \ |  _ \   / \   | |
 | |     | | | || | | || |_) | / _ \  | |
 | |___  | |_| || |_| ||  __/ / ___ \ | |___
 |_____|  \___/  \___/ |_|   /_/   \_\|_____|
  </pre>
  <em>扎根代码，伴随 loopal 持续生长。</em><br/>
  属于 <a href="https://agentsmesh.ai">AgentsMesh.ai</a>
</p>

<p align="center">
  <a href="#installation-zh">安装</a> •
  <a href="#quick-start-zh">快速开始</a> •
  <a href="#features-zh">功能特性</a> •
  <a href="#architecture-zh">架构</a> •
  <a href="#configuration-zh">配置</a> •
  <a href="#license-zh">许可协议</a>
</p>

<p align="center">
  <a href="./README.md">English</a> | 简体中文
</p>

---

**Loopal** 是一个基于 Rust 构建的终端原生 AI 编码 agent。它可以连接 LLM 提供方，读取和编辑代码库，运行命令，并在终端内通过丰富的 TUI 编排多 agent 工作流。

<a id="features-zh"></a>
## 功能特性

- 🚀 **终端原生 TUI**：基于 [Ratatui](https://ratatui.rs) 提供丰富交互界面，支持 Markdown 渲染、语法高亮和进度指示。
- 🧠 **多提供方 LLM 支持**：可对接 OpenAI、Anthropic 以及任意 OpenAI-compatible 端点；支持可配置的 thinking / reasoning 模式，例如 auto、不同 effort 等级和 token budget。
- 🔧 **完整工具集**：内置文件读取、写入、编辑、多点编辑、apply-patch、grep、glob、ls、bash 执行、后台任务和 fetch。
- 🤖 **多 agent 编排**：可并行拉起子 agent，通过消息传递和 pub/sub 通道通信，并共享任务存储。
- 🔌 **MCP 集成**：原生支持 [Model Context Protocol](https://modelcontextprotocol.io/)，可接入外部工具服务器。
- 🖥️ **ACP 服务模式**：通过 `--acp` 以 stdin/stdout JSON-RPC 运行 Agent Client Protocol，方便接入 Zed、JetBrains、Neovim 等 IDE。
- 🔒 **沙箱与权限**：文件系统访问、网络和命令执行都可配置沙箱策略；支持 supervised 或 bypass 权限模式。
- 🪝 **生命周期 Hook**：可在 session 启动、工具调用等事件上运行自定义脚本。
- 💾 **会话管理**：支持会话持久化与恢复，并通过智能截断与压缩控制上下文长度。
- 🧩 **上下文管线**：采用中间件式上下文处理流程，支持消息大小保护、智能压缩和上下文守卫。
- 📝 **记忆能力**：跨会话持久化观察结果和偏好。
- 🗂️ **Skills**：通过项目级 skill 定义扩展 agent 能力。
- 📋 **Plan 模式**：只读探索模式，适合在真正动手前先进行安全规划。

<a id="installation-zh"></a>
## 安装

### 前置要求

- **Bazel** 8.x（仓库根目录应视为 Bazel-first；根目录没有 `Cargo.toml`）
- 一个被 `rules_rust` / Bazel 支持的 Rust 工具链
- 你的 LLM 提供方 API key，例如 `ANTHROPIC_API_KEY`、`OPENAI_API_KEY`，或 Anthropic-compatible 代理使用的 `OPUS_API_KEY`

### 从源码构建

```bash
git clone https://github.com/AgentsMesh/Loopal.git
cd Loopal
bazel build //:loopal
```

生成的二进制位于 `bazel-bin/loopal`。

如果需要优化构建：

```bash
bazel build //:loopal -c opt
```

也可以使用仓库中的 `Makefile` 快捷命令：

```bash
make build
make release
make install
```

<a id="quick-start-zh"></a>
## 快速开始

```bash
# 进入你的项目目录
cd your-project

# 启动 Loopal（使用配置中的默认模型）
loopal

# 指定模型启动
loopal -m claude-sonnet-4-20250514

# 以 plan 模式启动（只读探索）
loopal --plan

# 恢复之前的会话
loopal -r <session-id>

# 带初始提示词启动
loopal "explain the architecture of this project"

# 跳过权限提示
loopal -P bypass
```

<a id="architecture-zh"></a>
## 架构

Loopal 采用模块化 Rust 代码结构，并清晰分离职责：

```text
loopal
├── src/                        # 二进制入口、CLI、启动逻辑
├── crates/
│   ├── loopal-kernel/          # 核心内核：provider registry、工具分发
│   ├── loopal-runtime/         # Agent 循环、会话管理、前端 trait
│   ├── loopal-agent/           # 多 agent registry、router、任务存储
│   ├── loopal-session/         # 会话状态、展示消息、事件处理
│   ├── loopal-tui/             # 终端 UI（基于 Ratatui）
│   ├── loopal-provider/        # LLM provider 实现
│   ├── loopal-provider-api/    # Provider trait 定义
│   ├── loopal-protocol/        # 共享协议类型（事件、消息）
│   ├── loopal-message/         # 消息类型与序列化
│   ├── loopal-config/          # 分层配置系统
│   ├── loopal-context/         # 上下文管线、压缩、system prompt
│   ├── loopal-memory/          # 跨会话持久化记忆
│   ├── loopal-storage/         # 会话持久化
│   ├── loopal-sandbox/         # 命令与文件系统沙箱
│   ├── loopal-hooks/           # 生命周期 Hook 系统
│   ├── loopal-mcp/             # MCP 客户端集成
│   ├── loopal-acp/             # IDE 集成用 ACP 服务
│   ├── loopal-git/             # Git 操作与 worktree 管理
│   ├── loopal-tool-api/        # 工具 trait 定义
│   ├── loopal-error/           # 错误类型
│   └── tools/                  # 内置工具实现
│       ├── filesystem/         # read、write、edit、multi-edit、apply-patch、
│       │                       # grep、glob、ls、fetch、file-ops
│       ├── process/            # bash、后台任务执行
│       └── agent/              # ask-user、plan-mode
```

### 核心设计决策

- **分层配置**：设置按照 defaults → global config → project config → CLI flags 的顺序合并。
- **上下文管线**：消息在发给 LLM 前会经过中间件链，处理 token 限制、压缩和保护逻辑。
- **统一前端**：TUI 和 ACP 共用同一套 `AgentFrontend` trait，使 agent loop 与 UI 解耦。
- **沙箱优先**：文件和命令访问默认都经过策略检查。

<a id="configuration-zh"></a>
## 配置

Loopal 使用基于 `settings.json` 的分层配置系统，而不是 `config.toml`。

从低到高的加载顺序如下：

- `~/.loopal/plugins/<name>/` 中的插件层
- `~/.loopal/` 中的全局配置
- `<project>/.loopal/` 中的项目配置
- `<project>/.loopal/settings.local.json` 和 `LOOPAL.local.md` 中的本地覆盖
- 使用 `LOOPAL_` 前缀的环境变量覆盖

最常见的文件包括：

```text
~/.loopal/settings.json                 全局设置
~/.loopal/LOOPAL.md                     全局指令
<project>/.loopal/settings.json         项目设置
<project>/.loopal/settings.local.json   本地覆盖（建议加入 gitignore）
<project>/LOOPAL.md                     项目级指令
<project>/.loopal/LOOPAL.local.md       本地指令覆盖
```

项目配置示例：

```json
{
  "model": "claude-sonnet-4-20250514",
  "max_turns": 200,
  "permission_mode": "supervised",
  "max_context_tokens": 120000,
  "thinking": {
    "type": "auto"
  },
  "mcp_servers": {
    "my-server": {
      "type": "stdio",
      "command": "npx",
      "args": ["-y", "@my/mcp-server"]
    }
  },
  "sandbox": {
    "policy": "strict"
  }
}
```

### Anthropic 兼容代理示例

如果你使用的是非官方 Anthropic-compatible 端点或本地代理，请在 `settings.json` 中配置 Anthropic provider，并通过环境变量提供凭据：

```json
{
  "model": "claude-opus-4-6",
  "providers": {
    "anthropic": {
      "api_key_env": "OPUS_API_KEY",
      "base_url": "http://localhost:8080"
    }
  }
}
```

`base_url` 支持以下三种形式：

- `http://localhost:8080`
- `http://localhost:8080/v1`
- `http://localhost:8080/v1/messages`

Loopal 内部会把它们统一规范到 Anthropic messages endpoint。

对于非官方 Anthropic-compatible 网关，还可以通过环境变量启用额外兼容 header。

### 环境变量

| 变量 | 说明 |
|---|---|
| `ANTHROPIC_API_KEY` | Anthropic API key |
| `ANTHROPIC_AUTH_TOKEN` | 可选的 Anthropic bearer token 覆盖 |
| `ANTHROPIC_BASE_URL` | Anthropic base URL 覆盖 |
| `ANTHROPIC_API_VERSION` | 覆盖 `anthropic-version` header |
| `ANTHROPIC_USER_AGENT` | 覆盖 `user-agent` header |
| `ANTHROPIC_EXTRA_HEADERS` | 额外 Anthropic headers；支持换行或 `;` 分隔，格式为 `name=value` 或 `name:value` |
| `OPENAI_API_KEY` | OpenAI API key |
| `GOOGLE_API_KEY` | Google API key |
| `OPUS_API_KEY` | Anthropic-compatible 代理 key fallback |
| `OPUS_API_URL` | Anthropic-compatible 代理 base URL fallback |
| `OPUS_BASE_URL` | 备用 Anthropic-compatible 代理 base URL fallback |
| `OPUS_API_VERSION` | 代理侧 Anthropic version 覆盖 |
| `OPUS_API_USER_AGENT` | 代理侧 user-agent 覆盖 |
| `OPUS_AUTH_TOKEN` | 代理 bearer token 覆盖 |
| `OPUS_EXTRA_HEADERS` | 代理额外 headers，格式与 `ANTHROPIC_EXTRA_HEADERS` 相同 |
| `LOOPAL_MODEL` | 覆盖默认模型 |
| `LOOPAL_MAX_TURNS` | 覆盖 `max_turns` |
| `LOOPAL_PERMISSION_MODE` | 覆盖 `permission_mode` |
| `LOOPAL_SANDBOX` | 覆盖 `sandbox.policy` |

当 `base_url` 指向非官方 Anthropic 基础设施，或设置了 `OPUS_API_URL` 时，Loopal 可能会自动用当前 API key 启用兼容 bearer header。如果你的网关需要不同的 bearer token，请显式设置 `OPUS_AUTH_TOKEN` 或 `ANTHROPIC_AUTH_TOKEN`。

## CLI 参考

```text
Usage: loopal [OPTIONS] [PROMPT]...

Arguments:
  [PROMPT]...           初始提示词（非交互）

Options:
  -m, --model <MODEL>       使用的模型
  -r, --resume <SESSION>    恢复之前的会话
  -P, --permission <MODE>   权限模式（supervised/bypass）
      --plan                以 plan 模式启动（只读）
      --no-sandbox          禁用沙箱强制执行
      --acp                 以 ACP server 模式运行（stdin/stdout JSON-RPC）
  -h, --help                打印帮助
```

<a id="license-zh"></a>
## 许可协议

专有软件。Copyright (c) 2024-2026 AgentsMesh.ai. All Rights Reserved.

本软件采用专有商业许可协议。完整条款请参见 [LICENSE](./LICENSE) 文件。未经授权，严禁复制、修改、分发或使用本软件。
