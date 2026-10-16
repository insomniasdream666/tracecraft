# AGENTS.md

## Project overview

TraceCraft is a local-first PCB design app built with a React + TypeScript frontend and a Rust + Tauri backend. The main UX is split across schematic editing, PCB layout, 3D preview, library tooling, and AI-assisted workflows. The project is intentionally desktop-focused and emphasizes local data ownership, editor responsiveness, and model-driven design logic.

## Key references

- Product and architecture overview: [docs/SPECIFICATION.md](docs/SPECIFICATION.md)
- Implementation patterns and project structure: [docs/IMPLEMENTATION.md](docs/IMPLEMENTATION.md)
- Frontend app entry: [src/App.tsx](src/App.tsx)
- UI state store example: [src/stores/uiStore.ts](src/stores/uiStore.ts)

Use these docs as the primary design source; do not duplicate large sections of them in code comments or new docs.

## Working conventions

### Frontend

- Use React + TypeScript for all UI work.
- Keep the app shell and editor views in [src/components](src/components).
- Prefer Zustand stores for shared UI and app state; follow existing store patterns in [src/stores](src/stores).
- Keep domain types in [src/types](src/types) and reusable geometry/constants in [src/lib](src/lib).
- Favor small, composable components over large monolithic screens.

### Backend

- Place Rust logic under [src-tauri/src](src-tauri/src).
- Keep domain modeling, commands, and engine logic separated by concern (for example: model, commands, engine, io, db, ai, undo).
- Use Tauri commands for cross-boundary operations; avoid mixing UI concerns into Rust modules.
- The project is built around command/undo patterns and arena-backed data structures, as described in [docs/IMPLEMENTATION.md](docs/IMPLEMENTATION.md).

### Testing and validation

Run the smallest relevant validation for the change:

- Frontend/unit tests: `npm test`
- E2E tests: `npm run test:e2e`
- Production build: `npm run build`
- Tauri desktop app: `npm run tauri`

Prefer targeted checks over broad suites when iterating. If a change touches Tauri IPC or state behavior, validate the relevant frontend and backend path together.

## Development guidance

- Match the existing patterns in nearby files instead of introducing new abstractions unless they clearly solve the same problem.
- Keep updates scoped to the specific subsystem being changed.
- Avoid editing generated or lockfile-only outputs unless the task requires it.
- Preserve local-first behavior and desktop-app assumptions; do not introduce cloud-only patterns or hidden network dependencies.
- For AI-related work, follow the project’s local inference and assistant patterns rather than assuming remote APIs.

## Suggested workflow for agents

1. Start by reading the relevant files in the feature area and the design docs above.
2. Implement the fix or feature in the smallest possible scope.
3. Validate with the narrowest relevant command.
4. If a change introduces new project conventions, update the documentation that already exists rather than creating duplicate guidance.

## Useful repo facts

- Build toolchain: Vite + TypeScript + Vitest + Playwright
- Desktop shell: Tauri 2.x with Rust backend
- UI rendering: React + Konva + Three.js
- State management: Zustand
- Primary app domains: schematic editor, PCB layout editor, preview, library, AI panel
