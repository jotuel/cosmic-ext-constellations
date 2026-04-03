Agent: coder
Phase: 3/3
Batch: single
Session: claw-foundation-phase1

# Agent Base Protocol ... (injected)
# Filesystem Safety Protocol ... (injected)

TOOL RESTRICTIONS (MANDATORY):
You are authorized to use ONLY the following tools: [read_file, write_file, replace, run_shell_command, list_directory, glob, grep_search].
Do NOT use any tools not listed above.

FILE WRITING RULES (MANDATORY):
Use ONLY `write_file` to create files and `replace` to modify files.
Do NOT use `run_shell_command` with cat, echo, printf, heredocs, or shell redirection (>, >>) to write file content.

Task: Implement Matrix Sync & Shell Integration.

Progress: Phase 3 of 3: Matrix Sync & Shell Integration
Session: claw-foundation-phase1

Files to modify:
- /home/jwnz/Git/cosmic-ext-claw/src/main.rs: Add Matrix sync logic to `update()` and bridge events via `iced::Subscription`.
- /home/jwnz/Git/cosmic-ext-claw/src/matrix/mod.rs: Add `sync()` method and event stream.

Deliverables:
- Background Matrix sync task.
- Integration of sync status into the libcosmic UI.
- `iced::Subscription` for Matrix events.

Validation: cargo build

Context:
Phases 1 & 2 established the shell and Matrix engine. Now bridge them. Use libcosmic's `Task` and `Subscription` to handle Matrix events.

Do NOT:
- Implement Room List or Timeline (Phase 2 & 3 of SPEC)
- Change the core architecture.
