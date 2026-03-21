// Single test binary — includes all test modules
#[path = "suite/auto_compact_test.rs"]
mod auto_compact_test;
#[path = "suite/compaction_test.rs"]
mod compaction_test;
#[path = "suite/context_guard_edge_test.rs"]
mod context_guard_edge_test;
#[path = "suite/context_guard_test.rs"]
mod context_guard_test;
#[path = "suite/message_size_guard_test.rs"]
mod message_size_guard_test;
#[path = "suite/pipeline_test.rs"]
mod pipeline_test;
#[path = "suite/price_limit_test.rs"]
mod price_limit_test;
#[path = "suite/smart_compact_test.rs"]
mod smart_compact_test;
#[path = "suite/system_prompt_test.rs"]
mod system_prompt_test;
#[path = "suite/token_counter_test.rs"]
mod token_counter_test;
