mod errors;
mod helpers;

pub use errors::{
    AgentOutput, ConfigError, HookError, LoopalError, McpError, ProviderError, StorageError,
    TerminateReason, ToolError,
};

pub type Result<T> = std::result::Result<T, LoopalError>;
