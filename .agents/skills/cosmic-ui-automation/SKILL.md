---
name: cosmic-ui-automation
description: Automate, navigate, and capture visual screenshots of native COSMIC/Wayland application layouts during UI/UX inspections.
---

# COSMIC UI/UX Automation

This skill helps you verify and inspect the layout of native COSMIC/Wayland applications using automated keyboard simulation and screen capture.

## Prerequisites
Ensure the system has:
* **`wtype`**: Virtual keyboard simulator for Wayland.
* **`cosmic-screenshot`**: Native screenshot utility.

## Instructions
1. Run the automated navigation and capture script located at `scripts/test_ui.sh`.
2. Inspect the saved frames under `scratch/screens/` for visual defects:
   - Text node overflow/truncation.
   - Clipped/hidden buttons (such as the Join button next to long room names).
   - Missing horizontal/vertical pane separators.
   - Selection indicator highlights on focused list elements.
