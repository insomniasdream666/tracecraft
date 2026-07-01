# TraceCraft — Specification

## Overview

**TraceCraft** is a local-first, AI-assisted PCB design desktop application inspired by
Flux.ai. It provides schematic capture, PCB layout editing, auto-routing, design rule
checking (DRC), 3D preview, and a local component library — all running natively via
Tauri with a React frontend and Rust backend.

Unlike cloud-based PCB tools, TraceCraft stores everything locally: designs, component
libraries, and AI models. No accounts, no subscriptions, no data leaving your machine.

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
| Netlist synchronization | Bidirectional sync with schematic changes |

### 3. Auto-Routing

| Feature | Description |
|---------|-------------|
| A\*-based pathfinding | Grid-based router with cost heuristics |
| Multi-layer routing | Layer assignment with via insertion |
| User-configurable rules | Preferred directions per layer, via cost, clearance |
| Fan-out | Automated BGA/QFP fan-out patterns |
| Batch routing | Route selected nets or entire board |

### 4. Component Library

| Feature | Description |
|---------|-------------|
| Local SQLite database | Symbols, footprints, 3D models, metadata |
| Component search | Search by name, manufacturer, category, parameters |
| Symbol editor | Create and edit schematic symbols |
| Footprint editor | Create and edit PCB footprints (land patterns) |
| 3D model attachment | Associate .step/.wrl models with footprints |
| Library import | KiCad library (.kicad_sym, .pretty), CSV, IPC-2581 |
| Custom categories | User-defined tagging and categorization |

### 5. File I/O

| Format | Direction | Notes |
|--------|-----------|-------|
| TraceCraft native (.tcproj) | Import/Export | SQLite-based single-file project format |
| KiCad schematic (.kicad_sch) | Import/Export | Interop with KiCad ecosystem |
| KiCad PCB (.kicad_pcb) | Import/Export | Full round-trip support |
| KiCad library (.kicad_sym, .pretty) | Import | Library ingestion |
| IPC-2581 | Export | Industry standard for fab output |
| Gerber RS-274X | Export | Fabrication output |
| Netlist (various) | Import/Export | PADS, Altium, Generic CSV |
| BOM (CSV, XLSX) | Export | Bill of Materials generation |

### 6. 3D Preview

| Feature | Description |
|---------|-------------|
| Real-time 3D rendering | WebGL-powered board visualization |
| Component bodies | 3D models attached to footprints |
| Layer visibility toggle | Show/hide copper, solder mask, silkscreen |
| Orbit/pan/zoom | Standard 3D viewport controls |
| Measurement tool | Distance, angle measurements in 3D |

### 7. AI Copilot

| Feature | Description |
|---------|-------------|
| Component recommendations | Suggest parts based on netlist context, voltage, current needs |
| Routing suggestions | Propose trace paths, via placements during manual routing |
| Design review | Scan for common mistakes, suggest fixes |
| Natural language Q&A | Local LLM chat about PCB design best practices |
| All inference runs locally | ONNX runtime for component models, llama.cpp for LLM |

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
| Database | **SQLite via rusqlite** | Embedded, zero-config, reliable |
| Auto-Router | **Custom A\* pathfinding** in Rust | Port of open-source PCB router concepts |
| AI Inference | **ONNX Runtime** (Rust) | Local component recommendation model |
| AI Chat | **llama.cpp** bindings | Local LLM for design Q&A |
| IPC | **Tauri invoke** | Typed IPC between React frontend and Rust backend |
| Build | **Vite** | Fast HMR, optimized builds for Tauri |

## Non-Goals (MVP)

- Multi-user collaboration / cloud sync
- SPICE simulation integration
- Web-based deployment
- Mobile app
- Plugin/extension system (post-MVP)
- Marketplace for shared components (post-MVP)

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
│  │                │      │  - AI Inference  │  │
│  └────────────────┘     └──────────────────┘  │
└──────────────────────────────────────────────┘
```

The React frontend handles all UI rendering and user interaction. The Rust backend
handles all data processing, algorithms, file I/O, and AI inference. Communication
is through Tauri's typed `invoke` IPC.
