# TraceCraft — Implementation Guide

## Design Patterns & Rationale

### 1. Arena Allocation — PCB Data Model

**Pattern:** Store all schematic/PCB elements in `Vec`s indexed by typed IDs.

**Why:** PCB designs are graph-like (components → pins → nets → traces). Rust's borrow
checker fights graph structures. Arena allocation gives us:
- Cheap `Copy` IDs instead of `Rc<RefCell<>>`
- Cache-friendly iteration for rendering
- Simple undo (snapshot the arena state)
- No lifetime complexity across IPC boundary

```rust
// src-tauri/src/model/pcb.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ComponentId(pub usize);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NetId(pub usize);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TraceId(pub usize);

pub struct PcbBoard {
    pub components: Arena<Component>,
    pub nets: Arena<Net>,
    pub traces: Arena<Trace>,
    pub vias: Arena<Via>,
    pub zones: Arena<Zone>,
    pub outline: Option<BoardOutline>,
    pub design_rules: DesignRules,
}

pub struct Arena<T> {
    items: Vec<Option<T>>,
    free_list: Vec<usize>,
}
```

### 2. Command Pattern — Undo/Redo

**Pattern:** Each user action is a `Command` trait object that knows how to `execute`
and `undo`.

**Why:** PCB editors are the quintessential undo-heavy application. A single user
action (place component, route trace) must be reversible. Commands also enable
macros and batch operations.

```rust
// src-tauri/src/undo/command.rs
pub trait Command {
    fn execute(&mut self, board: &mut PcbBoard) -> Result<(), Error>;
    fn undo(&mut self, board: &mut PcbBoard) -> Result<(), Error>;
    fn name(&self) -> &str;
}

pub struct CommandHistory {
    executed: Vec<Box<dyn Command>>,
    undone: Vec<Box<dyn Command>>,
    max_history: usize,
}

impl CommandHistory {
    pub fn execute(&mut self, cmd: impl Command, board: &mut PcbBoard) -> Result<(), Error> {
        cmd.execute(board)?;
        self.executed.push(Box::new(cmd));
        self.undone.clear(); // new action invalidates redo
        Ok(())
    }
    pub fn undo(&mut self, board: &mut PcbBoard) -> Result<(), Error> { /* ... */ }
    pub fn redo(&mut self, board: &mut PcbBoard) -> Result<(), Error> { /* ... */ }
}
```

### 3. CQRS via Tauri Commands

**Pattern:** Separate Rust functions into *queries* (read state, return data) and
*commands* (mutate state, return confirmation).

**Why:** Natural fit for Tauri IPC. Queries are safe to call frequently (every frame
for rendering). Commands are gated through the undo system. Clear separation prevents
accidental mutations.

```rust
// Query — called every frame for rendering
#[tauri::command]
fn get_board_state(state: State<AppState>) -> Result<PcbBoardSnapshot, Error> {
    Ok(state.board.snapshot())
}

// Command — user action, goes through undo
#[tauri::command]
fn place_component(
    state: State<AppState>,
    component_id: ComponentId,
    position: Point,
    rotation: f64,
) -> Result<(), Error> {
    let cmd = PlaceComponent { component_id, position, rotation };
    state.command_history.execute(cmd, &mut state.board)
}
```

### 4. Event Bus — Schematic ↔ PCB Sync

**Pattern:** The Rust backend emits Tauri events when the model changes. Frontend
subscribers react to specific events.

**Why:** Schematic and PCB editors share a common netlist. When the user connects a
wire in the schematic, the PCB layout must update its ratsnest and highlight the
net. Events decouple the editors completely.

```rust
// Backend — emit on model change
fn on_net_modified(app_handle: &AppHandle, net_id: NetId) {
    app_handle.emit("net-modified", NetModifiedPayload { net_id }).unwrap();
}

// Frontend — subscribe
useEffect(() => {
    const unlisten = listen<NetModifiedPayload>("net-modified", (event) => {
        pcbStore.getState().highlightNet(event.payload.netId);
    });
    return () => unlisten.then(fn => fn());
}, []);
```

### 5. Repository Pattern — Component Library

**Pattern:** Wrap all SQLite queries behind a `ComponentRepository` trait with a
`SqliteComponentRepository` implementation.

**Why:** Testability (swap in a mock repo) and future-proofing (could swap to a
different DB later). The rest of the codebase never touches SQL directly.

```rust
#[async_trait]
pub trait ComponentRepository: Send + Sync {
    async fn search(&self, query: &SearchQuery) -> Result<Vec<ComponentSummary>>;
    async fn get_by_id(&self, id: Uuid) -> Result<Option<Component>>;
    async fn save(&self, component: &Component) -> Result<()>;
    async fn delete(&self, id: Uuid) -> Result<()>;
    async fn get_categories(&self) -> Result<Vec<Category>>;
}
```

### 6. Strategy Pattern — Auto-Router

**Pattern:** Multiple routing algorithms implement a common `Router` trait.

**Why:** Different routing problems need different strategies: A* for point-to-point,
Lee-Moore for maze routing, fan-out for dense packages. Users can pick the strategy
that works best.

```rust
pub trait Router {
    fn route(&self, board: &PcbBoard, nets: &[NetId]) -> Result<Vec<Trace>, RoutingError>;
    fn name(&self) -> &str;
}

pub struct AStarRouter {
    pub grid_resolution: f64,
    pub via_cost: f64,
    pub preferred_directions: HashMap<Layer, Direction>,
}

pub struct FanOutRouter {
    pub pattern: FanOutPattern,
}

pub struct BatchRouter {
    routers: Vec<Box<dyn Router>>,
}
```

---

## Directory Structure

```
tracecraft/
├── src-tauri/
│   ├── src/
│   │   ├── main.rs                  # Tauri bootstrap, command registration, app state init
│   │   ├── commands/                # Tauri #[tauri::command] handlers
│   │   │   ├── mod.rs
│   │   │   ├── project.rs           # create/open/close/save project
│   │   │   ├── schematic.rs         # place symbol, draw wire, annotate
│   │   │   ├── pcb.rs               # place footprint, route trace, draw outline
│   │   │   ├── library.rs           # search, import, create component
│   │   │   ├── routing.rs           # auto-route, fan-out, batch route
│   │   │   ├── drc.rs               # run DRC, get violations
│   │   │   ├── ai.rs                # recommend component, suggest route, chat
│   │   │   └── viewport.rs          # get viewport state, zoom to fit
│   │   ├── model/                   # Core data structures (the "domain")
│   │   │   ├── mod.rs
│   │   │   ├── project.rs           # Project metadata struct
│   │   │   ├── schematic.rs         # SchematicSheet, Symbol, Wire, Junction, NetLabel
│   │   │   ├── pcb.rs               # PcbBoard, FootprintInstance, Trace, Via, Zone, Layer
│   │   │   ├── library.rs           # Component (symbol + footprint + 3D), Category
│   │   │   ├── netlist.rs           # Net (name, connected pins), Netlist
│   │   │   ├── geometry.rs          # Point, Size, Rect, Rotation, LayerId
│   │   │   └── id.rs                # Newtype IDs: ComponentId, NetId, SymbolId, ...
│   │   ├── engine/                  # Algorithms and computation
│   │   │   ├── mod.rs
│   │   │   ├── autorouter.rs        # A* grid router, cost heuristics, layer assignment
│   │   │   ├── fanout.rs            # BGA/QFP fan-out pattern generator
│   │   │   ├── drc.rs               # Clearance, width, annular ring, soldermask checks
│   │   │   ├── erc.rs               # Floating pins, shorted outputs, unconnected nets
│   │   │   ├── netlist.rs           # Extract netlist from schematic, diff, sync
│   │   │   └── ratsnest.rs          # Generate ratsnest lines for connected-but-unrouted nets
│   │   ├── io/                      # File format parsers/generators
│   │   │   ├── mod.rs
│   │   │   ├── native.rs            # .tcproj: SQLite-backed serialization format
│   │   │   ├── kicad_sch.rs         # KiCad 7+ schematic (.kicad_sch) S-expression parser
│   │   │   ├── kicad_pcb.rs         # KiCad 7+ PCB (.kicad_pcb) S-expression parser
│   │   │   ├── gerber.rs            # RS-274X Gerber export
│   │   │   ├── ipc2581.rs           # IPC-2581 XML export
│   │   │   └── bom.rs               # BOM CSV/XLSX export
│   │   ├── db/                      # Database layer (SQLite)
│   │   │   ├── mod.rs
│   │   │   ├── migrations.rs        # Schema versions 1..N, run on app startup
│   │   │   ├── library_repo.rs      # SqliteComponentRepository implementation
│   │   │   └── project_store.rs     # Save/load project state to SQLite
│   │   ├── ai/                      # AI inference
│   │   │   ├── mod.rs
│   │   │   ├── recommender.rs       # ONNX Runtime: embed netlist → rank components
│   │   │   ├── routing_assist.rs    # ONNX Runtime: trace path prediction
│   │   │   └── chat.rs             # llama.cpp bindings for PCB design Q&A
│   │   ├── undo/                    # Command pattern undo/redo
│   │   │   ├── mod.rs
│   │   │   └── command.rs           # Command trait, CommandHistory, concrete commands
│   │   └── app_state.rs             # Singleton app state: board, history, db, settings
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── capabilities/
│   │   └── default.json             # Tauri 2.x capability permissions
│   └── icons/                       # App icons
│
├── src/                             # React frontend
│   ├── main.tsx                     # Entry point, React root mount
│   ├── App.tsx                      # Router/layout: decides which editor is active
│   ├── vite-env.d.ts                # Vite + Tauri type declarations
│   ├── components/
│   │   ├── layout/
│   │   │   ├── AppShell.tsx         # Main chrome: menubar + toolbar + canvas + sidebar
│   │   │   ├── Toolbar.tsx          # Top toolbar: tool selection, undo/redo, zoom
│   │   │   ├── Sidebar.tsx          # Right sidebar: properties, layers, components
│   │   │   └── StatusBar.tsx        # Bottom bar: coordinates, tool hints, DRC count
│   │   ├── schematic/
│   │   │   ├── SchematicCanvas.tsx  # Konva Stage + Layer for schematic editor
│   │   │   ├── SymbolRenderer.tsx   # Renders a schematic symbol on canvas
│   │   │   ├── WireTool.tsx         # Interactive wire drawing tool
│   │   │   ├── SelectionOverlay.tsx # Rubber-band selection on canvas
│   │   │   └── SchematicToolbar.tsx # Tool-specific options for schematic mode
│   │   ├── pcb/
│   │   │   ├── PcbCanvas.tsx        # Konva Stage + Layer for PCB layout editor
│   │   │   ├── FootprintRenderer.tsx# Renders a footprint (pads, outline, refdes)
│   │   │   ├── TraceTool.tsx        # Interactive trace routing tool
│   │   │   ├── RatsnestRenderer.tsx # Draws ratsnest lines between connected pads
│   │   │   ├── PcbToolbar.tsx       # Tool-specific options for PCB mode
│   │   │   └── LayerPanel.tsx       # Layer visibility, color, stack-up control
│   │   ├── preview/
│   │   │   ├── Preview3D.tsx        # Three.js canvas for 3D board preview
│   │   │   └── ModelLoader.tsx      # Loads .step/.wrl → Three.js mesh
│   │   ├── library/
│   │   │   ├── LibraryBrowser.tsx   # Searchable component grid/list
│   │   │   ├── ComponentDetail.tsx  # Detailed view of a component (symbol + footprint)
│   │   │   ├── SymbolEditor.tsx     # Canvas-based symbol drawing tool
│   │   │   └── FootprintEditor.tsx  # Canvas-based footprint land pattern editor
│   │   ├── ai/
│   │   │   ├── AIPanel.tsx          # Sidebar panel for AI suggestions
│   │   │   ├── RecommenderWidget.tsx# Shows component recommendations inline
│   │   │   ├── RouteSuggestion.tsx  # Preview of suggested trace route
│   │   │   └── ChatWindow.tsx       # LLM chat interface
│   │   └── common/
│   │       ├── KonvaCanvas.tsx      # Shared Konva Stage wrapper with zoom/pan
│   │       ├── PropertyPanel.tsx    # Generic property editor grid
│   │       ├── Dialog.tsx           # Modal dialog component
│   │       └── ToolButton.tsx       # Toolbar button with active state
│   ├── stores/                      # Zustand stores
│   │   ├── projectStore.ts          # Current project metadata, file path
│   │   ├── schematicStore.ts        # Schematic editing state (elements, selection, tool)
│   │   ├── pcbStore.ts              # PCB editing state (board, layers, selection, tool)
│   │   ├── libraryStore.ts          # Search results, selected component
│   │   ├── aiStore.ts               # AI panel state, suggestions, chat history
│   │   └── uiStore.ts              # Active tool, sidebar open, theme, zoom level
│   ├── hooks/
│   │   ├── useTauriCommand.ts       # Typed wrapper around invoke() with loading/error
│   │   ├── useCanvasInteraction.ts  # Unified mouse/kbd handler for Konva canvases
│   │   └── useTauriEvent.ts         # Typed wrapper around listen() with cleanup
│   ├── types/
│   │   ├── schematic.ts             # TS mirror of Rust schematic types
│   │   ├── pcb.ts                   # TS mirror of Rust PCB types
│   │   ├── library.ts               # TS mirror of Rust library types
│   │   └── project.ts               # TS mirror of Rust project types
│   ├── lib/
│   │   ├── geometry.ts              # Point, Rect math utilities
│   │   ├── colors.ts                # Layer colors, DRC heatmap, theme
│   │   └── constants.ts             # Grid sizes, min/max values, version
│   └── styles/
│       ├── index.css                # Tailwind directives + custom layer styles
│       └── preview.css              # Three.js container styles
│
├── package.json
├── tsconfig.json
├── tsconfig.node.json
├── vite.config.ts
├── tailwind.config.js
├── postcss.config.js
├── index.html
├── .gitignore
└── README.md
```

---

## Dependencies

### Rust (Cargo.toml)

| Crate | Version | Purpose |
|-------|---------|---------|
| `tauri` | 2.x | Desktop shell, IPC, file dialogs |
| `serde` | 1.x (with derive) | Serialization for IPC and file formats |
| `serde_json` | 1.x | JSON handling (IPC, IPC-2581) |
| `rusqlite` | 0.31 | SQLite for component library and project store |
| `uuid` | 1.x | Unique IDs for components, projects |
| `chrono` | 0.4 | Timestamps for project metadata |
| `thiserror` | 1.x | Ergonomic error types |
| `tracing` | 0.1 | Structured logging |
| `tracing-subscriber` | 0.3 | Log output |
| `klv` (or custom) | — | KiCad S-expression parsing |
| `quick-xml` | 0.31 | IPC-2581 XML generation |
| `csv` | 1.3 | BOM CSV export |
| `rust_xlsxwriter` | 0.64 | BOM XLSX export |
| `ort` | 2.x | ONNX Runtime bindings for AI inference |
| `llama-cpp-2` | 0.1 | llama.cpp bindings for local LLM |
| `image` | 0.25 | Thumbnail generation for library |
| `parking_lot` | 0.12 | Faster Mutex/RwLock for app state |

### Frontend (package.json)

| Package | Purpose |
|---------|---------|
| `react` + `react-dom` | UI framework |
| `typescript` | Type safety |
| `vite` | Bundler + dev server |
| `@tauri-apps/api` | Tauri IPC client |
| `@tauri-apps/plugin-dialog` | File dialogs |
| `@tauri-apps/plugin-fs` | File access |
| `konva` + `react-konva` | Canvas-based 2D rendering |
| `three` | 3D preview rendering |
| `@react-three/fiber` | React renderer for Three.js |
| `zustand` | State management |
| `tailwindcss` + `postcss` + `autoprefixer` | Styling |
| `lucide-react` | Icon set |
| `react-hot-toast` | Notifications |
| `zod` | Runtime type validation for IPC payloads |

---

## Module Interfaces & Data Flow

### IPC Command Catalog

All IPC commands follow the pattern:
```
invoke("command_name", { arg1, arg2 }) → Promise<Result>
```

```
── Project Commands ──
create_project(name, template?)       → ProjectSummary
open_project(path)                    → ProjectSummary
save_project()                        → void
close_project()                       → void
get_recent_projects()                 → RecentProject[]

── Schematic Commands ──
get_schematic(project_id, sheet_id?)  → SchematicSheet
place_symbol(sheet_id, lib_id, pos)   → SymbolId
draw_wire(sheet_id, from, to)         → WireId
delete_element(sheet_id, element_id)  → void
annotate_schematic(sheet_id)          → void
run_erc(sheet_id)                     → ERCResult[]

── PCB Commands ──
get_board_state(project_id)           → PcbBoard
place_footprint(project_id, fp_id, pos, rot) → FootprintId
route_trace(project_id, net_id, points) → TraceId
add_via(project_id, pos, layers)      → ViaId
draw_polygon(project_id, layer, points) → ZoneId
set_board_outline(project_id, points) → void
run_drc(project_id)                   → DRCViolation[]
synchronize_netlist(project_id)       → NetlistDiff

── Auto-Routing Commands ──
autoroute_net(project_id, net_id, strategy?) → RouteResult
autoroute_all(project_id, strategy?)  → RouteResult[]
fanout_component(project_id, fp_id)   → RouteResult

── Library Commands ──
search_components(query)              → ComponentSummary[]
get_component_detail(id)              → Component
create_component(component)           → ComponentId
import_kicad_library(path)            → ImportResult
export_component(id, format)          → void

── AI Commands ──
recommend_components(netlist_context) → AIRecommendation[]
suggest_route(board_state, cursor_pos, net_id) → RouteSuggestion[]
ai_chat(message, context)             → AsyncIterator<ChatChunk> (via events)

── DRC Commands ──
run_drc(project_id)                   → DRCViolation[]
run_drc_region(project_id, bbox)      → DRCViolation[]
get_drc_settings(project_id)          → DRCSettings
update_drc_settings(project_id, settings) → void
```

### Data Flow: User Places a Footprint

```
User drags component from library to PCB canvas
    │
    ▼
React: PCBStore sets drag ghost position
    │
    ▼ (on drop)
React: invoke("place_footprint", { projectId, fpId, pos, rot })
    │
    ▼
Rust: Tauri command handler receives request
    │
    ▼
Rust: Creates PlaceFootprint command struct
    │
    ▼
Rust: CommandHistory.execute(PlaceFootprint)
    ├─ allocates FootprintId
    ├─ inserts into PcbBoard.footprints arena
    ├─ updates netlist (adds net connections)
    └─ pushes onto undo stack
    │
    ▼
Rust: Emits "footprint-placed" event with FootprintId
    │
    ▼
React: PCBStore listens, updates local state
React: RatsnestRenderer re-renders affected nets
React: FootprintRenderer renders new footprint on canvas
```

### Data Flow: Auto-Route a Net

```
User clicks "Auto-Route" on a selected net
    │
    ▼
React: invoke("autoroute_net", { projectId, netId, strategy: "astar" })
    │
    ▼
Rust: autorouter.rs builds grid from board state
    │
    ▼
Rust: A* search with cost heuristics:
    ├─ prefers same-layer straight traces
    ├─ penalizes vias (layer changes)
    ├─ respects user preferred direction per layer
    └─ avoids existing traces with clearance margin
    │
    ▼
Rust: If route found:
    ├─ Creates Trace commands for each segment
    ├─ Executes through CommandHistory (undoable)
    └─ Emits "traces-added" event
    │
    ▼
Rust: If no route found:
    └─ Returns RouteResult::Failure { reason: "..." }
    │
    ▼
React: Updates traces on canvas or shows error toast
```

---

## Critical Code Patterns

### App State (Rust)

```rust
// src-tauri/src/app_state.rs
pub struct AppState {
    pub project: Option<Project>,
    pub board: PcbBoard,
    pub schematics: HashMap<SheetId, SchematicSheet>,
    pub command_history: CommandHistory,
    pub library: Arc<dyn ComponentRepository>,
    pub settings: Settings,
    pub ai_engine: Option<AIEngine>,
}

impl AppState {
    pub fn new(library: Arc<dyn ComponentRepository>) -> Self {
        Self {
            project: None,
            board: PcbBoard::default(),
            schematics: HashMap::new(),
            command_history: CommandHistory::new(1000),
            library,
            settings: Settings::default(),
            ai_engine: None,
        }
    }
}
```

### Tauri Events (Rust → React)

```rust
// Emit pattern — used throughout engine/commands
fn emit_event(app_handle: &AppHandle, event: &str, payload: impl Serialize) {
    if let Err(e) = app_handle.emit(event, payload) {
        tracing::warn!("Failed to emit event {}: {}", event, e);
    }
}

// Standard event payloads
#[derive(Serialize, Clone)]
pub struct ElementModified {
    pub element_type: &'static str,
    pub element_id: usize,
    pub change_type: ChangeType, // Created | Modified | Deleted
}
```

### Frontend IPC Hook

```typescript
// src/hooks/useTauriCommand.ts
import { invoke } from "@tauri-apps/api/core";
import { useCallback, useState } from "react";

export function useTauriCommand<TResult, TArgs = void>() {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const execute = useCallback(
    async (command: string, args?: TArgs): Promise<TResult | null> => {
      setLoading(true);
      setError(null);
      try {
        const result = await invoke<TResult>(command, args as any);
        return result;
      } catch (e) {
        setError(String(e));
        return null;
      } finally {
        setLoading(false);
      }
    },
    []
  );

  return { execute, loading, error };
}
```

### Konva Canvas with Zoom/Pan (Shared)

```typescript
// src/components/common/KonvaCanvas.tsx
// Wraps Konva Stage with:
// - Mouse wheel zoom (centered on cursor)
// - Middle-click / spacebar-drag pan
// - Grid snap rendering
// - Coordinate transform helpers (screen ↔ world)

// Stage transform via Konva's built-in:
// stage.scale({ x, y })  — zoom
// stage.position({ x, y }) — pan
// Applied per-frame from uiStore zoom/offset
```

### SQLite Migrations

```rust
// src-tauri/src/db/migrations.rs
const MIGRATIONS: &[&str] = &[
    // v1: Initial schema
    "CREATE TABLE IF NOT EXISTS components (
        id TEXT PRIMARY KEY,
        name TEXT NOT NULL,
        description TEXT,
        manufacturer TEXT,
        category TEXT,
        symbol BLOB,
        footprint BLOB,
        model_3d BLOB,
        metadata TEXT,
        created_at TEXT NOT NULL,
        updated_at TEXT NOT NULL
    );
    CREATE TABLE IF NOT EXISTS categories (
        id TEXT PRIMARY KEY,
        name TEXT NOT NULL UNIQUE,
        parent_id TEXT REFERENCES categories(id)
    );
    CREATE VIRTUAL TABLE IF NOT EXISTS components_fts USING fts5(
        name, description, manufacturer, category
    );",
    // v2: Future migration here
];

pub fn run_migrations(conn: &Connection) -> Result<(), Error> {
    let version: i32 = conn.pragma_query_value(None, "user_version", |r| r.get(0))?;
    for (i, migration) in MIGRATIONS.iter().enumerate() {
        if version <= i as i32 {
            conn.execute_batch(migration)?;
            conn.pragma_update(None, "user_version", (i + 1) as i32)?;
        }
    }
    Ok(())
}
```

---

## Development Plan

### Phase 1: Foundation (Weeks 1-2)
- Scaffold Tauri + React + Vite + Tailwind project
- Implement AppState, CommandHistory, basic Arena model
- SQLite setup with migrations, ComponentRepository trait
- Empty SchematicCanvas and PcbCanvas with zoom/pan
- "Hello world" IPC round-trip

### Phase 2: Component Library (Weeks 3-4)
- LibraryBrowser UI with search
- Symbol editor (canvas-based)
- Footprint editor (canvas-based)
- KiCad library import
- Search with FTS5

### Phase 3: Schematic Editor (Weeks 5-8)
- Place/move/delete symbols
- Wire drawing with snap-to-pin
- Net labels and netlist extraction
- ERC engine
- Hierarchical sheets
- KiCad .kicad_sch import/export

### Phase 4: PCB Layout Editor (Weeks 9-12)
- Footprint placement with ratsnest
- Manual trace routing with push/shove
- Via insertion
- Layer stack-up management
- Copper pours
- Board outline editor
- DRC engine
- Netlist sync with schematic

### Phase 5: Auto-Routing (Weeks 13-14)
- A* grid router
- Multi-layer support with via insertion
- Fan-out router
- Batch routing
- User-configurable routing strategies

### Phase 6: 3D Preview (Week 15)
- Three.js board rendering
- 3D model loading (.step/.wrl → mesh)
- Layer visibility control
- Measurement tools

### Phase 7: AI Copilot (Weeks 16-18)
- ONNX Runtime integration
- Component recommendation model
- Routing suggestion model
- llama.cpp chat integration
- AI panel UI

### Phase 8: File I/O & Polish (Weeks 19-20)
- Gerber export
- IPC-2581 export
- BOM export
- Native .tcproj format
- Error handling polish
- Performance optimization
