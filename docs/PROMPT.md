# TraceCraft — Single-Shot Build Prompt

> **Historical document.** This is the original build prompt, preserved for context. It is
> not a status report. See `docs/SPECIFICATION.md` and `docs/TASKS.md` for as-built behavior
> and known limitations. AI inference (ONNX/llama.cpp) was not enabled in the shipped build.
>
> Build a local-first, AI-assisted PCB design desktop app called **TraceCraft**.
> Execute every task in order. Create every file listed. Do not skip steps.

---

## Project Overview

TraceCraft is a desktop PCB design tool (like a local Flux.ai) for hobbyists and
engineers. It runs via Tauri 2.x with a React frontend and Rust backend. All data
stays local — no cloud, no accounts.

**Key features:** Schematic editor, PCB layout editor, auto-routing (A*), component
library (SQLite), DRC, 3D preview (Three.js), AI copilot (ONNX + llama.cpp).

---

## Tech Stack

| Layer | Choice |
|-------|--------|
| Desktop shell | Tauri 2.x |
| Frontend | React 18 + TypeScript + Vite |
| 2D rendering | react-konva (Canvas) |
| 3D rendering | Three.js / @react-three/fiber |
| State | Zustand |
| Styling | Tailwind CSS |
| Backend | Rust (Tauri commands) |
| Database | SQLite via rusqlite |
| Auto-router | Custom A* pathfinding in Rust |
| AI inference | ONNX Runtime (ort crate) |
| AI chat | llama.cpp (llama-cpp-2 crate) |
| Icons | lucide-react |

---

## Directory Structure

```
tracecraft/
├── src-tauri/
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── capabilities/default.json
│   ├── icons/                     # App icons (use placeholder)
│   └── src/
│       ├── main.rs
│       ├── app_state.rs
│       ├── model/
│       │   ├── mod.rs
│       │   ├── id.rs
│       │   ├── geometry.rs
│       │   ├── arena.rs
│       │   ├── project.rs
│       │   ├── schematic.rs
│       │   ├── pcb.rs
│       │   ├── library.rs
│       │   └── netlist.rs
│       ├── commands/
│       │   ├── mod.rs
│       │   ├── project.rs
│       │   ├── schematic.rs
│       │   ├── pcb.rs
│       │   ├── library.rs
│       │   ├── routing.rs
│       │   ├── drc.rs
│       │   ├── ai.rs
│       │   └── viewport.rs
│       ├── engine/
│       │   ├── mod.rs
│       │   ├── autorouter.rs
│       │   ├── fanout.rs
│       │   ├── drc.rs
│       │   ├── erc.rs
│       │   ├── netlist.rs
│       │   └── ratsnest.rs
│       ├── io/
│       │   ├── mod.rs
│       │   ├── native.rs
│       │   ├── kicad_sch.rs
│       │   ├── kicad_pcb.rs
│       │   ├── gerber.rs
│       │   ├── ipc2581.rs
│       │   └── bom.rs
│       ├── db/
│       │   ├── mod.rs
│       │   ├── migrations.rs
│       │   ├── library_repo.rs
│       │   └── project_store.rs
│       ├── ai/
│       │   ├── mod.rs
│       │   ├── recommender.rs
│       │   ├── routing_assist.rs
│       │   └── chat.rs
│       └── undo/
│           ├── mod.rs
│           └── command.rs
├── src/
│   ├── main.tsx
│   ├── App.tsx
│   ├── vite-env.d.ts
│   ├── components/
│   │   ├── layout/
│   │   │   ├── AppShell.tsx
│   │   │   ├── Toolbar.tsx
│   │   │   ├── Sidebar.tsx
│   │   │   └── StatusBar.tsx
│   │   ├── schematic/
│   │   │   ├── SchematicCanvas.tsx
│   │   │   ├── SymbolRenderer.tsx
│   │   │   ├── WireTool.tsx
│   │   │   ├── SelectionOverlay.tsx
│   │   │   ├── BusRenderer.tsx
│   │   │   └── SchematicToolbar.tsx
│   │   ├── pcb/
│   │   │   ├── PcbCanvas.tsx
│   │   │   ├── FootprintRenderer.tsx
│   │   │   ├── TraceTool.tsx
│   │   │   ├── RatsnestRenderer.tsx
│   │   │   ├── ZoneRenderer.tsx
│   │   │   ├── DRCOverlay.tsx
│   │   │   ├── SelectionOverlay.tsx
│   │   │   ├── PcbToolbar.tsx
│   │   │   └── LayerPanel.tsx
│   │   ├── preview/
│   │   │   ├── Preview3D.tsx
│   │   │   └── ModelLoader.tsx
│   │   ├── library/
│   │   │   ├── LibraryBrowser.tsx
│   │   │   ├── ComponentDetail.tsx
│   │   │   ├── SymbolEditor.tsx
│   │   │   └── FootprintEditor.tsx
│   │   ├── ai/
│   │   │   ├── AIPanel.tsx
│   │   │   ├── RecommenderWidget.tsx
│   │   │   ├── RouteSuggestion.tsx
│   │   │   └── ChatWindow.tsx
│   │   └── common/
│   │       ├── KonvaCanvas.tsx
│   │       ├── PropertyPanel.tsx
│   │       ├── Dialog.tsx
│   │       └── ToolButton.tsx
│   ├── stores/
│   │   ├── projectStore.ts
│   │   ├── schematicStore.ts
│   │   ├── pcbStore.ts
│   │   ├── libraryStore.ts
│   │   ├── aiStore.ts
│   │   └── uiStore.ts
│   ├── hooks/
│   │   ├── useTauriCommand.ts
│   │   ├── useCanvasInteraction.ts
│   │   └── useTauriEvent.ts
│   ├── types/
│   │   ├── project.ts
│   │   ├── schematic.ts
│   │   ├── pcb.ts
│   │   └── library.ts
│   ├── lib/
│   │   ├── geometry.ts
│   │   ├── colors.ts
│   │   └── constants.ts
│   └── styles/
│       ├── index.css
│       └── preview.css
├── package.json
├── tsconfig.json
├── tsconfig.node.json
├── vite.config.ts
├── tailwind.config.js
├── postcss.config.js
├── index.html
└── .gitignore
```

---

## Ordered Build Checklist

Create every file below in order. Each file's content is described; inline critical
code where implementation is non-obvious.

---

### Step 1: Project Scaffold

Create `tracecraft/` directory. Run `npm create tauri-app` or scaffold manually.

**Files to create:**

#### `tracecraft/package.json`
```json
{
  "name": "tracecraft",
  "private": true,
  "version": "0.1.0",
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "tsc && vite build",
    "preview": "vite preview",
    "tauri": "tauri"
  },
  "dependencies": {
    "react": "^18.3.1",
    "react-dom": "^18.3.1",
    "@tauri-apps/api": "^2.0.0",
    "@tauri-apps/plugin-dialog": "^2.0.0",
    "@tauri-apps/plugin-fs": "^2.0.0",
    "konva": "^9.3.0",
    "react-konva": "^18.2.10",
    "three": "^0.168.0",
    "@react-three/fiber": "^8.17.0",
    "@react-three/drei": "^9.111.0",
    "zustand": "^4.5.0",
    "lucide-react": "^0.400.0",
    "react-hot-toast": "^2.4.1",
    "zod": "^3.23.0"
  },
  "devDependencies": {
    "typescript": "^5.5.0",
    "vite": "^5.4.0",
    "@tauri-apps/cli": "^2.0.0",
    "@types/react": "^18.3.0",
    "@types/react-dom": "^18.3.0",
    "@types/three": "^0.168.0",
    "tailwindcss": "^3.4.0",
    "postcss": "^8.4.0",
    "autoprefixer": "^10.4.0"
  }
}
```

#### `tracecraft/tsconfig.json`
```json
{
  "compilerOptions": {
    "target": "ES2021",
    "useDefineForClassFields": true,
    "lib": ["ES2021", "DOM", "DOM.Iterable"],
    "module": "ESNext",
    "skipLibCheck": true,
    "moduleResolution": "bundler",
    "allowImportingTsExtensions": true,
    "resolveJsonModule": true,
    "isolatedModules": true,
    "noEmit": true,
    "jsx": "react-jsx",
    "strict": true,
    "noUnusedLocals": true,
    "noUnusedParameters": true,
    "noFallthroughCasesInSwitch": true,
    "forceConsistentCasingInFileNames": true
  },
  "include": ["src"],
  "references": [{ "path": "./tsconfig.node.json" }]
}
```

#### `tracecraft/tsconfig.node.json`
```json
{
  "compilerOptions": {
    "composite": true,
    "skipLibCheck": true,
    "module": "ESNext",
    "moduleResolution": "bundler",
    "allowSyntheticDefaultImports": true
  },
  "include": ["vite.config.ts"]
}
```

#### `tracecraft/vite.config.ts`
```typescript
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

const host = process.env.TAURI_DEV_HOST;

export default defineConfig(async () => ({
  plugins: [react()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? { protocol: "ws", host, port: 1421 }
      : undefined,
    watch: { ignored: ["**/src-tauri/**"] },
  },
}));
```

#### `tracecraft/tailwind.config.js`
```javascript
/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{js,ts,jsx,tsx}"],
  theme: {
    extend: {
      colors: {
        copper: { DEFAULT: "#D4783C", dark: "#A55A2A" },
        bg: { deep: "#1A1B2E", surface: "#232540", elevated: "#2D2F52", canvas: "#0F1020" },
        text: { primary: "#E8E9F3", secondary: "#9496B8", muted: "#5C5E7E" },
        border: { DEFAULT: "#343760", subtle: "#282A4A" },
        error: "#E6454A",
        warning: "#F0A030",
        success: "#42BE65",
        info: "#5096E0",
      },
    },
  },
  plugins: [],
};
```

#### `tracecraft/postcss.config.js`
```javascript
export default { plugins: { tailwindcss: {}, autoprefixer: {} } };
```

#### `tracecraft/index.html`
```html
<!doctype html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <link rel="icon" type="image/svg+xml" href="/favicon.svg" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>TraceCraft</title>
  </head>
  <body class="bg-bg-deep text-text-primary">
    <div id="root"></div>
    <script type="module" src="/src/main.tsx"></script>
  </body>
</html>
```

#### `tracecraft/.gitignore`
```
node_modules/
dist/
src-tauri/target/
*.log
.DS_Store
```

#### `tracecraft/src/vite-env.d.ts`
```typescript
/// <reference types="vite/client" />
```

#### `tracecraft/src/main.tsx`
```typescript
import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import "./styles/index.css";

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode><App /></React.StrictMode>
);
```

#### `tracecraft/src/App.tsx`
```typescript
import { AppShell } from "./components/layout/AppShell";

export default function App() {
  return <AppShell />;
}
```

#### `tracecraft/src/styles/index.css`
```css
@tailwind base;
@tailwind components;
@tailwind utilities;

body {
  margin: 0;
  overflow: hidden;
  font-family: "Inter", system-ui, -apple-system, sans-serif;
}

::-webkit-scrollbar { width: 6px; }
::-webkit-scrollbar-track { background: transparent; }
::-webkit-scrollbar-thumb { background: #343760; border-radius: 3px; }
```

#### `tracecraft/src-tauri/Cargo.toml`
```toml
[package]
name = "tracecraft"
version = "0.1.0"
edition = "2021"

[lib]
name = "tracecraft_lib"
crate-type = ["lib", "cdylib", "staticlib"]

[build-dependencies]
tauri-build = { version = "2", features = [] }

[dependencies]
tauri = { version = "2", features = [] }
tauri-plugin-dialog = "2"
tauri-plugin-fs = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
rusqlite = { version = "0.31", features = ["bundled"] }
uuid = { version = "1", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
thiserror = "1"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
quick-xml = "0.31"
csv = "1.3"
rust_xlsxwriter = "0.64"
parking_lot = "0.12"
```

#### `tracecraft/src-tauri/tauri.conf.json`
```json
{
  "$schema": "https://raw.githubusercontent.com/tauri-apps/tauri/dev/crates/tauri-cli/schema.json",
  "productName": "TraceCraft",
  "version": "0.1.0",
  "identifier": "com.tracecraft.app",
  "build": {
    "frontendDist": "../dist",
    "devUrl": "http://localhost:1420",
    "beforeDevCommand": "npm run dev",
    "beforeBuildCommand": "npm run build"
  },
  "app": {
    "windows": [
      {
        "title": "TraceCraft",
        "width": 1400,
        "height": 900,
        "minWidth": 1000,
        "minHeight": 700,
        "resizable": true
      }
    ],
    "security": {
      "csp": null
    }
  },
  "plugins": {
    "dialog": {},
    "fs": {
      "scope": ["**"]
    }
  }
}
```

#### `tracecraft/src-tauri/capabilities/default.json`
```json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "default",
  "description": "Default permissions",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "dialog:default",
    "dialog:allow-open",
    "dialog:allow-save",
    "fs:default",
    "fs:allow-read-text-file",
    "fs:allow-write-text-file",
    "fs:allow-exists"
  ]
}
```

#### `tracecraft/src-tauri/src/main.rs`
```rust
// Pre-annotations needed for Tauri
fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

#### `tracecraft/src-tauri/build.rs`
```rust
fn main() {
    tauri_build::build()
}
```

**Run:** `npm install && npm run tauri dev` — verify window opens with "TraceCraft" title.

---

### Step 2: Rust Data Model — Core Types

Create the model module with arena allocator, geometry types, and ID newtypes.

#### `src-tauri/src/model/id.rs`
```rust
use serde::{Deserialize, Serialize};
use std::fmt;

macro_rules! id_type {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
        pub struct $name(pub usize);
        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}({})", stringify!($name), self.0)
            }
        }
    };
}

id_type!(SheetId);
id_type!(SymbolId);
id_type!(WireId);
id_type!(JunctionId);
id_type!(NetLabelId);
id_type!(BusId);
id_type!(TextAnnotationId);
id_type!(FootprintId);
id_type!(TraceId);
id_type!(ViaId);
id_type!(ZoneId);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ComponentId(pub uuid::Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LayerId(pub u8);

impl std::ops::Add<u8> for LayerId {
    type Output = Self;
    fn add(self, rhs: u8) -> Self { LayerId(self.0 + rhs) }
}
```

#### `src-tauri/src/model/geometry.rs`
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub fn new(x: f64, y: f64) -> Self { Self { x, y } }
    pub fn distance(&self, other: &Point) -> f64 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }
    pub fn rotate(&self, center: &Point, angle_deg: f64) -> Point {
        let rad = angle_deg.to_radians();
        let cos = rad.cos();
        let sin = rad.sin();
        let dx = self.x - center.x;
        let dy = self.y - center.y;
        Point::new(
            center.x + dx * cos - dy * sin,
            center.y + dx * sin + dy * cos,
        )
    }
    pub fn midpoint(&self, other: &Point) -> Point {
        Point::new((self.x + other.x) / 2.0, (self.y + other.y) / 2.0)
    }
    pub fn snap(&self, grid: f64) -> Point {
        Point::new(
            (self.x / grid).round() * grid,
            (self.y / grid).round() * grid,
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Size { pub width: f64, pub height: f64 }

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Rect { pub x: f64, pub y: f64, pub width: f64, pub height: f64 }

impl Rect {
    pub fn contains(&self, point: &Point) -> bool {
        point.x >= self.x && point.x <= self.x + self.width
            && point.y >= self.y && point.y <= self.y + self.height
    }
    pub fn from_points(points: &[Point]) -> Option<Rect> {
        if points.is_empty() { return None; }
        let min_x = points.iter().map(|p| p.x).fold(f64::INFINITY, f64::min);
        let max_x = points.iter().map(|p| p.x).fold(f64::NEG_INFINITY, f64::max);
        let min_y = points.iter().map(|p| p.y).fold(f64::INFINITY, f64::min);
        let max_y = points.iter().map(|p| p.y).fold(f64::NEG_INFINITY, f64::max);
        Some(Rect { x: min_x, y: min_y, width: max_x - min_x, height: max_y - min_y })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Rotation(pub f64); // degrees, clockwise
```

#### `src-tauri/src/model/arena.rs`
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Arena<T> {
    items: Vec<Option<T>>,
    free_list: Vec<usize>,
}

impl<T> Arena<T> {
    pub fn new() -> Self {
        Self { items: Vec::new(), free_list: Vec::new() }
    }

    pub fn insert(&mut self, item: T) -> usize {
        if let Some(index) = self.free_list.pop() {
            self.items[index] = Some(item);
            index
        } else {
            let index = self.items.len();
            self.items.push(Some(item));
            index
        }
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        self.items.get(index).and_then(|i| i.as_ref())
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.items.get_mut(index).and_then(|i| i.as_mut())
    }

    pub fn remove(&mut self, index: usize) -> Option<T> {
        let item = self.items.get_mut(index)?.take();
        if item.is_some() {
            self.free_list.push(index);
        }
        item
    }

    pub fn iter(&self) -> impl Iterator<Item = (usize, &T)> {
        self.items.iter().enumerate().filter_map(|(i, item)| item.as_ref().map(|t| (i, t)))
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (usize, &mut T)> {
        self.items.iter_mut().enumerate().filter_map(|(i, item)| item.as_mut().map(|t| (i, t)))
    }

    pub fn len(&self) -> usize {
        self.items.len() - self.free_list.len()
    }

    pub fn is_empty(&self) -> bool { self.len() == 0 }

    pub fn clear(&mut self) {
        self.items.clear();
        self.free_list.clear();
    }

    pub fn snapshot(&self) -> Vec<(usize, &T)> {
        self.iter().collect()
    }
}

impl<T> Default for Arena<T> {
    fn default() -> Self { Self::new() }
}
```

#### `src-tauri/src/model/project.rs`
```rust
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: Uuid,
    pub name: String,
    pub path: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectSummary {
    pub id: Uuid,
    pub name: String,
    pub path: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentProject {
    pub name: String,
    pub path: String,
    pub last_opened: String,
}
```

#### `src-tauri/src/model/schematic.rs`
```rust
use serde::{Deserialize, Serialize};
use super::arena::Arena;
use super::geometry::*;
use super::id::*;
use super::library::ComponentId;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolInstance {
    pub component_id: ComponentId,
    pub position: Point,
    pub rotation: Rotation,
    pub mirrored: bool,
    pub refdes: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wire {
    pub points: Vec<Point>,
    pub net_id: Option<NetId>,
    pub width: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetLabel {
    pub name: String,
    pub position: Point,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Junction {
    pub position: Point,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bus {
    pub name: String,
    pub members: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextAnnotation {
    pub text: String,
    pub position: Point,
    pub size: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HierarchicalSheet {
    pub id: SheetId,
    pub name: String,
    pub child_sheet_id: SheetId,
    pub pins: Vec<HierarchicalPin>,
    pub position: Point,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HierarchicalPin {
    pub name: String,
    pub pin_number: String,
    pub position: Point,
}
```

#### `src-tauri/src/model/pcb.rs`
```rust
use serde::{Deserialize, Serialize};
use super::arena::Arena;
use super::geometry::*;
use super::id::*;
use super::library::ComponentId;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PcbBoard {
    pub footprints: Arena<FootprintInstance>,
    pub traces: Arena<Trace>,
    pub vias: Arena<Via>,
    pub zones: Arena<Zone>,
    pub outline: Option<BoardOutline>,
    pub layers: LayerStack,
    pub design_rules: DesignRules,
}

impl Default for PcbBoard {
    fn default() -> Self {
        Self {
            footprints: Arena::new(),
            traces: Arena::new(),
            vias: Arena::new(),
            zones: Arena::new(),
            outline: None,
            layers: LayerStack::default_2layer(),
            design_rules: DesignRules::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FootprintInstance {
    pub component_id: ComponentId,
    pub position: Point,
    pub rotation: Rotation,
    pub mirrored: bool,
    pub refdes: String,
    pub locked: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trace {
    pub net_id: Option<NetId>,
    pub layer: LayerId,
    pub width: f64,
    pub points: Vec<Point>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Via {
    pub position: Point,
    pub start_layer: LayerId,
    pub end_layer: LayerId,
    pub diameter: f64,
    pub drill: f64,
    pub net_id: Option<NetId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Zone {
    pub net_id: Option<NetId>,
    pub layer: LayerId,
    pub outline: Vec<Point>,
    pub clearance: f64,
    pub thermal_relief: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoardOutline {
    pub shape: Vec<Vec<Point>>,
    pub cutouts: Vec<Vec<Point>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerStack {
    pub layers: Vec<LayerDef>,
}

impl LayerStack {
    pub fn default_2layer() -> Self {
        Self {
            layers: vec![
                LayerDef { id: LayerId(1), name: "Top".into(), kind: LayerKind::Top, color: "#D47A2F".into() },
                LayerDef { id: LayerId(2), name: "Bottom".into(), kind: LayerKind::Bottom, color: "#A0793F".into() },
                LayerDef { id: LayerId(10), name: "Top Solder Mask".into(), kind: LayerKind::SolderMask, color: "#30853A".into() },
                LayerDef { id: LayerId(11), name: "Bottom Solder Mask".into(), kind: LayerKind::SolderMask, color: "#25682D".into() },
                LayerDef { id: LayerId(20), name: "Top Silkscreen".into(), kind: LayerKind::Silkscreen, color: "#F0F0F0".into() },
                LayerDef { id: LayerId(21), name: "Bottom Silkscreen".into(), kind: LayerKind::Silkscreen, color: "#C0C0C0".into() },
            ],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerDef {
    pub id: LayerId,
    pub name: String,
    pub kind: LayerKind,
    pub color: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LayerKind {
    Top,
    Bottom,
    Inner,
    SolderMask,
    Silkscreen,
    Paste,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesignRules {
    pub min_trace_width: f64,
    pub min_clearance: f64,
    pub min_annular_ring: f64,
    pub min_hole_size: f64,
    pub default_trace_width: f64,
    pub default_via_diameter: f64,
    pub default_via_drill: f64,
}

impl Default for DesignRules {
    fn default() -> Self {
        Self {
            min_trace_width: 0.15,
            min_clearance: 0.15,
            min_annular_ring: 0.15,
            min_hole_size: 0.3,
            default_trace_width: 0.25,
            default_via_diameter: 0.6,
            default_via_drill: 0.3,
        }
    }
}
```

#### `src-tauri/src/model/library.rs`
```rust
use serde::{Deserialize, Serialize};
use super::geometry::*;
use super::id::*;
use super::pcb::LayerId;

pub use super::id::ComponentId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Component {
    pub id: ComponentId,
    pub name: String,
    pub description: String,
    pub manufacturer: Option<String>,
    pub category: Option<String>,
    pub symbol: SymbolDef,
    pub footprint: FootprintDef,
    pub model_3d: Option<Model3D>,
    pub parameters: std::collections::HashMap<String, String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolDef {
    pub pins: Vec<PinDef>,
    pub body: Vec<SymbolGraphics>,
    pub origin: Point,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PinDef {
    pub name: String,
    pub number: String,
    pub position: Point,
    pub length: f64,
    pub electrical_type: PinType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PinType {
    Input, Output, Bidirectional, Power, Gnd, Passive, OpenCollector, OpenEmitter, NotConnected,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SymbolGraphics {
    Rectangle { position: Point, size: Size, fill: String, stroke: String },
    Circle { center: Point, radius: f64, fill: String, stroke: String },
    Arc { center: Point, radius: f64, start_angle: f64, end_angle: f64, stroke: String },
    Line { start: Point, end: Point, stroke: String, width: f64 },
    Text { text: String, position: Point, size: f64 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FootprintDef {
    pub pads: Vec<PadDef>,
    pub outline: Vec<Point>,
    pub courtyard: Vec<Point>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PadDef {
    pub number: String,
    pub shape: PadShape,
    pub position: Point,
    pub size: Size,
    pub layer: LayerId,
    pub drill: Option<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum PadShape {
    Round,
    Rect,
    Oval,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Model3D {
    pub path: String,
    pub offset: Point,
    pub rotation: Rotation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentSummary {
    pub id: ComponentId,
    pub name: String,
    pub description: String,
    pub category: Option<String>,
    pub manufacturer: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    pub id: String,
    pub name: String,
    pub parent_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    pub text: String,
    pub category: Option<String>,
    pub page: usize,
    pub per_page: usize,
}

impl Default for SearchQuery {
    fn default() -> Self {
        Self { text: String::new(), category: None, page: 0, per_page: 20 }
    }
}

#[async_trait::async_trait]
pub trait ComponentRepository: Send + Sync {
    async fn search(&self, query: &SearchQuery) -> Result<Vec<ComponentSummary>, String>;
    async fn get_by_id(&self, id: ComponentId) -> Result<Option<Component>, String>;
    async fn save(&self, component: &Component) -> Result<(), String>;
    async fn delete(&self, id: ComponentId) -> Result<(), String>;
    async fn get_categories(&self) -> Result<Vec<Category>, String>;
}
```

#### `src-tauri/src/model/netlist.rs`
```rust
use serde::{Deserialize, Serialize};
use super::id::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Netlist {
    pub nets: Vec<Net>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Net {
    pub id: NetId,
    pub name: String,
    pub connections: Vec<PinRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PinRef {
    pub refdes: String,
    pub pin_number: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetlistDiff {
    pub added: Vec<Net>,
    pub removed: Vec<Net>,
    pub modified: Vec<(String, Net, Net)>,
}
```

#### `src-tauri/src/model/mod.rs`
```rust
pub mod id;
pub mod geometry;
pub mod arena;
pub mod project;
pub mod schematic;
pub mod pcb;
pub mod library;
pub mod netlist;
```

---

### Step 3: SQLite Database Layer

#### `src-tauri/src/db/migrations.rs`
```rust
use rusqlite::Connection;

const MIGRATIONS: &[&str] = &[
    "CREATE TABLE IF NOT EXISTS components (
        id TEXT PRIMARY KEY,
        name TEXT NOT NULL,
        description TEXT NOT NULL DEFAULT '',
        manufacturer TEXT,
        category TEXT,
        symbol_json TEXT NOT NULL DEFAULT '{}',
        footprint_json TEXT NOT NULL DEFAULT '{}',
        model_3d_json TEXT,
        parameters_json TEXT NOT NULL DEFAULT '{}',
        created_at TEXT NOT NULL,
        updated_at TEXT NOT NULL
    );
    CREATE TABLE IF NOT EXISTS categories (
        id TEXT PRIMARY KEY,
        name TEXT NOT NULL UNIQUE,
        parent_id TEXT REFERENCES categories(id)
    );
    CREATE VIRTUAL TABLE IF NOT EXISTS components_fts USING fts5(
        name, description, manufacturer, category,
        content='components',
        content_rowid='rowid'
    );
    INSERT INTO categories (id, name) VALUES
        ('resistor', 'Resistors'),
        ('capacitor', 'Capacitors'),
        ('inductor', 'Inductors'),
        ('diode', 'Diodes'),
        ('transistor', 'Transistors'),
        ('ic', 'ICs'),
        ('connector', 'Connectors'),
        ('power', 'Power'),
        ('sensor', 'Sensors'),
        ('electromechanical', 'Electromechanical'),
        ('misc', 'Miscellaneous')
    ON CONFLICT(name) DO NOTHING;",
];

pub fn run_migrations(conn: &Connection) -> Result<(), String> {
    let version: i32 = conn
        .pragma_query_value(None, "user_version", |r| r.get(0))
        .map_err(|e| e.to_string())?;

    for (i, migration) in MIGRATIONS.iter().enumerate() {
        let task_version = (i + 1) as i32;
        if version < task_version {
            conn.execute_batch(migration).map_err(|e| e.to_string())?;
            conn.pragma_update(None, "user_version", task_version)
                .map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}
```

#### `src-tauri/src/db/library_repo.rs`
```rust
use rusqlite::{params, Connection};
use std::sync::Mutex;
use super::super::model::library::*;

pub struct SqliteComponentRepository {
    conn: Mutex<Connection>,
}

impl SqliteComponentRepository {
    pub fn new(conn: Connection) -> Self {
        Self { conn: Mutex::new(conn) }
    }
}

#[async_trait::async_trait]
impl ComponentRepository for SqliteComponentRepository {
    async fn search(&self, query: &SearchQuery) -> Result<Vec<ComponentSummary>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let pattern = format!("%{}%", query.text);
        let offset = query.page * query.per_page;

        let mut stmt = if query.text.is_empty() {
            conn.prepare(
                "SELECT id, name, description, category, manufacturer
                 FROM components
                 WHERE (?1 IS NULL OR category = ?1)
                 ORDER BY name LIMIT ?2 OFFSET ?3"
            ).map_err(|e| e.to_string())?
        } else {
            conn.prepare(
                "SELECT c.id, c.name, c.description, c.category, c.manufacturer
                 FROM components c
                 JOIN components_fts fts ON c.rowid = fts.rowid
                 WHERE components_fts MATCH ?1
                 AND (?2 IS NULL OR c.category = ?2)
                 ORDER BY rank LIMIT ?3 OFFSET ?4"
            ).map_err(|e| e.to_string())?
        };

        let rows = if query.text.is_empty() {
            stmt.query_map(
                params![query.category, query.per_page as i64, offset as i64],
                |row| Ok(ComponentSummary {
                    id: ComponentId(uuid::Uuid::parse_str(&row.get::<_, String>(0)?).map_err(|e| e.to_string())?),
                    name: row.get(1)?,
                    description: row.get(2)?,
                    category: row.get(3)?,
                    manufacturer: row.get(4)?,
                }),
            ).map_err(|e| e.to_string())?
        } else {
            stmt.query_map(
                params![pattern, query.category, query.per_page as i64, offset as i64],
                |row| Ok(ComponentSummary {
                    id: ComponentId(uuid::Uuid::parse_str(&row.get::<_, String>(0)?).map_err(|e| e.to_string())?),
                    name: row.get(1)?,
                    description: row.get(2)?,
                    category: row.get(3)?,
                    manufacturer: row.get(4)?,
                }),
            ).map_err(|e| e.to_string())?
        };

        rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
    }

    async fn get_by_id(&self, id: ComponentId) -> Result<Option<Component>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn.prepare(
            "SELECT id, name, description, manufacturer, category,
                    symbol_json, footprint_json, model_3d_json,
                    parameters_json, created_at, updated_at
             FROM components WHERE id = ?1"
        ).map_err(|e| e.to_string())?;

        let mut rows = stmt.query_map(params![id.0.to_string()], |row| {
            Ok(Component {
                id: ComponentId(uuid::Uuid::parse_str(&row.get::<_, String>(0)?).unwrap()),
                name: row.get(1)?,
                description: row.get(2)?,
                manufacturer: row.get(3)?,
                category: row.get(4)?,
                symbol: serde_json::from_str(&row.get::<_, String>(5)?).unwrap_or_default(),
                footprint: serde_json::from_str(&row.get::<_, String>(6)?).unwrap_or_default(),
                model_3d: row.get::<_, Option<String>>(7)?.and_then(|s| serde_json::from_str(&s).ok()),
                parameters: serde_json::from_str(&row.get::<_, String>(8)?).unwrap_or_default(),
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
            })
        }).map_err(|e| e.to_string())?;

        rows.next().transpose().map_err(|e| e.to_string())
    }

    async fn save(&self, component: &Component) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let now = chrono::Utc::now().to_rfc3339();

        conn.execute(
            "INSERT INTO components (id, name, description, manufacturer, category,
             symbol_json, footprint_json, model_3d_json, parameters_json, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
             ON CONFLICT(id) DO UPDATE SET
             name=excluded.name, description=excluded.description,
             manufacturer=excluded.manufacturer, category=excluded.category,
             symbol_json=excluded.symbol_json, footprint_json=excluded.footprint_json,
             model_3d_json=excluded.model_3d_json, parameters_json=excluded.parameters_json,
             updated_at=excluded.updated_at",
            params![
                component.id.0.to_string(),
                component.name,
                component.description,
                component.manufacturer,
                component.category,
                serde_json::to_string(&component.symbol).unwrap_or_default(),
                serde_json::to_string(&component.footprint).unwrap_or_default(),
                component.model_3d.as_ref().map(|m| serde_json::to_string(m).unwrap_or_default()),
                serde_json::to_string(&component.parameters).unwrap_or_default(),
                now, now,
            ],
        ).map_err(|e| e.to_string())?;

        Ok(())
    }

    async fn delete(&self, id: ComponentId) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute("DELETE FROM components WHERE id = ?1", params![id.0.to_string()])
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    async fn get_categories(&self) -> Result<Vec<Category>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn.prepare("SELECT id, name, parent_id FROM categories ORDER BY name")
            .map_err(|e| e.to_string())?;
        let rows = stmt.query_map([], |row| {
            Ok(Category {
                id: row.get(0)?,
                name: row.get(1)?,
                parent_id: row.get(2)?,
            })
        }).map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
    }
}
```

#### `src-tauri/src/db/project_store.rs`
```rust
use rusqlite::{params, Connection};
use super::super::model::project::*;
use uuid::Uuid;

pub fn save_project(conn: &Connection, project: &Project) -> Result<(), String> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS project_meta (
            id TEXT PRIMARY KEY, name TEXT NOT NULL,
            created_at TEXT NOT NULL, updated_at TEXT NOT NULL
        )",
        [],
    ).map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT OR REPLACE INTO project_meta (id, name, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4)",
        params![project.id.to_string(), project.name, project.created_at, project.updated_at],
    ).map_err(|e| e.to_string())?;

    Ok(())
}

pub fn load_project(conn: &Connection, id: Uuid) -> Result<Option<Project>, String> {
    let mut stmt = conn.prepare(
        "SELECT id, name, '', created_at, updated_at FROM project_meta WHERE id = ?1"
    ).map_err(|e| e.to_string())?;

    let mut rows = stmt.query_map(params![id.to_string()], |row| {
        Ok(Project {
            id: uuid::Uuid::parse_str(&row.get::<_, String>(0)?).unwrap(),
            name: row.get(1)?,
            path: String::new(),
            created_at: row.get(3)?,
            updated_at: row.get(4)?,
        })
    }).map_err(|e| e.to_string())?;

    rows.next().transpose().map_err(|e| e.to_string())
}
```

#### `src-tauri/src/db/mod.rs`
```rust
pub mod migrations;
pub mod library_repo;
pub mod project_store;
```

---

### Step 4: App State & Undo System

#### `src-tauri/src/app_state.rs`
```rust
use std::sync::Arc;
use parking_lot::RwLock;
use crate::model::project::*;
use crate::model::schematic::*;
use crate::model::pcb::*;
use crate::model::library::*;
use crate::undo::command::CommandHistory;
use rusqlite::Connection;
use tauri::AppHandle;

pub struct AppState {
    pub project: RwLock<Option<Project>>,
    pub board: RwLock<PcbBoard>,
    pub schematics: RwLock<std::collections::HashMap<SheetId, SchematicSheet>>,
    pub command_history: RwLock<CommandHistory>,
    pub library: Arc<dyn ComponentRepository>,
    pub settings: RwLock<Settings>,
    pub db: std::sync::Mutex<Connection>,
    pub app_handle: RwLock<Option<AppHandle>>,
}

impl AppState {
    pub fn new(db: Connection, library: Arc<dyn ComponentRepository>) -> Self {
        Self {
            project: RwLock::new(None),
            board: RwLock::new(PcbBoard::default()),
            schematics: RwLock::new(std::collections::HashMap::new()),
            command_history: RwLock::new(CommandHistory::new(1000)),
            library,
            settings: RwLock::new(Settings::default()),
            db: std::sync::Mutex::new(db),
            app_handle: RwLock::new(None),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Settings {
    pub grid_size: f64,
    pub snap_to_grid: bool,
    pub dark_mode: bool,
    pub recent_files: Vec<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            grid_size: 2.54, // 0.1 inch in mm
            snap_to_grid: true,
            dark_mode: true,
            recent_files: Vec::new(),
        }
    }
}
```

#### `src-tauri/src/undo/command.rs`
```rust
use crate::model::pcb::*;
use crate::model::schematic::*;
use crate::model::id::*;
use crate::model::geometry::*;
use crate::model::library::ComponentId;

pub trait Command: Send {
    fn execute(&mut self, board: &mut PcbBoard) -> Result<(), String>;
    fn undo(&mut self, board: &mut PcbBoard) -> Result<(), String>;
    fn name(&self) -> &str;
}

pub struct CommandHistory {
    executed: Vec<Box<dyn Command>>,
    undone: Vec<Box<dyn Command>>,
    max: usize,
}

impl CommandHistory {
    pub fn new(max: usize) -> Self {
        Self { executed: Vec::new(), undone: Vec::new(), max }
    }

    pub fn execute(&mut self, mut cmd: Box<dyn Command>, board: &mut PcbBoard) -> Result<(), String> {
        cmd.execute(board)?;
        self.executed.push(cmd);
        self.undone.clear();
        if self.executed.len() > self.max {
            self.executed.remove(0);
        }
        Ok(())
    }

    pub fn undo(&mut self, board: &mut PcbBoard) -> Result<(), String> {
        if let Some(mut cmd) = self.executed.pop() {
            cmd.undo(board)?;
            self.undone.push(cmd);
        }
        Ok(())
    }

    pub fn redo(&mut self, board: &mut PcbBoard) -> Result<(), String> {
        if let Some(mut cmd) = self.undone.pop() {
            cmd.execute(board)?;
            self.executed.push(cmd);
        }
        Ok(())
    }

    pub fn can_undo(&self) -> bool { !self.executed.is_empty() }
    pub fn can_redo(&self) -> bool { !self.undone.is_empty() }
}

// Concrete Commands
pub struct PlaceFootprint {
    pub component_id: ComponentId,
    pub position: Point,
    pub rotation: Rotation,
    pub result_id: Option<FootprintId>,
}

impl Command for PlaceFootprint {
    fn execute(&mut self, board: &mut PcbBoard) -> Result<(), String> {
        let id = board.footprints.insert(FootprintInstance {
            component_id: self.component_id,
            position: self.position,
            rotation: self.rotation,
            mirrored: false,
            refdes: String::new(),
            locked: false,
        });
        self.result_id = Some(FootprintId(id));
        Ok(())
    }

    fn undo(&mut self, board: &mut PcbBoard) -> Result<(), String> {
        if let Some(id) = self.result_id {
            board.footprints.remove(id.0);
        }
        Ok(())
    }

    fn name(&self) -> &str { "Place Footprint" }
}
```

#### `src-tauri/src/undo/mod.rs`
```rust
pub mod command;
```

---

### Step 5: IPC Commands

#### `src-tauri/src/commands/project.rs`
```rust
use tauri::State;
use crate::app_state::AppState;
use crate::model::project::*;

#[tauri::command]
pub fn create_project(state: State<AppState>, name: String) -> Result<ProjectSummary, String> {
    let id = uuid::Uuid::new_v4();
    let now = chrono::Utc::now().to_rfc3339();
    let project = Project {
        id,
        name: name.clone(),
        path: String::new(),
        created_at: now.clone(),
        updated_at: now,
    };
    *state.project.write() = Some(project.clone());
    Ok(ProjectSummary {
        id, name,
        path: String::new(),
        updated_at: now,
    })
}

#[tauri::command]
pub fn save_project(state: State<AppState>, path: String) -> Result<(), String> {
    let project = state.project.read().clone().ok_or("No project open")?;
    let db = state.db.lock().map_err(|e| e.to_string())?;
    crate::db::project_store::save_project(&db, &project)?;
    *state.project.write() = Some(Project { path: path.clone(), ..project });
    Ok(())
}
```

#### `src-tauri/src/commands/mod.rs`
```rust
pub mod project;
pub mod schematic;
pub mod pcb;
pub mod library;
pub mod routing;
pub mod drc;
pub mod ai;
pub mod viewport;
```

---

### Step 6: Frontend Layout Shell

Build the React layout components. Use Tailwind classes consistently. The
copper/dark color scheme from the brand guide applies throughout.

#### Layout structure:

```
AppShell (grid h-screen)
├── Toolbar (h-12, bg-bg-surface, border-b border-border)
│   ├── ToolButton per tool (select, wire, component, route, zone, 3D)
│   ├── Separator
│   ├── undo/redo ToolButtons
│   └── Separator + zoom controls
├── Main area (flex flex-1 overflow-hidden)
│   ├── Canvas area (flex-1 relative)
│   │   └── Active editor component (SchematicCanvas | PcbCanvas | Preview3D)
│   └── Sidebar (w-80, bg-bg-surface, border-l border-border)
│       └── Tabbed panels: Properties | Library | Layers | AI
└── StatusBar (h-6, bg-bg-surface, border-t border-border, text-text-muted text-xs)
```

#### Store the active mode in uiStore (schematic | pcb | preview).

#### `src/stores/uiStore.ts`
```typescript
import { create } from "zustand";

export type AppMode = "schematic" | "pcb" | "preview";
export type ToolType = "select" | "wire" | "component" | "route" | "zone" | "measure" | "pan";

interface Point { x: number; y: number; }

interface UIState {
  mode: AppMode;
  activeTool: ToolType;
  zoom: number;
  panOffset: Point;
  sidebarTab: "properties" | "library" | "layers" | "ai";
  sidebarOpen: boolean;
  setMode: (mode: AppMode) => void;
  setTool: (tool: ToolType) => void;
  setZoom: (zoom: number) => void;
  setPan: (offset: Point) => void;
  setSidebarTab: (tab: UIState["sidebarTab"]) => void;
  toggleSidebar: () => void;
}

export const useUIStore = create<UIState>((set) => ({
  mode: "schematic",
  activeTool: "select",
  zoom: 1,
  panOffset: { x: 0, y: 0 },
  sidebarTab: "library",
  sidebarOpen: true,
  setMode: (mode) => set({ mode }),
  setTool: (tool) => set({ activeTool: tool }),
  setZoom: (zoom) => set({ zoom: Math.max(0.1, Math.min(100, zoom)) }),
  setPan: (offset) => set({ panOffset: offset }),
  setSidebarTab: (tab) => set({ sidebarTab: tab }),
  toggleSidebar: () => set((s) => ({ sidebarOpen: !s.sidebarOpen })),
}));
```

#### `src/hooks/useTauriCommand.ts`
```typescript
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
        return await invoke<TResult>(command, args as any);
      } catch (e) {
        const msg = typeof e === "string" ? e : String(e);
        setError(msg);
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

#### `src/hooks/useTauriEvent.ts`
```typescript
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { useEffect } from "react";

export function useTauriEvent<T>(
  event: string,
  handler: (payload: T) => void,
  deps: any[] = []
) {
  useEffect(() => {
    let unlisten: UnlistenFn | undefined;
    listen<T>(event, (e) => handler(e.payload)).then((fn) => {
      unlisten = fn;
    });
    return () => {
      unlisten?.();
    };
  }, deps);
}
```

Build all remaining React components following the exact structure in the directory
tree above. Each store wraps the corresponding Tauri IPC commands. Each canvas
component uses the shared `KonvaCanvas` with react-konva.

---

### Step 7: Schematic Editor

Create `SchematicCanvas` using `KonvaCanvas`. It:
- Calls `get_schematic` IPC on mount to load sheet data
- Renders symbols via `SymbolRenderer` (body shapes + pins)
- Renders wires as lines with junction dots at intersections
- Handles tool-specific interactions (wire tool, select tool, component placement)
- Uses `schematicStore` for state

#### Key interaction patterns:

**Wire drawing:**
- User selects "wire" tool → clicks on a pin → wire starts
- Each click adds a vertex (snapped to 45° angles)
- Click on another pin → wire completes → invokes `draw_wire` IPC
- Esc cancels current wire

**Symbol placement:**
- Drag from LibraryBrowser → drop on canvas
- On drop: invoke `place_symbol` IPC → update store

---

### Step 8: PCB Layout Editor

Create `PcbCanvas` using `KonvaCanvas`. It:
- Calls `get_board_state` IPC on mount to load board
- Renders footprints via `FootprintRenderer` (pads colored by layer)
- Renders traces, vias, zones
- Shows ratsnest lines via `RatsnestRenderer`
- Shows DRC violations via `DRCOverlay`
- Layer visibility controlled by `LayerPanel`

#### Trace routing interaction:
- Select "route" tool → click on a pad → starts trace
- Click adds vertices (45° snap)
- `V` key inserts via at cursor position
- Double-click on target pad completes trace
- Invokes `route_trace` IPC → updates store

---

### Step 9: 3D Preview

Create `Preview3D` using `@react-three/fiber`. It:
- Renders the board substrate as a flat box geometry
- Extrudes traces as thin 3D shapes on the substrate
- Loads 3D models at footprint positions
- Orbit controls for navigation
- Layer visibility toggles

---

### Step 10: AI Copilot

Create the AI module stubs. The actual ONNX/llama.cpp integration requires
downloading models. For the MVP:
- `AIPanel` shows the UI shell
- `RecommenderWidget` shows "AI recommendations (model not loaded)" placeholder
- `ChatWindow` has the chat UI but shows "local LLM not available"
- Backend AI commands return "AI model not loaded" errors gracefully

Full AI integration is post-MVP but the entire UI and IPC surface should be built
now so wiring it up later is seamless.

---

### Step 11: File I/O

Implement the IO module:

- **Native (.tcproj):** Project saved as SQLite database file with all state.
  The `save_project` command writes current board + schematics to DB. `open_project`
  reads them back.
- **KiCad import/export:** Parse/generate s-expression format. Focus on KiCad 7+
  format since it's the most stable.
- **Gerber export:** Generate RS-274X files. One file per layer +
  NC drill file.
- **IPC-2581:** XML-based export.
- **BOM:** CSV and XLSX columns: RefDes, Value, Footprint, Manufacturer, Part #, Qty.

---

### Step 12: Polish

Configure `tauri.conf.json` with app icon path. Set up tracing/logging in
`main.rs`. Add dark scrollbar styling. Ensure all edge cases (empty board,
no project open, etc.) show appropriate UI states.

---

## Key Design Decisions

1. **Arena<T> for all elements** — Never use `Vec<T>` directly for board elements.
   Arena gives stable IDs for undo/redo and avoids borrow issues.

2. **All mutations go through CommandHistory** — Never mutate PcbBoard directly in
   command handlers. Always create a Command, execute through history.

3. **Frontend is a thin rendering layer** — All data processing happens in Rust.
   The frontend renders what the backend sends and sends user actions back as
   commands.

4. **Events for cross-cutting concerns** — Netlist changes and DRC results flow
   via Tauri events, not polling.

5. **Graceful AI degradation** — If models aren't installed, AI features show
   a "download model" prompt. Never crash.

---

## Verification

After completing all steps:
1. `npm run tauri dev` launches the app
2. Create a new project
3. Browse the component library
4. Place a symbol on the schematic
5. Draw a wire between pins
6. Run ERC — no violations for connected nets
7. Switch to PCB view
8. Place footprints
9. Route a trace between two pads
10. Run DRC
11. Export Gerbers
12. Undo all actions — state returns to blank board
