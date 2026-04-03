Agent: coder
Phase: 2/3
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

Task: Implement Matrix Engine & Auth Logic.

Progress: Phase 2 of 3: Matrix Engine & Auth Logic
Session: claw-foundation-phase1

Files to create:
- /home/jwnz/Git/cosmic-ext-claw/src/matrix/mod.rs: Matrix engine with Client + SQLite persistence and basic Login logic.

Deliverables:
- Matrix SDK Client initialization with SQLite store.
- Implementation of a login function (Password-based).
- Integration with oo7 for token storage.

Validation: cargo test

Context:
Phase 1 established the project and libcosmic shell. Now implement the Matrix Logic layer in `src/matrix/mod.rs`. Use `matrix-sdk`, `matrix-sdk-ui`, and `oo7`.

Do NOT:
- Implement the UI for login yet (Phase 3 integration)
- Implement Room List or Timeline (Phases 2 & 3 of the SPEC)
