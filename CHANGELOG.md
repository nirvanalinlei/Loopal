# Changelog

English | <a href="./CHANGELOG.zh-CN.md">简体中文</a>

All notable changes to this fork will be documented in this file.

The format is based on Keep a Changelog principles, but entries may also include validation scope and known limitations when a change is still pre-release or environment-constrained.

## Unreleased (2026-04-01)

### Added
- Anthropic provider support for normalizing `base_url` values provided as a root URL, `/v1`, or a full `/v1/messages` endpoint.
- Optional compatibility request controls for `authorization` bearer tokens, `anthropic-version`, `user-agent`, and extra request headers.
- Anthropic provider registry fallback support for proxy-oriented environment variables such as `OPUS_API_KEY`, `OPUS_API_URL`, `OPUS_BASE_URL`, `OPUS_API_VERSION`, `OPUS_API_USER_AGENT`, `OPUS_AUTH_TOKEN`, and `OPUS_EXTRA_HEADERS`.
- Targeted tests covering full `/v1/messages` base URLs, `/v1` base URLs, compatibility headers, and provider registration via `OPUS_*` environment variables.

### Changed
- Corrected repository documentation in `README.md`, `LOOPAL.md`, and `CLAUDE.md` to reflect that the repository root is Bazel-first and does not contain a root `Cargo.toml` or `Cargo.lock`.
- Updated configuration guidance to describe the actual `settings.json` / `settings.local.json` model instead of a root `config.toml` workflow.
- Added Anthropic-compatible proxy examples and environment-variable guidance to the project documentation.

### Validation
- Verified that the installed `loopal` binary can run `--version` and `--help`.
- Verified that the mock provider `--headless --plan` path runs locally.
- Verified the compatibility patch direction through targeted code review and provider-level test coverage.
- Verified that the docs now align with the observed Bazel-first repository structure.

### Known Limitations
- Windows Bazel / rules_rust / MSVC build stability remains unresolved in the current environment.
- Full end-to-end validation with a newly built Windows binary from this fork has not yet been completed.
- Automatic bearer fallback for non-official Anthropic-compatible infrastructure remains heuristic and may require environment-specific overrides.
