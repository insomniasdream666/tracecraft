# TraceCraft — Task Breakdown

> **Total Tasks:** 83 (Tasks 77–83 were added after the original plan)
> **Phases:** 9
> **Estimated Duration:** 20 weeks (solo, original estimate)
>
> **Last verified:** 2026-10-28 against the packaged v0.2.0 `.deb` (WebDriver) and
> AppImage (launch check): schematic place/wire/annotate/ERC, ngspice transient run,
> project save/close/reopen, PCB netlist sync/place/route/via/DRC, Gerber/BOM/CPL/SPICE/
> STEP/KiCad exports, and canvas PNG generation.

Each task is self-contained, ordered by strict dependency, and completable by a
coding agent in a single session. Tasks specify exactly which files to create or
modify.

**Legend:** `[C]` = Create file, `[M]` = Modify file, `[R]` = Run command  
**Status:** `[x]` = Complete and smoke-tested, `[~]` = Partial/approximate or not yet
verified end-to-end, `[ ]` = Not implemented

---

## Phase 1: Foundation (Tasks 1–14)

### [x] Task 1: Scaffold Tauri + React + Vite + Tailwind

**Goal:** Working Tauri 2.x app with React frontend and dev hot-reload.

**Files:**
- `[C] tracecraft/package.json` — Dependencies: react, react-dom, typescript, vite,
  @tauri-apps/api, @tauri-apps/cli, konva, react-konva, zustand, tailwindcss,
  postcss, autoprefixer, lucide-react, react-hot-toast, zod, three, @react-three/fiber
- `[C] tracecraft/tsconfig.json` — Strict TS config for React
- `[C] tracecraft/tsconfig.node.json` — Node-env TS config for Vite
- `[C] tracecraft/vite.config.ts` — Vite config with Tauri plugin
- `[C] tracecraft/tailwind.config.js` — Content paths pointing to src/
- `[C] tracecraft/postcss.config.js` — Tailwind + autoprefixer
- `[C] tracecraft/index.html` — Entry HTML with `<div id="root">`
- `[C] tracecraft/src/main.tsx` — React root mount
- `[C] tracecraft/src/App.tsx` — Empty root component
- `[C] tracecraft/src/styles/index.css` — Tailwind directives
- `[C] tracecraft/src/vite-env.d.ts` — Vite client types
- `[C] tracecraft/src-tauri/Cargo.toml` — Tauri + dependencies
- `[C] tracecraft/src-tauri/tauri.conf.json` — Tauri 2.x config (window title "TraceCraft",
  800x600 min size)
- `[C] tracecraft/src-tauri/capabilities/default.json` — Default permissions
- `[C] tracecraft/src-tauri/src/main.rs` — Empty Tauri app with no commands yet
- `[C] tracecraft/.gitignore` — Node, Rust, Tauri ignores

**Verification:** `npm run tauri dev` opens a window with "TraceCraft" title.

---

### [x] Task 2: Rust Data Model — Geometry & ID Types

**Goal:** Core primitive types used across the entire codebase.

**Files:**
- `[C] tracecraft/src-tauri/src/model/id.rs` — Newtype IDs:
  ```rust
  ProjectId(Uuid), SheetId(usize), SymbolId(usize), WireId(usize),
  JunctionId(usize), NetId(usize), FootprintId(usize), TraceId(usize),
  ViaId(usize), ZoneId(usize), ComponentId(Uuid), LayerId(u8)
  ```
  Each derives: `Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize`
- `[C] tracecraft/src-tauri/src/model/geometry.rs` — Point, Size, Rect, Rotation
  with serde + Copy + Clone + math ops (distance, rotate, bounding box)
- `[C] tracecraft/src-tauri/src/model/mod.rs` — Re-exports

**Verification:** `cargo build` compiles. Unit tests for Point math ops.

---

### [x] Task 3: Rust Data Model — Arena Allocator

**Goal:** Generic `Arena<T>` for storing PCB/schematic elements.

**Files:**
- `[C] tracecraft/src-tauri/src/model/arena.rs` — Arena<T>:
  - `insert(item: T) -> usize` — returns index
  - `get(id: usize) -> Option<&T>`
  - `get_mut(id: usize) -> Option<&mut T>`
  - `remove(id: usize) -> Option<T>` — marks slot as free
  - `iter() -> impl Iterator<Item = (usize, &T)>`
  - `len()`, `is_empty()`, `clear()`
  - `snapshot() -> Vec<(usize, T)>` for undo/rendering
- `[M] tracecraft/src-tauri/src/model/mod.rs` — Add arena module

**Verification:** `cargo test` — test insert, remove, re-use freed slots.

---

### [x] Task 4: Rust Data Model — Schematic Elements

**Goal:** Complete schematic data types.

**Files:**
- `[C] tracecraft/src-tauri/src/model/schematic.rs`:
  ```rust
  pub struct SchematicSheet {
      pub id: SheetId,
      pub name: String,
      pub symbols: Arena<SymbolInstance>,
      pub wires: Arena<Wire>,
      pub junctions: Arena<Junction>,
      pub net_labels: Arena<NetLabel>,
      pub buses: Arena<Bus>,
      pub text_annotations: Arena<TextAnnotation>,
      pub properties: HashMap<String, String>,
  }
  pub struct SymbolInstance {
      pub component_id: ComponentId,
      pub position: Point,
      pub rotation: Rotation,
      pub mirrored: bool,
      pub refdes: String,
      pub value: String,
  }
  pub struct Wire {
      pub points: Vec<Point>,
      pub net_id: Option<NetId>,
      pub width: f64,
  }
  pub struct NetLabel { pub name: String, pub position: Point, pub sheet_id: SheetId }
  pub struct Junction { pub position: Point }
  pub struct Bus { pub name: String, pub members: Vec<String>, pub wires: Vec<Wire> }
  pub struct TextAnnotation { pub text: String, pub position: Point, pub size: f64 }
  ```
- `[M] tracecraft/src-tauri/src/model/mod.rs`

**Verification:** `cargo build`. Test constructing a sheet with symbols and wires.

---

### [x] Task 5: Rust Data Model — PCB Elements

**Goal:** Complete PCB board data types.

**Files:**
- `[C] tracecraft/src-tauri/src/model/pcb.rs`:
  ```rust
  pub struct PcbBoard {
      pub footprints: Arena<FootprintInstance>,
      pub traces: Arena<Trace>,
      pub vias: Arena<Via>,
      pub zones: Arena<Zone>,
      pub outline: Option<BoardOutline>,
      pub layers: LayerStack,
      pub design_rules: DesignRules,
  }
  pub struct FootprintInstance {
      pub component_id: ComponentId,
      pub position: Point,
      pub rotation: Rotation,
      pub mirrored: bool,
      pub refdes: String,
      pub locked: bool,
  }
  pub struct Trace {
      pub net_id: NetId,
      pub layer: LayerId,
      pub width: f64,
      pub points: Vec<Point>,
  }
  pub struct Via {
      pub position: Point,
      pub start_layer: LayerId,
      pub end_layer: LayerId,
      pub diameter: f64,
      pub drill: f64,
      pub net_id: Option<NetId>,
  }
  pub struct Zone {
      pub net_id: Option<NetId>,
      pub layer: LayerId,
      pub outline: Vec<Point>,
      pub clearance: f64,
      pub thermal_relief: bool,
  }
  pub struct BoardOutline { pub shape: Vec<Vec<Point>>, pub cutouts: Vec<Vec<Point>> }
  pub struct LayerStack { pub layers: Vec<LayerDef> }
  pub struct LayerDef { pub id: LayerId, pub name: String, pub kind: LayerKind, pub color: String }
  pub enum LayerKind { Top, Bottom, Inner, SolderMask, SilkscreenTop, SilkscreenBottom, Paste }
  pub struct DesignRules { pub min_trace: f64, pub min_clearance: f64, pub min_annular: f64, pub min_hole: f64 }
  ```
- `[C] tracecraft/src-tauri/src/model/library.rs` — Component (symbol + footprint + 3D model + metadata):
  ```rust
  pub struct Component {
      pub id: ComponentId,
      pub name: String,
      pub description: String,
      pub manufacturer: Option<String>,
      pub category: Option<String>,
      pub symbol: SymbolDef,
      pub footprint: FootprintDef,
      pub model_3d: Option<Model3D>,
      pub parameters: HashMap<String, String>,
      pub created_at: String,
      pub updated_at: String,
  }
  pub struct SymbolDef {
      pub pins: Vec<PinDef>,
      pub body: Vec<SymbolGraphics>,
      pub origin: Point,
  }
  pub struct PinDef { pub name: String, pub number: String, pub position: Point, pub length: f64, pub electrical_type: PinType }
  pub enum PinType { Input, Output, Bidirectional, Power, Gnd, Passive, OpenCollector, OpenEmitter, NotConnected }
  pub struct FootprintDef { pub pads: Vec<PadDef>, pub outline: Vec<Point>, pub courtyard: Vec<Point> }
  pub struct PadDef { pub number: String, pub shape: PadShape, pub position: Point, pub size: Size, pub layer: LayerId }
  pub enum PadShape { Round, Rect, Oval, Custom(Vec<Point>) }
  ```
- `[C] tracecraft/src-tauri/src/model/netlist.rs` — Netlist:
  ```rust
  pub struct Netlist { pub nets: Vec<Net> }
  pub struct Net { pub name: String, pub connections: Vec<PinRef> }
  pub struct PinRef { pub refdes: String, pub pin_number: String }
  ```
- `[M] tracecraft/src-tauri/src/model/mod.rs`

**Verification:** `cargo build`. Test creating a PcbBoard with footprints and traces.

---

### [x] Task 6: SQLite Database — Schema & Migrations

**Goal:** Database layer ready for component library and project storage.

**Files:**
- `[C] tracecraft/src-tauri/src/db/mod.rs` — Module setup
- `[C] tracecraft/src-tauri/src/db/migrations.rs`:
  - Run schema version tracking via `PRAGMA user_version`
  - V1: components table, categories table, components_fts FTS5 virtual table
  - V2: (future) projects table
- `[C] tracecraft/src-tauri/src/db/library_repo.rs`:
  ```rust
  pub struct SqliteComponentRepository { conn: Mutex<Connection> }
  impl SqliteComponentRepository {
      pub fn new(path: &str) -> Result<Self>;
  }
  ```
  Implement `ComponentRepository` trait (defined in Task 7).
- `[C] tracecraft/src-tauri/src/db/project_store.rs`:
  ```rust
  pub fn save_project(conn: &Connection, project: &Project) -> Result<()>;
  pub fn load_project(conn: &Connection, id: Uuid) -> Result<Option<Project>>;
  ```
- `[M] tracecraft/src-tauri/Cargo.toml` — Add rusqlite (features: bundled), uuid, chrono

**Verification:** `cargo test` — run migrations, insert component, FTS search finds it.

---

### [x] Task 7: AppState & ComponentRepository Trait

**Goal:** Central app state and library repository interface.

**Files:**
- `[C] tracecraft/src-tauri/src/app_state.rs`:
  ```rust
  pub struct AppState {
      pub project: RwLock<Option<Project>>,
      pub board: RwLock<PcbBoard>,
      pub schematics: RwLock<HashMap<SheetId, SchematicSheet>>,
      pub command_history: RwLock<CommandHistory>,
      pub library: Arc<dyn ComponentRepository>,
      pub settings: RwLock<Settings>,
      pub db: Mutex<Connection>,
      pub app_handle: RwLock<Option<AppHandle>>,
  }
  ```
- `[M] tracecraft/src-tauri/src/model/library.rs` — Add `ComponentRepository` trait:
  ```rust
  #[async_trait]
  pub trait ComponentRepository: Send + Sync {
      async fn search(&self, query: &SearchQuery) -> Result<Vec<ComponentSummary>>;
      async fn get_by_id(&self, id: ComponentId) -> Result<Option<Component>>;
      async fn save(&self, component: &Component) -> Result<()>;
      async fn delete(&self, id: ComponentId) -> Result<()>;
      async fn get_categories(&self) -> Result<Vec<Category>>;
  }
  pub struct SearchQuery { pub text: String, pub category: Option<String>, pub page: usize, pub per_page: usize }
  pub struct ComponentSummary { pub id: ComponentId, pub name: String, pub description: String, pub category: Option<String> }
  ```
- `[M] tracecraft/src-tauri/src/main.rs` — Initialize AppState with SqliteComponentRepository,
  register as Tauri managed state

**Verification:** `cargo build`. App launches with initialized DB.

---

### [~] Task 8: Command Pattern — Undo/Redo System

**Goal:** Generic command history with concrete commands for PCB operations.

**Files:**
- `[C] tracecraft/src-tauri/src/undo/mod.rs`
- `[C] tracecraft/src-tauri/src/undo/command.rs`:
  ```rust
  pub trait Command: Send {
      fn execute(&mut self, board: &mut PcbBoard) -> Result<(), Error>;
      fn undo(&mut self, board: &mut PcbBoard) -> Result<(), Error>;
      fn name(&self) -> &str;
  }
  pub struct CommandHistory { /* Vec<Box<dyn Command>> executed, Vec<Box<dyn Command>> undone, max */ }
  impl CommandHistory {
      pub fn new(max: usize) -> Self;
      pub fn execute(&mut self, cmd: Box<dyn Command>, board: &mut PcbBoard) -> Result<()>;
      pub fn undo(&mut self, board: &mut PcbBoard) -> Result<()>;
      pub fn redo(&mut self, board: &mut PcbBoard) -> Result<()>;
      pub fn can_undo(&self) -> bool;
      pub fn can_redo(&self) -> bool;
  }
  ```
- Concrete commands (in same file or separate):
  ```rust
  pub struct PlaceFootprint { pub component_id: ComponentId, pub position: Point, pub rotation: Rotation, result_id: Option<FootprintId> }
  pub struct RemoveFootprint { pub id: FootprintId, saved: Option<FootprintInstance> }
  pub struct RouteTrace { pub net_id: NetId, pub layer: LayerId, pub width: f64, pub points: Vec<Point>, result_id: Option<TraceId> }
  // ... one struct per mutable operation
  ```

**Verification:** `cargo test` — execute PlaceFootprint, undo, verify not present, redo.

---

### [x] Task 9: Frontend — Layout Shell

**Goal:** App shell with toolbar, sidebar, canvas area, and status bar.

**Files:**
- `[C] tracecraft/src/components/layout/AppShell.tsx` — Main layout grid:
  ```
  ┌──────────────────────────────┐
  │         Toolbar              │
  ├──────────┬───────────────────┤
  │ Sidebar  │   Canvas Area     │
  │          │   (Outlet)        │
  │          │                   │
  ├──────────┴───────────────────┤
  │         StatusBar            │
  └──────────────────────────────┘
  ```
- `[C] tracecraft/src/components/layout/Toolbar.tsx` — Icon buttons for: select, wire,
  component, route, zone, 3D preview, undo, redo, zoom in/out, save. Active tool is
  highlighted.
- `[C] tracecraft/src/components/layout/Sidebar.tsx` — Right sidebar with tabbed panels:
  Properties, Library, Layers, AI. Renders children per active tab.
- `[C] tracecraft/src/components/layout/StatusBar.tsx` — Shows: cursor coordinates,
  active tool name, DRC violation count, project name.
- `[C] tracecraft/src/components/common/ToolButton.tsx` — Styled icon button with
  `active` prop for selected tool state.
- `[C] tracecraft/src/stores/uiStore.ts` — Zustand store:
  ```typescript
  interface UIState {
    activeTool: ToolType;
    zoom: number;
    panOffset: Point;
    sidebarTab: 'properties' | 'library' | 'layers' | 'ai';
    sidebarOpen: boolean;
    // actions
    setTool: (tool: ToolType) => void;
    setZoom: (zoom: number) => void;
    setPan: (offset: Point) => void;
  }
  export type ToolType = 'select' | 'wire' | 'component' | 'route' | 'zone' | 'measure' | 'pan';
  ```
- `[M] tracecraft/src/App.tsx` — Render AppShell as root layout

**Verification:** `npm run tauri dev` shows the shell layout.

---

### [x] Task 10: Frontend — Shared Konva Canvas with Zoom/Pan

**Goal:** Reusable canvas component with infinite pan/zoom via Konva.

**Files:**
- `[C] tracecraft/src/components/common/KonvaCanvas.tsx`:
  - Wraps `<Stage>` from react-konva
  - Mouse wheel zoom (centered on cursor position)
  - Middle-click or Space+click drag to pan
  - Grid rendering (optional, togglable)
  - Exposes `worldToScreen(point)`, `screenToWorld(point)` via ref
  - Clips zoom between 0.1 and 100x
- `[C] tracecraft/src/lib/geometry.ts` — Point/Rect utilities for TS:
  ```typescript
  export interface Point { x: number; y: number; }
  export interface Rect { x: number; y: number; width: number; height: number; }
  export function distance(a: Point, b: Point): number;
  export function rotate(point: Point, center: Point, angle: number): Point;
  export function boundingBox(points: Point[]): Rect;
  ```
- `[C] tracecraft/src/lib/colors.ts` — Layer color definitions, theme colors

**Verification:** Canvas renders with grid, zoom/pan works smoothly.

---

### [x] Task 11: Frontend — IPC Hook & Tauri Event Hook

**Goal:** Typed hooks for Tauri IPC and event subscription.

**Files:**
- `[C] tracecraft/src/hooks/useTauriCommand.ts`:
  ```typescript
  export function useTauriCommand<TResult, TArgs = void>() {
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);
    const execute = async (command: string, args?: TArgs): Promise<TResult | null> => {
      // invoke, set loading/error, return result
    };
    return { execute, loading, error };
  }
  ```
- `[C] tracecraft/src/hooks/useTauriEvent.ts`:
  ```typescript
  export function useTauriEvent<T>(event: string, handler: (payload: T) => void, deps?: any[]) {
    useEffect(() => {
      const unlisten = listen<T>(event, (e) => handler(e.payload));
      return () => { unlisten.then(fn => fn()); };
    }, deps);
  }
  ```
- `[C] tracecraft/src/hooks/useCanvasInteraction.ts` — Unified mouse/kbd handler:
  returns handlers for `onMouseDown`, `onMouseMove`, `onMouseUp`, `onKeyDown`
  that reconcile screen/world coordinates and tool state.

**Verification:** Unit tests or manual test via simple round-trip invoke.

---

### [x] Task 12: Project Commands — Create/Open/Save/Close

**Goal:** IPC commands for project lifecycle.

**Files:**
- `[C] tracecraft/src-tauri/src/commands/mod.rs` — Module setup
- `[C] tracecraft/src-tauri/src/commands/project.rs`:
  ```rust
  #[tauri::command]
  fn create_project(state: State<AppState>, name: String, template: Option<String>) -> Result<ProjectSummary, String>;
  #[tauri::command]
  fn open_project(state: State<AppState>, path: String) -> Result<ProjectSummary, String>;
  #[tauri::command]
  fn save_project(state: State<AppState>) -> Result<(), String>;
  #[tauri::command]
  fn close_project(state: State<AppState>) -> Result<(), String>;
  #[tauri::command]
  fn get_recent_projects(state: State<AppState>) -> Result<Vec<RecentProject>, String>;
  ```
- `[C] tracecraft/src-tauri/src/model/project.rs`:
  ```rust
  pub struct Project { pub id: Uuid, pub name: String, pub path: String, pub created_at: String, pub updated_at: String }
  pub struct ProjectSummary { pub id: Uuid, pub name: String, pub path: String, pub updated_at: String }
  pub struct RecentProject { pub name: String, pub path: String, pub last_opened: String }
  ```
- `[M] tracecraft/src-tauri/src/main.rs` — Register project commands, file dialog plugins

**Verification:** Create project via Tauri invoke, verify SQLite file created.

---

### [x] Task 13: TypeScript Types — Sync with Rust Models

**Goal:** TypeScript interfaces matching Rust serialized types for IPC.

**Files:**
- `[C] tracecraft/src/types/project.ts`
- `[C] tracecraft/src/types/schematic.ts`
- `[C] tracecraft/src/types/pcb.ts`
- `[C] tracecraft/src/types/library.ts`

Each file mirrors the Rust struct with `interface` definitions. Uses `zod` schemas
for runtime validation of IPC responses.

**Verification:** TypeScript compiles without errors.

---

### [x] Task 14: Frontend Stores — Project & Library

**Goal:** Zustand stores bridging IPC responses to React state.

**Files:**
- `[C] tracecraft/src/stores/projectStore.ts`:
  ```typescript
  interface ProjectState {
    current: ProjectSummary | null;
    recent: RecentProject[];
    loading: boolean;
    create: (name: string) => Promise<void>;
    open: (path: string) => Promise<void>;
    save: () => Promise<void>;
    loadRecent: () => Promise<void>;
  }
  ```
- `[C] tracecraft/src/stores/libraryStore.ts`:
  ```typescript
  interface LibraryState {
    results: ComponentSummary[];
    selected: Component | null;
    query: string;
    loading: boolean;
    search: (q: string) => Promise<void>;
    select: (id: string) => Promise<void>;
  }
  ```

**Verification:** Stores work with mocked IPC in tests.

---

## Phase 2: Component Library (Tasks 15–22)

### [x] Task 15: Library Browser — Search Grid UI

**Goal:** Searchable grid of components from the local database.

**Files:**
- `[C] tracecraft/src/components/library/LibraryBrowser.tsx`:
  - Search input with debounce (300ms)
  - Grid of component cards (icon, name, category, manufacturer)
  - Click to select, shows detail in ComponentDetail
  - Pagination (20 per page)
- `[M] tracecraft/src/stores/libraryStore.ts` — Wire up search IPC calls

**Verification:** Type in search, results appear from DB.

---

### [~] Task 16: Library Browser — Component Detail View

**Goal:** Detailed view of a selected component with symbol preview.

**Files:**
- `[C] tracecraft/src/components/library/ComponentDetail.tsx`:
  - Symbol preview (SVG render from symbol data)
  - Parameter table (name/value pairs)
  - Footprint preview
  - 3D model indicator
  - "Place in Schematic" button → emits event

**Verification:** Click component in browser → detail panel shows symbol.

---

### [x] Task 17: Library Commands — Search & CRUD

**Goal:** Backend IPC commands for library operations.

**Files:**
- `[C] tracecraft/src-tauri/src/commands/library.rs`:
  ```rust
  #[tauri::command]
  fn search_components(state: State<AppState>, query: SearchQuery) -> Result<Vec<ComponentSummary>, String>;
  #[tauri::command]
  fn get_component_detail(state: State<AppState>, id: ComponentId) -> Result<Component, String>;
  #[tauri::command]
  fn create_component(state: State<AppState>, component: NewComponent) -> Result<ComponentId, String>;
  #[tauri::command]
  fn update_component(state: State<AppState>, id: ComponentId, component: UpdateComponent) -> Result<(), String>;
  #[tauri::command]
  fn delete_component(state: State<AppState>, id: ComponentId) -> Result<(), String>;
  ```
- `[M] tracecraft/src-tauri/src/main.rs` — Register library commands

**Verification:** `cargo test` or Tauri invoke round-trip.

---

### [~] Task 18: Symbol Editor — Canvas Drawing Tool

**Goal:** Create/edit schematic symbols with pin and body graphics.

**Files:**
- `[C] tracecraft/src/components/library/SymbolEditor.tsx`:
  - Konva canvas with grid
  - Draw: rectangle, circle, arc, line, text
  - Pin placement with snap-to-grid
  - Pin property editing (name, number, electrical type)
  - Undo/redo within editor
  - Export as SymbolDef (serialized JSON)
- `[M] tracecraft/src-tauri/src/commands/library.rs` — Add save_symbol command

**Verification:** Draw symbol, add pins, save, re-open renders same.

---

### [~] Task 19: Footprint Editor — Land Pattern Drawing Tool

**Goal:** Create/edit PCB footprints with pads and outline.

**Files:**
- `[C] tracecraft/src/components/library/FootprintEditor.tsx`:
  - Konva canvas with grid and layer color overlay
  - Pad placement: round/rect/oval with size/position
  - Pad property editing (number, shape, size, layer)
  - Outline drawing (board outline + courtyard)
  - Import from KiCad footprint data
  - 3D model association (file picker for STL/OBJ/GLTF/GLB)

**Verification:** Create footprint with pads, saved and reloaded correctly.

---

### [x] Task 20: KiCad Library Import

**Goal:** Import KiCad symbols and footprints into the local database.

**As-built:** `.kicad_sym`, `.pretty` and CSV BOM import are implemented, along with the
in-app official KiCad library download. Fidelity depends on the source constructs the
S-expression parser supports.

**Files:**
- `[C] tracecraft/src-tauri/src/io/kicad_sch.rs`:
  - S-expression parser for .kicad_sym (library format)
  - Extracts: symbol name, pins, body graphics, aliases
  - Converts to internal Component format
- `[C] tracecraft/src-tauri/src/io/kicad_pcb.rs`:
  - S-expression parser for .pretty (footprint format)
  - Extracts: pads, outline, courtyard, 3D model path
  - Converts to internal FootprintDef
- `[M] tracecraft/src-tauri/src/commands/library.rs` — Add import_kicad_library command
- `[M] tracecraft/src-tauri/src/io/mod.rs`

**Verification:** Import KiCad library, verify components searchable in browser.

---

### [~] Task 21: Library — Categories & Organization

**Goal:** Browseable category tree for the component library.

**Files:**
- `[C] tracecraft/src-tauri/src/db/migrations.rs` — Add category seed data
- `[M] tracecraft/src-tauri/src/db/library_repo.rs` — Category queries:
  ```rust
  fn get_category_tree(conn: &Connection) -> Result<Vec<CategoryTreeNode>>;
  fn get_components_by_category(conn: &Connection, category_id: &str) -> Result<Vec<ComponentSummary>>;
  ```
- `[M] tracecraft/src/components/library/LibraryBrowser.tsx` — Add category sidebar tree

**Categories (seed):** Resistors, Capacitors, Inductors, Diodes, Transistors, ICs,
Connectors, Power, RF, Sensors, Electromechanical, Miscellaneous

**Verification:** Browse categories, see filtered components.

---

### [~] Task 22: Component Library — Polish & Performance

**Goal:** Loading states, error handling, keyboard navigation.

**Files:**
- `[M] tracecraft/src/components/library/LibraryBrowser.tsx` — Add loading skeleton,
  empty state, error state, keyboard arrow navigation
- `[M] tracecraft/src/components/library/ComponentDetail.tsx` — Loading spinner,
  missing symbol placeholder, error boundary

**Verification:** Test all states (empty query, no results, network error).

---

## Phase 3: Schematic Editor (Tasks 23–38)

### [x] Task 23: Schematic Canvas — Display Sheet

**Goal:** Render a schematic sheet on the Konva canvas.

**Files:**
- `[C] tracecraft/src/components/schematic/SchematicCanvas.tsx`:
  - Uses shared KonvaCanvas wrapper
  - Renders: grid, sheet border, symbols (via SymbolRenderer), wires, net labels,
    junctions, text annotations
  - Reads state from schematicStore
  - Calls `get_schematic` IPC on mount
- `[C] tracecraft/src/stores/schematicStore.ts`:
  ```typescript
  interface SchematicState {
    sheet: SchematicSheet | null;
    selectedIds: Set<string>;
    activeTool: ToolType;
    loading: boolean;
    // actions
    loadSheet: (projectId: string, sheetId?: number) => Promise<void>;
    select: (id: string, add?: boolean) => void;
    clearSelection: () => void;
  }
  ```
- `[C] tracecraft/src/components/schematic/SymbolRenderer.tsx` — Renders a symbol:
  - Body shapes (rect, circle, arc, line)
  - Pins with pin numbers/names
  - Selection highlight

**Verification:** Load a sheet with symbols, all render in correct positions.

---

### [x] Task 24: Symbol Placement — Drag from Library

**Goal:** Drag a component from the library onto the schematic canvas.

**Files:**
- `[M] tracecraft/src/components/library/LibraryBrowser.tsx` — Make components draggable
- `[M] tracecraft/src/components/schematic/SchematicCanvas.tsx` — Drop target handler
- `[C] tracecraft/src-tauri/src/commands/schematic.rs`:
  ```rust
  #[tauri::command]
  fn place_symbol(state: State<AppState>, sheet_id: SheetId, component_id: ComponentId, position: Point) -> Result<SymbolId, String>;
  ```
- `[M] tracecraft/src-tauri/src/undo/command.rs` — Add `PlaceSymbol` command
- `[M] tracecraft/src-tauri/src/main.rs` — Register schematic commands

**Verification:** Drag component from library → appears on schematic canvas at drop position.

---

### [x] Task 25: Wire Drawing Tool

**Goal:** Click-to-connect wire drawing with snap-to-pin.

**Files:**
- `[C] tracecraft/src/components/schematic/WireTool.tsx`:
  - Click on pin → starts wire
  - Click empty space → adds vertex (corner)
  - Click on another pin → completes wire
  - Snap: wire start/end auto-connects to nearest pin (within 10px)
  - Right-click/Escape → cancels current wire
  - Visual feedback: ghost wire following cursor
- `[M] tracecraft/src-tauri/src/commands/schematic.rs` — Add `draw_wire`, `delete_element`
- `[M] tracecraft/src-tauri/src/undo/command.rs` — Add `DrawWire`, `DeleteElement` commands

**Verification:** Draw wire between two symbol pins, wire appears on canvas.

---

### [x] Task 26: Net Labels & Junctions

**Goal:** Named net labels for clean wiring and bus connections.

**Files:**
- `[C] tracecraft/src/components/schematic/SchematicToolbar.tsx` — Tool-specific options:
  net label name input, junction toggle
- `[M] tracecraft/src/components/schematic/SchematicCanvas.tsx` — Handle net label
  placement and junction insertion at wire intersections
- `[M] tracecraft/src-tauri/src/commands/schematic.rs` — Add `add_net_label`, `add_junction`
- `[M] tracecraft/src-tauri/src/undo/command.rs` — Corresponding commands

**Verification:** Place net labels, verify they display and associate with wires.

---

### [x] Task 27: Element Selection & Manipulation

**Goal:** Select, move, rotate, delete schematic elements.

**Files:**
- `[C] tracecraft/src/components/schematic/SelectionOverlay.tsx`:
  - Rubber-band selection (drag empty space)
  - Click to select single element
  - Shift+click to add to selection
  - Selection box with resize handles
  - Delete key removes selected elements
- `[M] tracecraft/src/components/schematic/SchematicCanvas.tsx` — Wire up selection
  events, move/rotate handlers
- `[M] tracecraft/src-tauri/src/commands/schematic.rs` — Add `move_elements`, `rotate_elements`,
  `delete_elements` batch commands

**Verification:** Select multiple symbols, move them, verify positions update.

---

### [x] Task 28: Netlist Extraction

**Goal:** Extract netlist from schematic connectivity.

**Files:**
- `[C] tracecraft/src-tauri/src/engine/netlist.rs`:
  ```rust
  pub fn extract_netlist(sheet: &SchematicSheet) -> Netlist;
  pub fn extract_netlist_multi(sheets: &[SchematicSheet]) -> Netlist;
  ```
  Algorithm: Walk all wires → group connected pins via wire segments + junctions →
  assign net names (user-defined labels or auto-generated `N$xxx`)
- `[M] tracecraft/src-tauri/src/commands/schematic.rs` — Add `get_netlist` command

**Verification:** Draw wires connecting pins → `extract_netlist` returns correct nets.

---

### [x] Task 29: Electrical Rule Check (ERC)

**Goal:** Detect floating pins, shorted outputs, unconnected nets.

**Files:**
- `[C] tracecraft/src-tauri/src/engine/erc.rs`:
  ```rust
  pub fn run_erc(sheet: &SchematicSheet) -> Vec<ERC violation>;
  pub struct ERC violation {
      pub severity: Severity, // Error | Warning
      pub rule: ERCRule,      // FloatingPin | ShortedOutput | UnconnectedNet | ...
      pub description: String,
      pub location: Point,
      pub elements: Vec<(ElementType, usize)>, // what's involved
  }
  ```
- `[M] tracecraft/src-tauri/src/commands/schematic.rs` — Add `run_erc` command
- `[M] tracecraft/src/components/schematic/SchematicCanvas.tsx` — Overlay violation
  markers (colored dots/underlines on offending elements)

**Verification:** Create schematic with floating pin → ERC returns violation.

---

### [x] Task 30: Auto-Annotation

**Goal:** Auto-assign reference designators (R1, R2, ..., C1, C2, ...).

**Files:**
- `[M] tracecraft/src-tauri/src/commands/schematic.rs` — `annotate_schematic`:
  - Groups symbols by library component type (resistor → R, capacitor → C, ...)
  - Assigns sequential numbers based on position (top-left to bottom-right)
  - Updates refdes on SymbolInstance
- `[M] tracecraft/src-tauri/src/undo/command.rs` — Add `Annotate` command

**Verification:** Place 5 resistors and 3 capacitors → annotate → R1-R5, C1-C3.

---

### [x] Task 31: Buses

**Goal:** Group related nets into buses for clean schematics.

**Files:**
- `[M] tracecraft/src/model/schematic.rs` — Bus type already defined
- `[M] tracecraft/src-tauri/src/commands/schematic.rs` — Add `create_bus`, `add_to_bus`
- `[M] tracecraft/src/components/schematic/SchematicCanvas.tsx` — Bus rendering
  (thick colored line with bus name label)
- `[C] tracecraft/src/components/schematic/BusRenderer.tsx` — Bus-specific rendering

**Verification:** Create bus with 4 members, wire connects display correctly.

---

### [x] Task 32: Hierarchical Sheets

**Goal:** Sub-schematics as reusable blocks.

**Files:**
- `[C] tracecraft/src/model/schematic.rs` — Add:
  ```rust
  pub struct HierarchicalSheet {
      pub id: SheetId,
      pub name: String,
      pub filename: String,
      pub pins: Vec<HierarchicalPin>, // entry/exit connections
  }
  pub struct HierarchicalPin { pub name: String, pub sheet_id: SheetId, pub pin_number: String }
  ```
- `[M] tracecraft/src-tauri/src/commands/schematic.rs` — Add `create_hierarchical_sheet`,
  `navigate_to_sheet`
- `[M] tracecraft/src/components/schematic/SchematicCanvas.tsx` — Render hierarchical
  sheet block with pins, double-click to descend

**Verification:** Create child sheet, add pins, navigate between parent/child.

---

### [~] Task 33: KiCad Schematic Import

**Goal:** Open `.kicad_sch` files in TraceCraft.

**As-built:** Basic symbols, wires, labels and sheets are mapped into the internal model.
Full KiCad 7/8 fidelity (embedded library symbols, complex graphics, buses) is not
guaranteed.

**Files:**
- `[M] tracecraft/src-tauri/src/io/kicad_sch.rs` — Extend to parse full schematic
  (not just library symbols):
  - Parse wires, junctions, net labels, hierarchical sheets
  - Map KiCad symbol instances to TraceCraft SymbolInstance
  - Handle text annotations
  - Support KiCad 7+ and 8 format

**Verification:** Open a real KiCad schematic → all elements render correctly.

---

### [~] Task 34: KiCad Schematic Export

**Goal:** Save TraceCraft schematics as `.kicad_sch` for KiCad interop.

**As-built:** Headers, symbols (reference/value), wires and net labels are written. Library
symbol definitions and pin data are not embedded, so KiCad needs the referenced libraries
to render the file properly. Not a full round-trip.

**Files:**
- `[M] tracecraft/src-tauri/src/io/kicad_sch.rs` — Add `export_schematic` function:
  - Generates KiCad S-expression format
  - Maps TraceCraft elements to KiCad equivalents
  - Handles: symbols (lib_symbols + instances), wires, labels, junctions

**Verification:** Export → import back → identical netlist.

---

### [~] Task 35: Schematic Undo/Redo UI

**Goal:** Toolbar buttons and keyboard shortcuts for undo/redo.

**Files:**
- `[M] tracecraft/src/components/layout/Toolbar.tsx` — Undo/redo buttons with shortcut
  labels (Ctrl+Z / Ctrl+Shift+Z)
- `[M] tracecraft/src/components/schematic/SchematicCanvas.tsx` — Keyboard event listener
  for Ctrl+Z / Ctrl+Y
- `[M] tracecraft/src-tauri/src/commands/schematic.rs` — Add `undo`, `redo` commands
  that delegate to CommandHistory

**Verification:** Place symbol, undo, symbol disappears. Redo, returns.

---

### [~] Task 36: Schematic — Print & Export Image

**Goal:** Export schematic as PNG/SVG/PDF.

**As-built:** PNG only, rendered from the Konva stage at 2× pixel ratio. SVG and PDF are
not implemented. The canvas API now exposes the stage; before the smoke-test fix the
action failed silently.

**Files:**
- `[M] tracecraft/src-tauri/src/commands/schematic.rs` — Add `export_schematic_image`:
  - Renders schematic to SVG (native) or PNG (via resvg or similar)
- `[C] tracecraft/src/components/common/Dialog.tsx` — Export dialog with format picker

**Verification:** Export → open in image viewer → matches canvas.

---

### [x] Task 37: Schematic Store — Full State Sync

**Goal:** Bidirectional sync between frontend state and Rust backend.

**Files:**
- `[M] tracecraft/src/stores/schematicStore.ts` — Full sync implementation:
  - On mount: fetch full sheet state
  - On each mutation: invoke IPC, on success update local state
  - Listen for remote events (e.g., from auto-save)
  - Debounce rapid mutations (wire drawing produces many points)

**Verification:** Schema edits persist across component unmount/remount.

---

### [~] Task 38: Schematic — Polish & Performance

**Goal:** Smooth interaction, loading states, error boundaries.

**As-built:** Basic loading/error state exists in stores; canvas virtualization, systematic
memoization and React error boundaries are not implemented.

**Files:**
- `[M] tracecraft/src/components/schematic/SchematicCanvas.tsx`:
  - Virtualization: only render elements visible in viewport
  - Layer organization: grid layer, symbol layer, wire layer, overlay layer
  - FPS counter (dev mode)
- `[M] tracecraft/src/components/schematic/SymbolRenderer.tsx` — Memoize symbol rendering
  with React.memo

**Verification:** 100+ symbols on sheet, zoom/pan stays smooth.

---

## Phase 4: PCB Layout Editor (Tasks 39–55)

### [x] Task 39: PCB Canvas — Display Board

**Goal:** Render PCB board on the Konva canvas with layer support.

**Files:**
- `[C] tracecraft/src/components/pcb/PcbCanvas.tsx`:
  - Uses shared KonvaCanvas
  - Renders: board outline, footprints (via FootprintRenderer), traces, vias, zones
  - Layer stack rendering (bottom layers darker, top layers bright)
  - Layer visibility toggle via LayerPanel
- `[C] tracecraft/src/stores/pcbStore.ts`:
  ```typescript
  interface PCBState {
    board: PcbBoard | null;
    selectedIds: Set<string>;
    activeLayer: LayerId;
    activeTool: ToolType;
    visibleLayers: Set<LayerId>;
    loading: boolean;
    // actions
    loadBoard: (projectId: string) => Promise<void>;
    select: (id: string, add?: boolean) => void;
    setActiveLayer: (layer: LayerId) => void;
    toggleLayer: (layer: LayerId) => void;
  }
  ```
- `[C] tracecraft/src/components/pcb/FootprintRenderer.tsx` — Renders a footprint:
  - Pads (colored by layer, shaped per PadDef)
  - Outline
  - Refdes text
  - Courtyard (dimmed)
  - Selection highlight

**Verification:** Load board with 5 footprints → all render with correct pad shapes/colors.

---

### [~] Task 40: Footprint Placement — Drag from Library

**Goal:** Place components from library onto PCB canvas.

**Files:**
- `[M] tracecraft/src/components/pcb/PcbCanvas.tsx` — Drop handler for library components
- `[C] tracecraft/src-tauri/src/commands/pcb.rs`:
  ```rust
  #[tauri::command]
  fn place_footprint(state: State<AppState>, component_id: ComponentId, position: Point, rotation: Rotation) -> Result<FootprintId, String>;
  ```
- `[M] tracecraft/src-tauri/src/undo/command.rs` — Add `PlaceFootprint` PCB command
- `[M] tracecraft/src-tauri/src/main.rs` — Register PCB commands

**Verification:** Drag component from library → footprint appears on PCB at drop position.

---

### [~] Task 41: PCB Stores — Schematic Netlist Sync

**Goal:** When schematic changes, PCB netlist updates with diff.

**Files:**
- `[M] tracecraft/src-tauri/src/engine/netlist.rs` — Add `diff_netlists`:
  ```rust
  pub fn diff_netlists(old: &Netlist, new: &Netlist) -> NetlistDiff;
  pub struct NetlistDiff {
      pub added: Vec<Net>,
      pub removed: Vec<Net>,
      pub modified: Vec<(String, Net, Net)>, // name, old, new
  }
  ```
- `[C] tracecraft/src-tauri/src/commands/pcb.rs` — `synchronize_netlist` command:
  - Extracts netlist from current schematics
  - Diffs with PCB's current netlist
  - Applies changes (add/remove nets, update ratsnest)

**Verification:** Add wire in schematic → PCB netlist has new net.

---

### [x] Task 42: Ratsnest Rendering

**Goal:** Show airwires (unrouted connections) between component pads.

**Files:**
- `[C] tracecraft/src/components/pcb/RatsnestRenderer.tsx`:
  - For each net in the netlist, get all connected pads
  - Compute minimal spanning tree (MST) for pad positions
  - Draw lines between connected pads in the MST
  - Color: unassigned net = yellow, assigned = dimmed
  - Update when netlist changes or components move
- `[M] tracecraft/src-tauri/src/engine/ratsnest.rs` (Backend):
  ```rust
  pub fn compute_ratsnest(board: &PcbBoard, netlist: &Netlist) -> Vec<RatsnestLine>;
  pub struct RatsnestLine { pub net_name: String, pub start: Point, pub end: Point }
  ```
  Uses Prim's or Kruskal's algorithm for MST.

**Verification:** Place two footprints with same net → ratsnest line between them.

---

### [x] Task 43: Manual Trace Routing Tool

**Goal:** Interactive trace drawing with click-to-place routing.

**Files:**
- `[C] tracecraft/src/components/pcb/TraceTool.tsx`:
  - Click start pad → begins trace
  - Click to add vertices (45° angle snapping by default)
  - Double-click/complete-click on target pad → finishes trace
  - Right-click removes last vertex
  - Visual feedback: ghost trace, via preview on layer change
  - Via insertion: pressing `V` inserts via at current cursor
  - Width controlled by active design rule
- `[M] tracecraft/src-tauri/src/commands/pcb.rs` — Add `route_trace`, `add_via`,
  `delete_element`
- `[M] tracecraft/src-tauri/src/undo/command.rs` — Corresponding commands

**Verification:** Route trace between two pads → trace appears on correct layer.

---

### [~] Task 44: Push & Shove Routing

**Goal:** Existing traces move aside when new trace conflicts.

**As-built:** `route_trace` calls `engine/shove.rs::resolve_conflicts`, which pushes
existing trace points aside and smooths them. It is a single-pass geometric shove, not
a full iterative router with rip-up/reroute.

**Files:**
- `[M] tracecraft/src-tauri/src/commands/pcb.rs` — Implement push-and-shove in Rust:
  - When new trace segment conflicts with existing trace:
    1. Find conflicting segments
    2. Compute alternative path around obstacle
    3. Adjust existing trace to accommodate new one
  - Fallback: highlight conflict, ask user to resolve manually
- This is a complex algorithm. Start with obstacle-avoidance snapping before full
  push-and-shove.

**Verification:** Route trace near existing one → existing trace shifts to maintain clearance.

---

### [~] Task 45: Board Outline Editor

**Goal:** Draw and edit the PCB board shape.

**Files:**
- `[M] tracecraft/src/components/pcb/PcbCanvas.tsx` — Board outline editing mode:
  - Click to add vertices
  - Drag vertices to reposition
  - Double-click to close shape
  - Cutout support (draw sub-polygon inside outline while holding Ctrl)
- `[M] tracecraft/src-tauri/src/commands/pcb.rs` — `set_board_outline` command
- `[M] tracecraft/src-tauri/src/undo/command.rs` — `SetBoardOutline` command

**Verification:** Draw rectangular board with rounded corners → outline renders.

---

### [x] Task 46: Copper Pours / Zones

**Goal:** Polygon fill zones with thermal relief.

**Files:**
- `[C] tracecraft/src/components/pcb/ZoneRenderer.tsx` — Renders filled polygon with
  clearance and thermal relief (using Konva Shape + fill)
- `[M] tracecraft/src-tauri/src/commands/pcb.rs` — Add `draw_zone`, `edit_zone`, `delete_zone`
- `[M] tracecraft/src-tauri/src/undo/command.rs` — Corresponding commands
- Compute zone fills in Rust: polygon triangulation + thermal spoke generation

**Verification:** Draw zone → polygon fills with clearance around traces and thermal
spokes on connected pads.

---

### [x] Task 47: Layer Panel & Stack-Up Management

**Goal:** View and control layer stack, per-layer visibility.

**Files:**
- `[C] tracecraft/src/components/pcb/LayerPanel.tsx`:
  - List of layers with: visibility toggle, color indicator, active layer highlight
  - Layer type icon (Top, Bottom, Inner, SolderMask, Silkscreen, Paste)
  - Drag to reorder (for inner layers)
  - Add/remove inner layers (2-layer → 4-layer)
- `[M] tracecraft/src-tauri/src/commands/pcb.rs` — `update_layer_stack` command

**Verification:** Toggle layer visibility → layer disappears/reappears on canvas.

---

### [x] Task 48: DRC Engine — Clearance & Width Checks

**Goal:** Detect design rule violations.

**Files:**
- `[M] tracecraft/src-tauri/src/engine/drc.rs`:
  ```rust
  pub fn run_drc(board: &PcbBoard) -> Vec<DRCViolation>;
  pub struct DRCViolation {
      pub severity: Severity,
      pub rule: DRCRule,
      pub description: String,
      pub location: Point,
      pub layer: LayerId,
      pub elements: Vec<(ElementType, usize)>,
  }
  ```
  Checks:
  - Trace-to-trace clearance
  - Trace-to-pad clearance
  - Pad-to-pad clearance
  - Minimum trace width
  - Minimum annular ring (pad diameter - hole diameter)
  - Minimum hole size
  - Silkscreen overlap with pads
- `[C] tracecraft/src-tauri/src/commands/drc.rs`:
  ```rust
  #[tauri::command]
  fn run_drc(state: State<AppState>) -> Result<Vec<DRCViolation>, String>;
  #[tauri::command]
  fn get_drc_settings(state: State<AppState>) -> Result<DRCSettings, String>;
  #[tauri::command]
  fn update_drc_settings(state: State<AppState>, settings: DRCSettings) -> Result<(), String>;
  ```

**Verification:** Route two traces too close → DRC returns violation.

---

### [x] Task 49: DRC Visualization on Canvas

**Goal:** Show DRC violations as colored markers on the PCB canvas.

**Files:**
- `[C] tracecraft/src/components/pcb/DRCOverlay.tsx`:
  - Renders violation markers at violation locations
  - Color: red = error, yellow = warning
  - Click marker → shows violation detail in sidebar
  - Violations persist until re-run DRC
- `[M] tracecraft/src/components/pcb/PcbCanvas.tsx` — Render DRC overlay on top layer

**Verification:** Run DRC → violations visible on canvas as colored dots.

---

### [~] Task 50: Netlist Synchronization — Bidirectional

**Goal:** Changes in PCB can update schematic (and vice versa).

**As-built:** `sync_netlist` is the primary schematic → PCB path and writes pin-level nets.
`sync_pcb_to_schematic` only adds missing net labels for user-named nets; it does not sync
footprint positions, values or deletions.

**Files:**
- `[M] tracecraft/src-tauri/src/engine/netlist.rs` — Add `sync_pcb_to_schematic`:
  - If user deletes a footprint in PCB, should it remove the symbol from schematic?
  - Approach: PCB is downstream of schematic. PCB changes that affect netlist
    (e.g., adding a via that connects two nets) update the schematic net labels.

**Verification:** Add net in schematic → PCB gets new net. (One-way is fine for MVP.)

---

### [~] Task 51: Viewport Commands — Zoom to Fit, Center

**Goal:** Smart viewport navigation.

**Files:**
- `[C] tracecraft/src-tauri/src/commands/viewport.rs`:
  ```rust
  #[tauri::command]
  fn get_board_bounds(state: State<AppState>) -> Result<Rect, String>;
  ```
- `[M] tracecraft/src/components/layout/Toolbar.tsx` — Add zoom-to-fit button,
  zoom-to-selection button

**Verification:** Click "Zoom to fit" → entire board visible in canvas.

---

### [~] Task 52: PCB Undo/Redo

**Goal:** Full undo/redo for PCB layout operations.

**Files:**
- `[M] tracecraft/src-tauri/src/undo/command.rs` — Ensure all PCB commands implement
  Command trait. Verify trace routing undo restores previous state.
- `[M] tracecraft/src/components/pcb/PcbCanvas.tsx` — Keyboard shortcuts for Ctrl+Z/Y

**Verification:** Route trace, undo → trace gone. Redo → trace restored.

---

### [x] Task 53: PCB Selection & Group Operations

**Goal:** Select, move, rotate, align PCB elements.

**Files:**
- `[C] tracecraft/src/components/pcb/SelectionOverlay.tsx` (similar to schematic version):
  - Rubber-band selection
  - Multi-select with Shift
  - Move selected group
  - Rotate (R key, increments of 45°)
  - Align tools (left, right, top, bottom, center)
  - Lock toggle (prevents accidental moves)
- `[M] tracecraft/src-tauri/src/commands/pcb.rs` — Batch move, rotate lock commands

**Verification:** Select 3 footprints, move them together, positions update.

---

### [~] Task 54: PCB Properties Panel

**Goal:** Edit selected element properties in the sidebar.

**Files:**
- `[C] tracecraft/src/components/common/PropertyPanel.tsx` — Generic property editor:
  - Renders a list of (label, input) pairs
  - Supports: text, number, dropdown, color, boolean
  - Emits `onChange` for each property
- `[M] tracecraft/src/components/layout/Sidebar.tsx` — Show PropertyPanel when element selected
- `[M] tracecraft/src/components/pcb/PcbCanvas.tsx` — Send selected element properties
  to sidebar

**Verification:** Select trace → sidebar shows trace width, layer, net name. Change
width → trace updates.

---

### [~] Task 55: PCB Polish & Performance

**Goal:** Smooth performance with large boards.

**As-built:** No dedicated large-board profiling, culling, memoization or IPC throttling has
been done. The smoke test used a four-component board.

**Files:**
- `[M] tracecraft/src/components/pcb/PcbCanvas.tsx`:
  - Layer-based rendering (only visible layers)
  - Viewport culling (skip elements outside view)
  - Throttled ratsnest updates (only recompute MST after drag ends)
- `[M] tracecraft/src/components/pcb/FootprintRenderer.tsx` — React.memo with
  deep comparison

**Verification:** Board with 200 footprints + 500 traces → smooth zoom/pan.

---

## Phase 5: Auto-Routing (Tasks 56–61)

### [x] Task 56: A* Grid Router — Single Net

**Goal:** Route a single net between two pads using A* pathfinding.

**Files:**
- `[C] tracecraft/src-tauri/src/engine/autorouter.rs`:
  ```rust
  pub struct AStarRouter {
      grid_resolution: f64,     // e.g., 0.1mm grid cells
      via_cost: f64,
      trace_cost_per_unit: f64,
      preferred_directions: HashMap<LayerId, Direction>,
      clearance: f64,
  }
  impl AStarRouter {
      pub fn route_net(&self, board: &PcbBoard, net_id: NetId) -> Result<Vec<TraceSegment>>;
  }
  ```
  Algorithm:
  1. Create cost grid from board state (obstacles = existing traces, pads, vias)
  2. Heuristic: Manhattan distance (with preferred direction bonus)
  3. Search: standard A* on grid nodes
  4. Post-process: simplify path (remove collinear points), smooth corners
  5. Assign resulting segments to layers, insert vias where needed

**Verification:** Route single net between two pads far apart → continuous trace
avoiding obstacles.

---

### [x] Task 57: Multi-Layer A* Routing

**Goal:** Route nets across multiple layers with via insertion.

**Files:**
- `[M] tracecraft/src-tauri/src/engine/autorouter.rs`:
  - Extend A* to 3D grid (x, y, layer)
  - Via cost: penalty for transitioning between layers
  - Layer direction preferences: Top = horizontal, Bottom = vertical (user configurable)
  - Via placement: only at grid points, not in middle of traces

**Verification:** Route net on 2-layer board → routes on both layers with vias.

---

### [x] Task 58: Fan-Out Router

**Goal:** Automated fan-out for dense packages (BGA, QFP).

**Files:**
- `[C] tracecraft/src-tauri/src/engine/fanout.rs`:
  ```rust
  pub struct FanOutRouter { /* pattern type, via pattern, escape direction */ }
  impl FanOutRouter {
      pub fn fanout(&self, board: &PcbBoard, footprint_id: FootprintId) -> Result<Vec<Trace>>;
  }
  ```
  Patterns: dog-bone (BGA), perimeter (QFP), staggered (fine-pitch BGA)
  Algorithm: For each pad, route a short trace to a via placed at a fan-out position
  (between pads or in a ring around the package)

**Verification:** Fan-out a BGA footprint → all pads have short traces to vias.

---

### [~] Task 59: Batch Router — Route All Nets

**Goal:** Route all unrouted nets on the board in order.

**As-built:** `autoroute_all` computes A* routes and returns them with progress events,
but the returned traces are not inserted into the board, no vias are emitted at layer
transitions, and no UI control invokes it yet. The verification (traces appear) is not met.

**Files:**
- `[M] tracecraft/src-tauri/src/engine/autorouter.rs` — Add `BatchRouter`:
  ```rust
  pub struct BatchRouter {
      strategy: RoutingStrategy,
      order: RoutingOrder, // ByLength | ByArea | ByPins
  }
  impl BatchRouter {
      pub fn route_all(&self, board: &PcbBoard, nets: &[NetId]) -> Result<Vec<RouteResult>>;
  }
  ```
  Strategy:
  1. Sort nets by heuristic complexity (shortest first, fewest pins first)
  2. Route each net with A* router
  3. If routing fails, skip and report
  4. After first pass, attempt failed nets with relaxed constraints

**Verification:** 10 unrouted nets → batch route → all routed (or failure report).

---

### [~] Task 60: Routing Commands & UI

**Goal:** User control over auto-routing from the frontend.

**As-built:** The `autorouteAll`/`cancelRouting` store actions invoke the commands and
listen for progress/done events, but no toolbar or menu control calls them. Persisting the
returned routes and a progress/cancel UI are missing.

**Files:**
- `[C] tracecraft/src-tauri/src/commands/routing.rs`:
  ```rust
  #[tauri::command]
  fn autoroute_net(state: State<AppState>, net_id: NetId, strategy: Option<RoutingStrategy>) -> Result<RouteResult, String>;
  #[tauri::command]
  fn autoroute_all(state: State<AppState>, strategy: Option<RoutingStrategy>) -> Result<Vec<RouteResult>, String>;
  #[tauri::command]
  fn fanout_component(state: State<AppState>, footprint_id: FootprintId) -> Result<Vec<RouteResult>, String>;
  #[tauri::command]
  fn get_routing_status(state: State<AppState>) -> Result<RoutingStatus, String>;
  ```
- `[M] tracecraft/src/components/pcb/PcbToolbar.tsx` — Auto-route buttons with
  strategy dropdown

**Verification:** Click "Auto-Route All" → traces appear, status shows success/failure.

---

### [~] Task 61: Routing — Progress & Cancellation

**Goal:** Show routing progress, allow cancellation.

**As-built:** `autoroute_all` emits `routing-progress`/`routing-done` and honours the
`cancel_routing` flag. The store listens for both events, but there is no UI trigger,
progress bar or cancel button yet.

**Files:**
- `[M] tracecraft/src-tauri/src/commands/routing.rs` — Add streaming progress via
  Tauri events:
  ```rust
  app_handle.emit("routing-progress", RoutingProgress { current: 5, total: 50, net: "N$12" });
  ```
- `[M] tracecraft/src/components/pcb/PcbToolbar.tsx` — Progress bar with cancel button
- `[M] tracecraft/src-tauri/src/engine/autorouter.rs` — Check cancellation flag

**Verification:** Route 50 nets → progress updates in real time, cancel stops routing.

---

## Phase 6: 3D Preview (Tasks 62–65)

### [x] Task 62: 3D Preview Canvas Setup

**Goal:** Three.js canvas that displays the PCB board in 3D.

**Files:**
- `[C] tracecraft/src/components/preview/Preview3D.tsx`:
  - Three.js scene with orbit controls
  - React-Three-Fiber integration
  - Lighting: ambient + directional
  - Grid helper plane
  - Board substrate rendering (green FR4 rectangle with board outline)
  - Copper layer visualization (traces as extruded shapes on substrate)
  - Component 3D model rendering at footprint positions
- `[M] tracecraft/src/components/layout/Sidebar.tsx` — Add "3D" tab

**Verification:** Open 3D preview → board substrate rendered with copper traces visible.

---

### [~] Task 63: 3D Model Loading

**Goal:** Load STL/OBJ/GLTF/GLB 3D models associated with footprints.

**As-built:** `get_3d_model_data` returns positions/normals for STL, OBJ, glTF and GLB
(embedded or external buffers). ModelLoader caches geometry and applies footprint
position/rotation. STEP/WRL import is not implemented.

**Files:**
- `[C] tracecraft/src/components/preview/ModelLoader.tsx`:
  - Parse STL/OBJ/GLTF/GLB into Three.js geometry through `get_3d_model_data`
  - STEP import is not implemented (STEP export is)
  - Cache loaded models in memory
  - Place model at footprint position with correct rotation
- `[M] tracecraft/src-tauri/src/commands/pcb.rs` — `get_3d_model_data` command that
  returns model path from footprint

**Verification:** Footprint with 3D model → model appears at correct position in preview.

---

### [x] Task 64: 3D Layer Visibility

**Goal:** Toggle visibility of copper, soldermask, silkscreen in 3D.

**Files:**
- `[M] tracecraft/src/components/preview/Preview3D.tsx` — Layer visibility controls:
  - Checkbox per layer group: Copper, Soldermask, Silkscreen, Substrate
  - Soldermask: semi-transparent colored coating over copper
  - Silkscreen: white text/lines on top of soldermask
- `[M] tracecraft/src/components/pcb/LayerPanel.tsx` — Sync 2D layer visibility
  to 3D preview

**Verification:** Toggle soldermask off → copper visible without green coating.

---

### [~] Task 65: 3D Measurement Tools

**Goal:** Measure distances and angles in the 3D viewport.

**As-built:** Two-point distance measurement only; angle measurement is not implemented.

**Files:**
- `[M] tracecraft/src/components/preview/Preview3D.tsx` — Measurement mode:
  - Click two points → shows distance
  - Click three points → shows angle
  - Measurement labels in screen space (CSS2DRenderer)
  - Snaps to pad centers and trace vertices when cursor is near

**Verification:** Click two pad centers → distance displayed correctly.

---

## Phase 7: AI Copilot (Tasks 66–70)

### [~] Task 66: ONNX Runtime Integration

**Goal:** Load and run ONNX models for component recommendations and routing assistance.

**As-built:** Not implemented. `ort` is commented out in `Cargo.toml`; AI commands use
keyword/template heuristics instead.

**Files:**
- `[M] tracecraft/src-tauri/Cargo.toml` — Add `ort` crate
- `[C] tracecraft/src-tauri/src/ai/mod.rs` — AI engine initialization:
  ```rust
  pub struct AIEngine {
      recommender: Option<Session>,  // ONNX session
      routing_assist: Option<Session>,
      chat_engine: Option<ChatEngine>,
  }
  impl AIEngine {
      pub fn new(models_dir: &str) -> Result<Self>;
      pub fn is_available(&self) -> bool;
  }
  ```
- Models are loaded from `~/.tracecraft/models/` directory
- Graceful degradation if models not present (AI features show "download model" prompt)

**Verification:** AIEngine initializes without crashing when models absent.

---

### [~] Task 67: Component Recommendations

**Goal:** Suggest components based on netlist context.

**As-built:** Keyword matching against the local component catalog; no ONNX model,
ranking model or netlist-aware scoring.

**Files:**
- `[C] tracecraft/src-tauri/src/ai/recommender.rs`:
  - Input: netlist context (nearby nets, pin types, signal names)
  - Output: ranked list of component recommendations
  - Uses an ONNX model (e.g., a small BERT-like model trained on component data)
  - Fallback: keyword matching when model not available
- `[C] tracecraft/src/components/ai/RecommenderWidget.tsx`:
  - Shows in sidebar when a net or pin is selected
  - Displays: component name, match score, quick-add button
- `[M] tracecraft/src-tauri/src/commands/ai.rs` — `recommend_components` command

**Verification:** Select a net → relevant components appear in recommender.

---

### [~] Task 68: Routing Assistance

**Goal:** Suggest trace paths during manual routing.

**As-built:** Heuristic straight-line path suggestion from the AI command; no ghost
trace overlay, model inference or accept-with-Enter UI.

**Files:**
- `[C] tracecraft/src-tauri/src/ai/routing_assist.rs`:
  - Input: board state, cursor position, target net
  - Output: suggested trace path (series of points)
  - Uses an ONNX model (CNN on grid cells) or heuristic (A* warm-start)
  - Displays suggestion as ghost trace on canvas
- `[C] tracecraft/src/components/ai/RouteSuggestion.tsx`:
  - Ghost trace overlay while routing
  - Tab key to cycle through alternative suggestions
  - Enter to accept suggestion
- `[M] tracecraft/src-tauri/src/commands/ai.rs` — `suggest_route` command

**Verification:** While routing a net, AI suggests path → accept with Enter.

---

### [~] Task 69: LLM Chat for PCB Design

**Goal:** Natural language Q&A about PCB design.

**As-built:** Template-based replies in `ai/chat.rs`; no llama.cpp binding, model file,
streaming events or board-context inference.

**Files:**
- `[C] tracecraft/src-tauri/src/ai/chat.rs`:
  - Uses llama.cpp bindings (`llama-cpp-2` crate)
  - Model: a small quantized model (e.g., Phi-3-mini, Llama-3.2-1B)
  - Context: includes current board state, selected elements, design rules
  - System prompt: "You are a PCB design assistant..."
  - Streaming response via Tauri events
- `[C] tracecraft/src/components/ai/ChatWindow.tsx`:
  - Chat interface with message history
  - Streaming text rendering
  - Context-aware: "What's wrong with my current selection?" → analyzes selected element
  - Suggested prompts: "How should I route this net?", "Is my board ready for fab?"
- `[C] tracecraft/src/components/ai/AIPanel.tsx` — Container for all AI widgets

**Verification:** Ask "What clearance should I use for 5V nets?" → AI responds.

---

### [~] Task 70: AI — Model Download & Management

**Goal:** Download AI models on demand, show availability status.

**As-built:** Not implemented; no model directory, download endpoint or availability UI.

**Files:**
- `[M] tracecraft/src-tauri/src/ai/mod.rs` — Add model download logic:
  - Check `~/.tracecraft/models/` for required models
  - Download button triggers HTTP download with progress
  - Models hosted on Hugging Face or similar
- `[M] tracecraft/src/components/ai/AIPanel.tsx` — Status indicator:
  - Green dot: model loaded
  - Yellow dot: downloading (with progress %)
  - Red dot: not available (show download button)

**Verification:** Remove model file → app shows download prompt → download completes.

---

## Phase 8: File I/O & Polish (Tasks 71–76)

### [x] Task 71: Native .tcproj Format

**As-built:** `.tcproj` is a pretty-printed JSON document containing project metadata,
the full `PcbBoard`, and all schematic sheets. SQLite stores the component library, app
settings and the last-project path, but not the project file itself.

**Files:**
- `src-tauri/src/io/native.rs` — JSON save/load
- `src-tauri/src/commands/project.rs` — create/open/save/close

**Verification:** Save → close → reopen restores board, sheets and metadata (smoke-tested
against the packaged `.deb`).

---

### [~] Task 72: Gerber Export

**As-built:** one RS-274X file with every trace segment and zone outline. It does not yet
emit pads, vias, solder mask, silkscreen or an NC drill file, so the output is not
manufacturable as-is. The exporter is wired to File → Export Gerber.

**Files:** `src-tauri/src/io/gerber.rs`, `src-tauri/src/commands/project.rs`

**Remaining:** per-layer files, pads/vias, aperture macros, solder mask/silk and drill.

---

### [~] Task 73: IPC-2581 Export

**As-built:** hand-written XML with layer stack, net list and trace sections. It is not
validated against the IPC-2581 schema and does not include full component/padstack data.

**Files:** `src-tauri/src/io/ipc2581.rs`

---

### [~] Task 74: BOM Export

**As-built:** CSV BOM with `Reference, Value, Component ID, Quantity`, plus a separate CPL
CSV (`Reference, Mid X, Mid Y, Rotation, Side`). `rust_xlsxwriter` is a dependency, but
XLSX output is not wired up. Manufacturer/part-number columns are not populated.

**Files:** `src-tauri/src/io/bom.rs`

---

### [~] Task 75: Error Handling & Logging

**As-built:** `tracing_subscriber` initializes with `RUST_LOG`/`tracecraft=debug` and logs
to the process output; there is no log file or rotation. `commands/errors.rs` defines an
`AppError`, but most commands still return `String` errors and the frontend displays them
via store error state/toasts.

**Files:** `src-tauri/src/lib.rs`, `src-tauri/src/commands/errors.rs`

---

### [~] Task 76: Final Polish & Performance

**As-built:** dark copper theme, window sizing, icons and `.deb`/AppImage packaging exist.
File associations (`.tcproj`), canvas virtualization, systematic memoization, IPC
throttling and a frame-time budget have not been completed. AppImage packaging and its
documented fallback command are covered in `docs/IMPLEMENTATION.md`.

---

## Phase 9: Simulation, Mechanical Export & Smoke-Test Fixes

This phase records work completed after the original 76-task plan.

### [x] Task 77: Pin-Aware Netlist Extraction

Replaced name-based connectivity with pin geometry and union-find connectivity:

- Pin transforms account for symbol rotation/mirror and pin length.
- Wires connect at endpoints and T-junctions; crossings only connect at junctions.
- Pin-to-pin nets, labels, GND/power symbols and `ComponentMap` resolution.
- Backend tests cover series, parallel, junction and label nets.

**Files:** `src-tauri/src/engine/netlist.rs`, `src-tauri/tests/simulation.rs`

---

### [x] Task 78: SPICE Netlist Generation

- Device classification: R, C, L, D, LED, NPN, PNP, VCC, GND.
- Node sanitization, refdes hygiene, value/unit normalization (`100nF` → `100n`).
- Model lines, analysis validation and warnings for unsupported parts.
- `.option filetype=ascii` is emitted so ngspice produces a parseable rawfile.
- Safe defaults: 5 V, 1 kΩ, 100 nF when parameters are missing.

**Files:** `src-tauri/src/io/spice.rs`, `src-tauri/src/commands/project.rs`

---

### [x] Task 79: ngspice Runner and Simulation Panel

- `simulate_schematic` runs the open schematic without a netlist path round-trip.
- ngspice output is parsed from the ASCII rawfile into plots/vectors/points.
- Sidebar Sim tab: transient/AC/DC/operating-point presets, editable directive, Run,
  `.cir` export, SVG plot and generated-netlist view.
- Requires `ngspice` on `PATH`; it is not bundled.

**Files:** `src/components/schematic/SimulationPanel.tsx`,
`src-tauri/src/commands/project.rs`, `src-tauri/tests/simulation.rs`

**Verified:** packaged `.deb`, 108-point transient run with 4 vectors and a rendered plot.

---

### [x] Task 80: AP203 STEP Export

- Generates `MANIFOLD_SOLID_BREP` with `ADVANCED_FACE`, `EDGE_LOOP`,
  `ORIENTED_EDGE`, `EDGE_CURVE` and `LINE` entities.
- Board, trace segments, vias and extruded component footprint outlines.
- Validated with OCCT on the final smoke board: 9 closed solids, 62 faces,
  13,452.41 mm³, no read errors.
- Board cutouts are **not** exported yet; copper is represented as simple prisms.

**Files:** `src-tauri/src/io/step.rs`, `src-tauri/src/commands/pcb.rs`

---

### [x] Task 81: GLTF/GLB Model Loading

- Real `gltf` 0.14 parser for `.gltf` and `.glb`, including external `.bin` buffers,
  embedded GLB chunks, accessors/indices and node transforms.
- STL (ASCII/binary) and OBJ loaders retained.
- Tests cover ASCII STL, external-buffer GLTF and indexed GLB.

**Files:** `src-tauri/src/commands/pcb.rs`, `src-tauri/tests/model3d.rs`

---

### [x] Task 82: Smoke-Test Fixes (Packaged .deb and AppImage)

Verified with WebDriver against the installed `.deb` and fixed:

- Coordinates/refdes/value are assigned when placing PCB footprints; schematic refdes are
  reused when available. Ratsnest now finds footprints (was 0 lines for a wired board).
- BOM/CPL/KiCad PCB exports now include references and values.
- Gerber and KiCad PCB exports emit every segment of multi-point traces.
- Schematic PNG export now receives the Konva stage (previously always failed silently).
- App version shown in the UI matches `package.json`/`tauri.conf.json` (0.2.0); the About
  dialog no longer claims full KiCad compatibility.
- Final smoke test: save → close → reopen preserved 4 footprints, 3 traces, 1 via,
  4 symbols and 3 wires; simulation returned 108 points / 4 vectors; all eight export
  commands produced files; STEP validated as 9 solids / 62 faces / 13,452.41 mm³.
- The schematic canvas exported a valid 2160×1616 PNG. The native save dialog itself is
  not scriptable on this display, so the dialog-granted fs write was not automated.
- The 105 MB AppImage launches with `APPIMAGE_EXTRACT_AND_RUN=1` and shows v0.2.0.
  AppImage bundling takes ~10–12 minutes in the GTK/WebKit deploy step; it is slow, not
  hung. One build exited with a transient `failed to run linuxdeploy`; rerunning
  `npx tauri bundle --bundles appimage -v` completed successfully.

---

### [ ] Task 83: Remaining Export & Routing Gaps

- STEP board cutouts and realistic copper/mask geometry.
- Gerber pads/vias/drill/mask/silkscreen and per-layer files; IPC-2581 schema validation.
- KiCad schematic library embedding and PCB pads/stackup for true round-trip.
- Persist auto-router output (currently returns preview paths only) and emit vias at layer
  transitions.
- Real pad geometry for DRC/ratsnest instead of footprint origins.
- XLSX BOM, SVG/PDF schematic export, full UI undo/redo, file associations.
