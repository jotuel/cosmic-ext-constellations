Agent: coder
Phase: 1/3
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

Task: Initialize the Rust project and set up the libcosmic application shell.

Progress: Phase 1 of 3: Project Initialization & Foundation
Session: claw-foundation-phase1

Files to create:
- /home/jwnz/Git/cosmic-ext-claw/Cargo.toml: Configure dependencies (libcosmic, matrix-sdk-ui, matrix-sdk-sqlite, tokio, oo7)
- /home/jwnz/Git/cosmic-ext-claw/src/main.rs: Basic libcosmic application shell

Deliverables:
- Initialized Rust project
- Configured Cargo.toml
- Functional (but empty) libcosmic application shell

Validation: cargo check

Context:
This is the first phase of building 'claw', a COSMIC Matrix client. Follow the SPEC.md and .agents/skills/COSMIC-DEVELOPMENT.md for design patterns.

Do NOT:
- Implement Matrix logic yet (Phase 2)
- Add UI widgets beyond the basic shell
- Modify any files outside the project root
