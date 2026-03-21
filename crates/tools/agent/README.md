# agent/ — Agent Internal State Tools

Tools that interact with Loopal's own data structures and control flow rather than
external resources. These are intercepted by the agent loop runner — they never
perform real I/O.

| Crate | Tool(s) | Description |
|-------|---------|-------------|
| `ask-user` | AskUser | Present structured questions to the user via TUI dialog |
| `plan-mode` | EnterPlanMode, ExitPlanMode | Switch agent between plan mode (read-only) and act mode |

## Design notes

- These tools are **intercepted** by the agent loop, not executed via the normal
  tool pipeline. The `execute()` method is never called at runtime.
- They exist as `Tool` trait implementations so the LLM sees them in the tool
  definitions and can invoke them through the standard tool-call mechanism.
- `AskUser` triggers a `UserQuestionRequest` event routed to the TUI/frontend.
- Plan mode restricts the agent to `ReadOnly`-permission tools only.
