pub mod apply_patch;
pub mod ask_user;
pub mod background;
pub mod bash;
pub mod diff;
pub mod edit;
pub mod fetch;
pub mod file_ops;
pub mod glob;
pub mod grep;
pub mod grep_format;
pub mod grep_search;
pub mod ls;
pub mod ls_format;
pub mod multi_edit;
pub mod plan_mode;
pub mod read;
pub mod read_pdf;
pub mod web_search;
pub mod write;

use crate::registry::ToolRegistry;

/// Register all built-in tools with the given registry.
pub fn register_all(registry: &mut ToolRegistry) {
    registry.register(Box::new(apply_patch::ApplyPatchTool));
    registry.register(Box::new(read::ReadTool));
    registry.register(Box::new(write::WriteTool));
    registry.register(Box::new(edit::EditTool));
    registry.register(Box::new(multi_edit::MultiEditTool));
    registry.register(Box::new(glob::GlobTool));
    registry.register(Box::new(grep::GrepTool));
    registry.register(Box::new(bash::BashTool));
    registry.register(Box::new(ls::LsTool));
    registry.register(Box::new(fetch::FetchTool));
    registry.register(Box::new(web_search::WebSearchTool));
    registry.register(Box::new(ask_user::AskUserTool));
    registry.register(Box::new(plan_mode::EnterPlanModeTool));
    registry.register(Box::new(plan_mode::ExitPlanModeTool));
    registry.register(Box::new(background::task_output::TaskOutputTool));
    registry.register(Box::new(background::task_stop::TaskStopTool));
    registry.register(Box::new(diff::DiffTool));
    registry.register(Box::new(file_ops::move_file::MoveFileTool));
    registry.register(Box::new(file_ops::delete::DeleteTool));
    registry.register(Box::new(file_ops::copy::CopyFileTool));
}
