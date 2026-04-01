<p align="center">
  <pre align="center">
  _        ___    ___   ____     _     _
 | |      / _ \  / _ \ |  _ \   / \   | |
 | |     | | | || | | || |_) | / _ \  | |
 | |___  | |_| || |_| ||  __/ / ___ \ | |___
 |_____|  \___/  \___/ |_|   /_/   \_\|_____|
  </pre>
  <em>Rooted in code, Growing with loopal.</em><br/>
  Part of <a href="https://agentsmesh.ai">AgentsMesh.ai</a>
</p>

<p align="center">
  <a href="#installation">Installation</a> •
  <a href="#quick-start">Quick Start</a> •
  <a href="#features">Features</a> •
  <a href="#architecture">Architecture</a> •
  <a href="#configuration">Configuration</a> •
  <a href="#license">License</a>
</p>

<p align="center">
  English | <a href="./README.zh-CN.md">简体中文</a>
</p>

---

**Loopal** is a terminal-native AI coding agent built in Rust. It connects to LLM providers, reads and edits your codebase, runs commands, and orchestrates multi-agent workflows — all from inside your terminal with a rich TUI.

## Features

- 🚀 **Terminal-native TUI** — Rich interactive interface powered by [Ratatui](https://ratatui.rs), with Markdown rendering, syntax highlighting, and progress indicators.
- 🧠 **Multi-provider LLM support** — Works with OpenAI, Anthropic, and any OpenAI-compatible endpoint. Configurable thinking/reasoning modes (auto, effort levels, token budgets).
- 🔧 **Comprehensive tool suite** — File read/write/edit, multi-edit, apply-patch, grep, glob, ls, bash execution, background tasks, and fetch.
- 🤖 **Multi-agent orchestration** — Spawn sub-agents that run in parallel, communicate via message passing and pub/sub channels, with a shared task store.
- 🔌 **MCP integration** — First-class [Model Context Protocol](https://modelcontextprotocol.io/) support for connecting external tool servers.
- 🖥️ **ACP server mode** — Agent Client Protocol over stdin/stdout JSON-RPC for IDE integration (Zed, JetBrains, Neovim, etc.) via `--acp`.
- 🔒 **Sandbox & permissions** — Configurable sandbox policies for filesystem access, network, and command execution. Supervised or bypass permission modes.
- 🪝 **Lifecycle hooks** — Run custom scripts on agent events (session start, tool calls, etc.).
- 💾 **Session management** — Persist and resume sessions. Context compaction with smart truncation to stay within token limits.
- 🧩 **Context pipeline** — Middleware-based context processing with message size guards, smart compaction, and context guards.
- 📝 **Memory** — Cross-session memory that persists observations and preferences across conversations.
- 🗂️ **Skills** — Extend agent capabilities with project-specific skill definitions.
- 📋 **Plan mode** — Read-only exploration mode for safe planning before making changes.

## Installation

### Prerequisites

- **Bazel** 8.x (the repository root is Bazel-first; there is no root `Cargo.toml`)
- A Rust toolchain supported by `rules_rust` / Bazel
- An API key for your LLM provider (for example `ANTHROPIC_API_KEY`, `OPENAI_API_KEY`, or an Anthropic-compatible proxy key such as `OPUS_API_KEY`)

### Build from source

```bash
git clone https://github.com/AgentsMesh/Loopal.git
cd Loopal
bazel build //:loopal
```

The binary will be at `bazel-bin/loopal`.

For an optimized build:

```bash
bazel build //:loopal -c opt
```

You can also use the repository `Makefile` shortcuts:

```bash
make build
make release
make install
```

## Quick Start

```bash
# Navigate to your project
cd your-project

# Start Loopal (uses default model from config)
loopal

# Start with a specific model
loopal -m claude-sonnet-4-20250514

# Start in plan mode (read-only exploration)
loopal --plan

# Resume a previous session
loopal -r <session-id>

# Run with an initial prompt
loopal "explain the architecture of this project"

# Bypass permission prompts
loopal -P bypass
```

## Architecture

Loopal is built as a modular Rust workspace with clear separation of concerns:

```text
loopal
├── src/                        # Binary entry point, CLI, bootstrap
├── crates/
│   ├── loopal-kernel/          # Core kernel: provider registry, tool dispatch
│   ├── loopal-runtime/         # Agent loop, session management, frontend traits
│   ├── loopal-agent/           # Multi-agent registry, router, task store
│   ├── loopal-session/         # Session state, display messages, event handling
│   ├── loopal-tui/             # Terminal UI (Ratatui-based)
│   ├── loopal-provider/        # LLM provider implementations
│   ├── loopal-provider-api/    # Provider trait definitions
│   ├── loopal-protocol/        # Shared protocol types (events, messages)
│   ├── loopal-message/         # Message types and serialization
│   ├── loopal-config/          # Layered configuration system
│   ├── loopal-context/         # Context pipeline, compaction, system prompt
│   ├── loopal-memory/          # Cross-session persistent memory
│   ├── loopal-storage/         # Session persistence
│   ├── loopal-sandbox/         # Command & filesystem sandboxing
│   ├── loopal-hooks/           # Lifecycle hook system
│   ├── loopal-mcp/             # MCP client integration
│   ├── loopal-acp/             # ACP server for IDE integration
│   ├── loopal-git/             # Git operations & worktree management
│   ├── loopal-tool-api/        # Tool trait definitions
│   ├── loopal-error/           # Error types
│   └── tools/                  # Built-in tool implementations
│       ├── filesystem/         # read, write, edit, multi-edit, apply-patch,
│       │                       # grep, glob, ls, fetch, file-ops
│       ├── process/            # bash, background task execution
│       └── agent/              # ask-user, plan-mode
```

### Key design decisions

- **Layered configuration** — Settings merge from defaults → global config → project config → CLI flags.
- **Context pipeline** — Middleware chain processes messages before sending to the LLM, handling token limits, compaction, and guards.
- **Unified frontend** — Both TUI and ACP share the same `AgentFrontend` trait, making the agent loop UI-agnostic.
- **Sandbox-first** — File and command access goes through policy checks by default.

## Configuration

Loopal uses a layered configuration system based on `settings.json`, not `config.toml`.

Load order, from lowest to highest priority:

- plugin layers in `~/.loopal/plugins/<name>/`
- global config in `~/.loopal/`
- project config in `<project>/.loopal/`
- local overrides in `<project>/.loopal/settings.local.json` and `LOOPAL.local.md`
- environment variable overrides with the `LOOPAL_` prefix

The most common files are:

```text
~/.loopal/settings.json                 Global settings
~/.loopal/LOOPAL.md                     Global instructions
<project>/.loopal/settings.json         Project settings
<project>/.loopal/settings.local.json   Local overrides (gitignored)
<project>/LOOPAL.md                     Project instructions
<project>/.loopal/LOOPAL.local.md       Local instruction overrides
```

Example project config:

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

### Anthropic-compatible proxy example

If you are using a non-official Anthropic-compatible endpoint or local proxy, configure the Anthropic provider in `settings.json` and supply credentials through environment variables:

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

`base_url` accepts all of the following forms:

- `http://localhost:8080`
- `http://localhost:8080/v1`
- `http://localhost:8080/v1/messages`

Loopal normalizes these to the Anthropic messages endpoint internally.

For non-official Anthropic-compatible gateways, the Anthropic provider also supports optional compatibility headers via environment variables.

### Environment variables

| Variable | Description |
|---|---|
| `ANTHROPIC_API_KEY` | Anthropic API key |
| `ANTHROPIC_AUTH_TOKEN` | Optional Anthropic bearer token override |
| `ANTHROPIC_BASE_URL` | Anthropic base URL override |
| `ANTHROPIC_API_VERSION` | Override `anthropic-version` header |
| `ANTHROPIC_USER_AGENT` | Override `user-agent` header |
| `ANTHROPIC_EXTRA_HEADERS` | Extra Anthropic headers, separated by newline or `;`, using `name=value` or `name:value` |
| `OPENAI_API_KEY` | OpenAI API key |
| `GOOGLE_API_KEY` | Google API key |
| `OPUS_API_KEY` | Anthropic-compatible proxy key fallback |
| `OPUS_API_URL` | Anthropic-compatible proxy base URL fallback |
| `OPUS_BASE_URL` | Alternate Anthropic-compatible proxy base URL fallback |
| `OPUS_API_VERSION` | Proxy-specific Anthropic version override |
| `OPUS_API_USER_AGENT` | Proxy-specific user-agent override |
| `OPUS_AUTH_TOKEN` | Proxy bearer token override |
| `OPUS_EXTRA_HEADERS` | Proxy extra headers in the same format as `ANTHROPIC_EXTRA_HEADERS` |
| `LOOPAL_MODEL` | Override the default model |
| `LOOPAL_MAX_TURNS` | Override `max_turns` |
| `LOOPAL_PERMISSION_MODE` | Override `permission_mode` |
| `LOOPAL_SANDBOX` | Override `sandbox.policy` |

When `base_url` is non-official Anthropic infrastructure, or when `OPUS_API_URL` is set, Loopal may automatically enable a compatibility bearer header using the configured API key. If your gateway needs a different bearer token, set `OPUS_AUTH_TOKEN` or `ANTHROPIC_AUTH_TOKEN` explicitly.

## CLI Reference

```text
Usage: loopal [OPTIONS] [PROMPT]...

Arguments:
  [PROMPT]...           Initial prompt (non-interactive)

Options:
  -m, --model <MODEL>       Model to use
  -r, --resume <SESSION>    Resume a previous session
  -P, --permission <MODE>   Permission mode (supervised/bypass)
      --plan                Start in plan mode (read-only)
      --no-sandbox          Disable sandbox enforcement
      --acp                 Run as ACP server (stdin/stdout JSON-RPC)
  -h, --help                Print help
```

## License

Proprietary. Copyright (c) 2024-2026 AgentsMesh.ai. All Rights Reserved.

This software is licensed under a proprietary commercial license. See the [LICENSE](./LICENSE) file for the full terms. Unauthorized copying, modification, distribution, or use of this software is strictly prohibited.
