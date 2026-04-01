# 更新日志

<a href="./CHANGELOG.md">English</a> | 简体中文

这个文件用于记录当前 fork 的重要变更。

格式参考 Keep a Changelog，但在仍处于预发布阶段或受环境限制的变更上，也会补充验证范围与已知限制说明。

## 未发布（2026-04-01）

### 新增
- Anthropic provider 现在支持将 `base_url` 规范化处理为以下三种输入形式：根 URL、`/v1`、完整 `/v1/messages` endpoint。
- 新增可选兼容请求控制项，包括 `authorization` bearer token、`anthropic-version`、`user-agent` 以及额外请求 header。
- Anthropic provider registry 新增对代理型环境变量的 fallback 支持，例如 `OPUS_API_KEY`、`OPUS_API_URL`、`OPUS_BASE_URL`、`OPUS_API_VERSION`、`OPUS_API_USER_AGENT`、`OPUS_AUTH_TOKEN` 与 `OPUS_EXTRA_HEADERS`。
- 新增针对完整 `/v1/messages` base URL、`/v1` base URL、兼容 header，以及通过 `OPUS_*` 环境变量完成 provider 注册的定向测试覆盖。

### 变更
- 修正 `README.md`、`LOOPAL.md` 与 `CLAUDE.md` 中的仓库文档，使其反映真实情况：仓库根目录是 Bazel-first，且不存在根级 `Cargo.toml` 或 `Cargo.lock`。
- 更新配置说明，明确真实配置模型是 `settings.json` / `settings.local.json`，而不是根级 `config.toml` 工作流。
- 在项目文档中补充 Anthropic-compatible 代理示例以及相关环境变量说明。

### 验证情况
- 已确认已安装的 `loopal` 二进制可以运行 `--version` 和 `--help`。
- 已确认 mock provider 的 `--headless --plan` 路径可在本地运行。
- 已通过定向代码审查与 provider 级测试覆盖验证兼容补丁方向。
- 已确认当前文档与观察到的 Bazel-first 仓库结构一致。

### 已知限制
- 当前环境下，Windows Bazel / rules_rust / MSVC 构建链稳定性问题仍未解决。
- 尚未完成“基于该 fork 新构建出的 Windows 二进制”的完整端到端验证。
- 对非官方 Anthropic-compatible 基础设施的自动 bearer fallback 仍然是启发式逻辑，某些环境可能仍需手动覆盖。
