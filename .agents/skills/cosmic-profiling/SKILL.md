---
name: cosmic-profiling
description: Profile performance, memory heap usage, and execution hotspots of native COSMIC/Wayland applications under automated load.
---

# COSMIC Application Profiling

This skill profiles CPU execution hotspots and memory usage during deterministic UI interactions.

## Prerequisites
Ensure target profiling tools are installed:
* **`samply`**: CPU execution profiler (runs on cargo-installed binary).
* **`heaptrack`**: Memory heap allocation profiler.

## Instructions
1. The script compiles the application using the custom `profiling` profile defined in `Cargo.toml` (which inherits release optimizations, uses memory-friendly settings like thin LTO and split units, and retains line debug symbols).
2. Run the profiling script located at `.agents/skills/cosmic-profiling/scripts/profile_run.sh <tool>` (where `<tool>` is `samply` or `heaptrack`).
3. Analyze output:
   - For `samply`: Load the generated `profile.json.gz` file into [profiler.firefox.com](https://profiler.firefox.com/).
   - For `heaptrack`: Run `heaptrack_gui heaptrack.<binary>.<pid>.gz` to inspect heap allocation size and leaks.
