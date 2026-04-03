# Implementation Plan: claw - Phase 4: The Markdown Composer

## 1. Objective
Implement a robust Markdown composer for 'claw' with multi-line editing support, live preview, and formatted message sending.

## 2. Execution Strategy

| Phase | ID | Agent | Objective | Parallel | Blocked By |
|---|---|---|---|---|---|
| Foundation | 1 | `coder` | Add dependencies and refactor UI for SegmentedButton. | No | - |
| Implementation | 2 | `coder` | Integrate `cosmic-edit` and implement the `PreviewArea`. | No | 1 |

## 3. Phase Details

### Phase 1: Foundation
- **Objective**: Prepare the project for the new composer components.
- **Agent**: `coder`
- **Files to Modify**:
    - `Cargo.toml`: Add `cosmic-edit` dependency.
    - `src/main.rs`: 
        - Update imports for `segmented_button`.
        - Refactor `view()` to use a `segmented_button` for the Edit/Preview toggle.
- **Validation**: `cargo check`

### Phase 2: Implementation
- **Objective**: Implement the multi-line editor and rich preview area.
- **Agent**: `coder`
- **Files to Modify**:
    - `src/main.rs`:
        - Replace `text_input` with `cosmic_edit::Editor` (or the equivalent multi-line widget).
        - Implement `view_preview()` helper that uses `pulldown-cmark` to render a `Column` of styled text widgets (headings, bold, italic, code).
        - Ensure state reset (`composer_text.clear()`) after successful `SendMessage`.
- **Validation**: `cargo build`

## 4. Cost Estimation

| Phase | Agent | Model | Est. Input | Est. Output | Est. Cost |
|-------|-------|-------|-----------|------------|----------|
| 1 | coder | Flash | 2000 | 500 | $0.004 |
| 2 | coder | Flash | 3000 | 1200 | $0.008 |
| **Total** | | | **5000** | **1700** | **$0.012** |

Approve this implementation plan to begin execution?

1. Approve plan
2. Revise plan
3. Abort execution
