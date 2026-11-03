# AGENTS.md

Working rules for coding agents. Keep them short and literal.

## Product

Windows-only Tauri 2 + Preact + Vite desktop app. It indexes a local wojak image library in place under `C:\Users\William\Wojak-World` (Main Wojak + Wojakland). The app never copies, moves, or owns the image binaries.

## Metadata

SQLite lives in Tauri app-local data. It stores paths, favorites, tags, derived filename tokens and categories, and thumbnail metadata. The thumbnail image cache also lives in app-local data only. Never write thumbnails into Wojak-World.

## Search

Split descriptive filenames into tokens and phrases for search and suggested categories. Manual tags refine that. The first mature cut has no separate taxonomy editor.

## Planning

Always build toward the mature end state. Small requested edits stay small. A large update is one unfinished arc until the feature is whole: no half-feature ships, and no bridge phases that force a rewrite.

For a new task: investigate, discuss findings with the owner, write an implementation plan, and begin coding only when the owner confirms the plan. Every implementation plan lists existing files expected to change and files expected to be created, or explicitly states none.

After the owner confirms a plan, land that confirmed change on main directly. No pull request unless the owner asks for one or explicitly wants review isolation.

## Verification

The owner verifies all UI/UX. Do not launch the app, smoke-test visuals, or browser-automate unless explicitly asked.

## Command failure

If an expected command fails, diagnose why. Do not jump to workarounds. Record the broken command and what actually works in `docs/Agent-Workflow-Notes.md` so later sessions do not repeat the dead path.

## Docs policy

- Change `AGENTS.md` only when the owner asks.
- Update `docs/architecture.md` when architecture or file scope meaningfully changes.
- Update `docs/UPDATE-LOG.md` only when the owner asks.
- Update `docs/Agent-Workflow-Notes.md` when command truth is learned.

## Hard rules

- Windows-only. No Apple or Android accommodations.
- Prefer established libraries and ordinary platform and filesystem behavior over bespoke architecture.
- Keep change cost proportional to the trigger.
- Inspect mounted code before editing. Do not guess.
- No tests unless asked.
- Do not commit or push until the owner confirms the plan. Once confirmed, push and land that confirmed change on main. No history rewrite unless asked. No unsolicited commits.
- Ask before destructive cleanup, broad moves, deletions, or schema or user-data impact.
- Spoken owner instruction beats older docs when they conflict.
- Prefer small targeted edits. Do not invent process etiquette docs.

Out of scope for this file: generic style guides and long checklists of obvious defaults.
