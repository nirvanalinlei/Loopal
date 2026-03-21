# process/ — Process Execution Tools

Tools that spawn and manage external processes. These interact with the operating
system's process model rather than the file system directly.

| Crate | Tool(s) | Description |
|-------|---------|-------------|
| `bash` | Bash | Execute shell commands with timeout, output capture, background support |
| `background` | TaskOutput, TaskStop | Manage long-running background tasks: poll output, stop processes |

## Design notes

- **Bash** is the only `Dangerous`-permission tool — it can execute arbitrary commands
- Background tasks are tracked in a global in-memory store (`LazyLock<HashMap>`)
- Output is merged (stdout + stderr) and truncated to 2000 lines / 512 KB
- Background mode returns a `task_id` for later polling via TaskOutput
