# TraceCraft — Specification

## Overview

**TraceCraft** is a local-first, AI-assisted PCB design desktop application inspired by
Flux.ai. It provides schematic capture, PCB layout editing, auto-routing, design rule
checking (DRC), 3D preview, and a local component library — all running natively via
Tauri with a React frontend and Rust backend.

Unlike cloud-based PCB tools, TraceCraft stores designs and component libraries locally.
No accounts, no subscriptions, and no design data leaves your machine. AI assistance in
this build is implemented as local keyword/template heuristics (no models are downloaded).

## Target Audience

- **Primary:** Hobbyist electronics engineers, PCB designers, and makers building
  custom circuits
- **Secondary:** Professional engineers who want a fast, local alternative to cloud
  tools for early-stage prototyping
- **Tertiary:** Educators teaching PCB design fundamentals

The tool is designed for a single user working on one project at a time.

## Core Features

### 1. Schematic Editor

| Feature | Description |
|---------|-------------|
| Drag-drop components | Place symbols from library onto schematic canvas |
| Wire drawing | Click-to-connect wires with snap-to-pin, auto-junction |
| Net labels | Named nets for clean hierarchical wiring |
| Bus support | Group related nets into buses |
| Hierarchical sheets | Sub-schematics as reusable blocks |
| Annotation | Text labels, notes, dimensions on schematic |
| ERC (Electrical Rule Check) | Unconnected pins, shorted outputs, floating nets |

### 2. PCB Layout Editor

| Feature | Description |
|---------|-------------|
| Board outline editor | Draw/import board shape with cutouts |
| Component placement | Drag, rotate, snap to grid, density heatmap |
| Manual trace routing | Click-to-route with push/shove, via placement |
| Differential pairs | Length-matched pair routing |
| Copper pours / planes | Polygon fills with thermal relief |
| Design Rule Check (DRC) | Clearance, width, annular ring, soldermask checks |
| Silkscreen & fab layers | Top/bottom overlay, solder paste, solder mask |
| Netlist synchronization | Primary schematic → PCB netlist sync; PCB → schematic sync is partial |

### 3. Auto-Routing

| Feature | Description |
|---------|-------------|
| A\*-based pathfinding | Grid-based router with cost heuristics |
| Multi-layer routing | Layer assignment with via insertion |
| User-configurable rules | Preferred directions per layer, via cost, clearance |
| Fan-out | Automated BGA/QFP fan-out patterns |
| Batch routing | Route selected nets or entire board |

**Current status:** the router is preview-only. No UI control invokes it, results are not
persisted to the board, and layer transitions do not produce vias. See Known Limitations.

### 4. Component Library

| Feature | Description |
|---------|-------------|
| Local SQLite database | Symbols, footprints, 3D models, metadata |
| Component search | Search by name, manufacturer, category, parameters |
| Symbol editor | Create and edit schematic symbols |
| Footprint editor | Create and edit PCB footprints (land patterns) |
| 3D model attachment | Associate STL, OBJ, GLTF, or GLB models with footprints |
| Library import | KiCad symbol/footprint libraries (.kicad_sym, .pretty) and CSV BOM files |
| Custom categories | User-defined tagging and categorization |

### 5. File I/O
 
| Format | Direction | Notes |
|--------|-----------|-------|
| TraceCraft native (.tcproj) | Import/Export | JSON single-file project (metadata, board, schematics) |
| KiCad schematic (.kicad_sch) | Import/Export | Import maps symbols, wires and labels; export is minimal and not a full round-trip |
| KiCad PCB (.kicad_pcb) | Import/Export | Import reads tracks and footprints; export writes tracks, outline and footprint reference/value (no pads or stackup) |
| KiCad library (.kicad_sym, .pretty) | Import | Symbol and footprint ingestion; official KiCad libraries can be downloaded in-app |
| IPC-2581 | Export | Basic XML output (layer stack, nets, traces); not schema-validated |
| Gerber RS-274X | Export | Single file with copper traces and zone outlines only; no pads, vias, mask, silkscreen or drill file |
| BOM / CPL | Export | CSV BOM (reference, value, component ID, quantity) and CSV placement file |
| SPICE (.cir) | Export/Run | Netlist export plus ngspice-backed transient, AC, DC and operating-point analysis |
| **STEP (.step/.stp)** | **Export** | **AP203 `MANIFOLD_SOLID_BREP` (OCCT-validated); board cutouts are not exported** |
 
### 6. 3D Preview
 
| Feature | Description |
|---------|-------------|
| Real-time 3D rendering | WebGL-powered board visualization |
| Component bodies | 3D models attached to footprints (STL, OBJ, GLTF, GLB) |
| Layer visibility toggle | Show/hide copper, solder mask, silkscreen |
| Orbit/pan/zoom | Standard 3D viewport controls |
| Measurement tool | Distance measurement between two 3D points |
| **STEP export** | **AP203 `MANIFOLD_SOLID_BREP`; board, trace/via prisms and extruded component bodies. Board cutouts are not represented.** |
 
 ### 7. Signal Integrity & Analysis
 
| Feature | Description |
|---------|-------------|
| Eye diagrams | PRBS-like signal with ISI, jitter, mask violations |
| Reflection simulation | Time-domain reflectometry with TDR analysis |
| Transmission line analysis | Frequency sweep: Z, phase, attenuation vs frequency |
| Crosstalk estimation | Near-end / far-end crosstalk between parallel traces |
| Impedance control | IPC-2141A microstrip, stackup-driven trace width |
| Length tuning | Serpentine/accordion patterns for matched lengths |
 
 ### 8. Rigid-Flex Support
 
| Feature | Description |
|---------|-------------|
| Flex zones | Named zones with bend radius and bend count |
| Stackup visualization | Visual layer stack with dielectric properties |
| KiCad stackup import | Parse KiCad .kicad_pcb for stackup data |
| Flex zone drawing | Click-to-draw zones on canvas |
 
 ### 9. Advanced Routing & Analysis
 
| Feature | Description |
|---------|-------------|
| Impedance control | IPC-2141A microstrip, stackup-driven trace width |
| Length tuning | Serpentine/accordion patterns for matched lengths |
| Bus routing | Parallel multi-net routing with fixed spacing |
| Via stitching/shielding | Ground via arrays along traces/lines |
| Advanced DFM | Acid trap, solder mask sliver, annular ring checks |
| SPICE netlist export | .cir format with simulation directives |
| ngspice wrapper | Transient/AC/DC simulation via ngspice |

### 10. Simulation

| Feature | Description |
|---------|-------------|
| Schematic simulation | The Sim sidebar tab runs the open schematic directly through ngspice |
| Analyses | Transient, AC sweep, DC sweep and operating point; the directive is editable |
| Results | Multi-vector SVG plot in the sidebar, plus generated netlist and ngspice output |
| Netlist export | `.cir` export with `.option filetype=ascii` so the rawfile can be parsed |
| Dependency | ngspice must be installed and on `PATH`; it is not bundled with TraceCraft |

The generator resolves pins from the component library, builds nets from wires,
junctions, labels and power symbols, and emits R/C/L/D/LED/NPN/PNP/VCC/GND devices.
Missing parameters fall back to safe defaults (5 V, 1 kΩ, 100 nF).

### 11. AI Copilot

| Feature | Description |
|---------|-------------|
| Component recommendations | Keyword-based recommendations from the local component catalog |
| Routing suggestions | Heuristic route hints along the direct path |
| Design review | Template answers for common clearance, width and via questions |
| Natural language Q&A | Template-based responses; no LLM runtime is enabled in this build |
| Local-only | No network calls; ONNX/llama.cpp dependencies are commented out |

## Tech Stack

| Layer | Technology | Rationale |
|-------|-----------|-----------|
| Desktop Shell | **Tauri 2.x** | Native performance, small binary, Rust backend |
| Frontend | **React 18 + TypeScript** | Mature ecosystem, Canvas/WebGL libraries |
| 2D Rendering | **React-Konva** (Canvas) | High-performance canvas for schematic/PCB editors |
| 3D Rendering | **Three.js** | WebGL-based 3D preview |
| State Management | **Zustand** | Lightweight, TypeScript-first atomic stores |
| Styling | **Tailwind CSS** | Utility-first, fast UI development |
| Backend (Rust) | **Tauri commands** | All data processing, algorithms, file I/O |
| Database | **SQLite via rusqlite** | Component library and app metadata; `.tcproj` files are JSON |
| Auto-Router | **Custom A\* pathfinding** in Rust | Port of open-source PCB router concepts |
| AI | **Keyword/template heuristics** | ONNX and llama.cpp are not enabled in this build |
| IPC | **Tauri invoke** | Typed IPC between React frontend and Rust backend |
| Build | **Vite** | Fast HMR, optimized builds for Tauri |
| 3D Models | **gltf + custom parsers** | GLTF/GLB via the `gltf` crate; STL/OBJ via local parsers |
| SPICE | **ngspice** (external) | Transient/AC/DC operating-point simulation |

## Non-Goals (MVP)

- Multi-user collaboration / cloud sync
- Web-based deployment
- Mobile app
- Plugin/extension system (post-MVP)
- Marketplace for shared components (post-MVP)

## Completed Post-MVP Features

- **SPICE Simulation Integration** — pin-aware netlist export, ngspice runner, ASCII rawfile parser, and a Sim sidebar panel with transient/AC/DC/operating-point analyses
- **Signal Integrity Analysis** — Eye diagrams, reflection simulation, transmission line analysis, crosstalk estimation
- **Rigid-Flex Support** — Flex zones with bend radius, stackup visualization with KiCad import
- **STEP Export** — AP203 `MANIFOLD_SOLID_BREP` export for board, copper and component bodies (OCCT-validated; cutouts omitted)
- **GLTF/GLB Support** — `.gltf` and `.glb` loading with external/embedded buffers, plus STL and OBJ
- **KiCad Library Download** — Fetch official KiCad symbol and footprint libraries into the local database
- **Impedance Control** — IPC-2141A microstrip calculator with stackup-driven trace width
- **Length Tuning** — Serpentine/accordion pattern generator for matched trace lengths
- **Bus Routing** — Parallel multi-net routing with fixed spacing
- **Via Stitching/Shielding** — Ground via array generation along traces and lines
- **Advanced DFM** — Acid trap detection, solder mask sliver checks

## Known Limitations

These are intentional boundaries of the current implementation, not roadmap items:

- **Gerber** is one copper file containing trace segments and zone outlines. It has no
  pads, vias, solder mask, silkscreen or NC drill data and is not manufacturable as-is.
- **IPC-2581** is a minimal hand-written XML document and is not validated against the
  IPC-2581 schema.
- **BOM/CPL** are CSV only; XLSX export is not wired up. The BOM lists reference, value,
  component ID and quantity, not manufacturer/part number.
- **KiCad export** is deliberately minimal: schematic symbols are written without
  embedded library definitions, and PCB footprints are written without pads or stackup.
  Files are useful for inspection and simple interop, not full round-trips.
- **STEP** omits board cutouts, emits copper as simple prisms, and carries no assembly,
  colour or material data.
- **DRC** pad clearance uses the footprint origin and a fixed 5 mm radius; it is an
  approximation, not real pad geometry, and reports any trace passing under a footprint.
  Ratsnest lines connect footprint origins rather than individual pads.
- **Manual routing** does not assign a net to new traces or vias, and the ratsnest
  considers a connection routed only when a trace runs origin-to-origin between the two
  footprints. Pad-to-pad traces therefore remain visible as ratsnest lines even though
  copper exists.
- **Auto-routing** computes A* routes and emits progress events, but no UI control
  invokes it yet, it does not persist the result to the board, and it does not emit vias
  at layer transitions. Manual routing is the production path.
- **Board outline** editing is basic; multiple contours are stored but not fully edited
  in the UI.
- **AI** features are keyword/template heuristics. ONNX Runtime and llama.cpp are not
  enabled in this build, and no models are downloaded.
- **Schematic image export** produces PNG only (no SVG/PDF).
- **Undo/redo** is only partially wired through the UI (see `docs/TASKS.md`).
- **ngspice** is an external runtime dependency and is not bundled in the `.deb` or
  AppImage.

## Architecture Overview

```
┌──────────────────────────────────────────────┐
│                 Tauri Shell                    │
│  ┌────────────────┐     ┌──────────────────┐  │
│  │   React SPA     │ ←──→ │   Rust Backend   │  │
│  │  (WebView)     │ IPC  │  (Tauri Commands) │  │
│  │                │      │                  │  │
│  │  - Schematic   │      │  - Project Mgr   │  │
│  │  - PCB Layout  │      │  - PCB Data Model│  │
│  │  - 3D Preview  │      │  - Auto-Router   │  │
│  │  - Library     │      │  - DRC Engine    │  │
│  │  - AI Chat UI  │      │  - File I/O      │  │
│  │                │      │  - Heuristics    │  │
│  └────────────────┘     └──────────────────┘  │
└──────────────────────────────────────────────┘
```

The React frontend handles all UI rendering and user interaction. The Rust backend
handles data processing, algorithms, file I/O, and local heuristic assistance.
Communication is through Tauri's typed `invoke` IPC.
