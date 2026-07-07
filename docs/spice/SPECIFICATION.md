# SPICE Integration — Specification

## Overview

Integrate SPICE simulation into TraceCraft, allowing users to export schematic netlists to
SPICE format, run ngspice simulations, and visualize waveforms — all within the app. No
external tools required beyond an ngspice binary.

## User Stories

- **As a circuit designer**, I want to export my schematic as a SPICE netlist to verify
  correctness.
- **As a designer**, I want to configure a simulation (DC sweep, transient, AC analysis)
  from within the app.
- **As a designer**, I want to run ngspice and view waveform results without leaving
  TraceCraft.
- **As a designer**, I want to click on nets/wires in the schematic to probe them, then
  see their waveforms plotted after simulation.
- **As a designer**, I want to zoom, pan, and measure (voltage difference, time
  difference) on waveforms.

## Scope

### In Scope (v1)

| Feature | Description |
|---------|------------|
| SPICE netlist export | Export schematic to `.cir` format (fully compatible with ngspice) |
| Simulation config panel | UI to set analysis type (.OP, .DC, .TRAN, .AC) and parameters |
| ngspice runner | Run ngspice as subprocess with progress feedback |
| Raw output parser | Parse ngspice `.raw` binary output into structured data |
| Waveform viewer | Canvas-based plot with multiple traces, zoom, pan, cursors |
| Probe system | Click nets on schematic to add probes → auto-plotted after simulation |
| Model library | Basic support for R, L, C, D, Q, U (IC) with SPICE parameters from component values |

### Out of Scope (v1)

- Interactive schematic probing during live simulation (deferred)
- Monte Carlo / sensitivity analysis
- Digital simulation (ngspice's `a` and `d` devices)
- Subcircuit autogeneration from hierarchical sheets
- SPICE model editor / library browser
- FFT / noise analysis plots
- Export to other simulators (PSpice, LTspice)

## User Interface

### Simulation Panel (Sidebar)

When in schematic mode, a new "Simulation" tab appears in the sidebar containing:

1. **Analysis Type** dropdown: `.OP` (Operating Point), `.DC` (DC Sweep), `.TRAN`
   (Transient), `.AC` (AC Sweep)
2. **Parameter fields** dependent on analysis type:
   - `.TRAN`: Stop Time (s), Max Step (s), Start Time (s — optional)
   - `.DC`: Source name, Start V, Stop V, Step V
   - `.AC`: Sweep type (decade/octave/linear), Points, Start Freq, Stop Freq
3. **Run / Stop** button
4. **Status** indicator: Idle / Running / Complete / Error
5. **Probe list**: checkboxes to toggle visibility on waveform plot
6. **Export Netlist** button

### Waveform Viewer (Replace or overlay sidebar)

After simulation completes, the sidebar switches to waveform view:

1. **Plot canvas**: SVG or Konva-based with axis labels, grid lines
2. **Trace list**: signal name + color swatch + visibility toggle
3. **Measurement tools**: cursor readout (voltage/time delta)
4. **Zoom**: scroll-wheel zoom, drag-to-zoom on plot area
5. **Pan**: click-drag on plot area

### Probe System (on Schematic Canvas)

1. **Probe mode toggle** in toolbar (probe icon)
2. When active, clicking any wire or net label places a probe marker
3. Probes render as small orange/red dots with signal labels on the schematic
4. Probes persist across mode switches
5. Probes feed into the simulation run — all probed nets are saved as `.plot` or `.print`
   directives in the SPICE netlist

## Architecture

```
┌──────────────────────────────────────────────────────────────┐
│                    Frontend (React/TypeScript)                │
│                                                               │
│  ┌──────────┐  ┌──────────────┐  ┌──────────────┐           │
│  │SimPanel  │  │WaveformViewer│  │ProbeOverlay  │           │
│  │(config ) │  │(plot canvas )│  │(schematic    │           │
│  │ run/stop │  │ cursor meas  │  │ canvas)      │           │
│  └────┬─────┘  └──────┬───────┘  └──────┬───────┘           │
│       │               │                 │                     │
│  ┌────▼───────────────▼─────────────────▼──────┐             │
│  │           spiceStore (Zustand)               │             │
│  │  config | probes | results | status          │             │
│  └───────────────────┬─────────────────────────┘             │
│                      │ invoke()                               │
└──────────────────────┼───────────────────────────────────────┘
                       │
┌──────────────────────┼───────────────────────────────────────┐
│           Tauri IPC  │                                        │
│                      ▼                                        │
│  ┌─────────────────────────────────────────┐                 │
│  │          Rust Backend                    │                 │
│  │                                          │                 │
│  │  ┌─────────────────┐                    │                 │
│  │  │ export_spice_    │  io/spice.rs      │                 │
│  │  │ netlist          │  (netlist writer)  │                 │
│  │  └────────┬────────┘                    │                 │
│  │           │                              │                 │
│  │  ┌────────▼────────┐                    │                 │
│  │  │ simulate_spice  │  engine/spice.rs   │                 │
│  │  │ (subprocess     │  (ngspice runner,  │                 │
│  │  │  management)    │   raw parser)      │                 │
│  │  └────────┬────────┘                    │                 │
│  │           │                              │                 │
│  │  ┌────────▼────────┐                    │                 │
│  │  │ AppState        │  app_state.rs      │                 │
│  │  │ .spice_state    │  (probes, results) │                 │
│  │  └─────────────────┘                    │                 │
│  └─────────────────────────────────────────┘                 │
└──────────────────────────────────────────────────────────────┘
```

## Data Model

### New Rust Types (in `model/spice.rs`)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpiceConfig {
    pub analysis_type: AnalysisType,
    pub tran_params: Option<TransientParams>,
    pub dc_params: Option<DcSweepParams>,
    pub ac_params: Option<AcSweepParams>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnalysisType { Op, Dc, Tran, Ac }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransientParams {
    pub stop_time: f64,
    pub max_step: f64,
    pub start_time: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DcSweepParams {
    pub source_name: String,
    pub start_v: f64,
    pub stop_v: f64,
    pub step_v: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcSweepParams {
    pub sweep_type: SweepType,
    pub points: u32,
    pub start_freq: f64,
    pub stop_freq: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SweepType { Decade, Octave, Linear }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpiceProbe {
    pub net_name: String,
    pub position: Point,          // schematic coordinates
    pub sheet_id: SheetId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationResult {
    pub analysis_type: AnalysisType,
    pub probes: Vec<TraceData>,
    pub status: SimulationStatus,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SimulationStatus { Idle, Running, Complete, Error }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceData {
    pub net_name: String,
    pub x_label: String,          // "Time (s)", "Voltage (V)", "Frequency (Hz)"
    pub y_label: String,          // "Voltage (V)" or "dB"
    pub x_values: Vec<f64>,
    pub y_values: Vec<f64>,
}
```

### Addition to `AppState`

```rust
pub struct SpiceState {
    pub config: SpiceConfig,
    pub probes: Vec<SpiceProbe>,
    pub result: Option<SimulationResult>,
}
```

### Addition to `SchematicSheet` (optional — probes could be stored separately)

Probes are stored in `SpiceState` in the `AppState`, keyed by schematic sheet.

## SPICE Netlist Format

The exporter produces standard ngspice-compatible netlists:

```spice
* TraceCraft SPICE Netlist
* Generated: 2026-07-01

R1 N001 0 10k
C1 N001 N002 100n
Q1 N002 N003 0 2N2222
V1 N003 0 DC 5V

.TRAN 1u 10m
.PLOT TRAN V(N001) V(N002)
.END
```

Model library: ngspice built-in models suffice for v1. Component SPICE values come from:

| Component | Value field → SPICE | Example |
|-----------|-------------------|---------|
| Resistor | Resistance | `R1 N001 0 10k` |
| Capacitor | Capacitance | `C1 N001 0 100n` |
| Inductor | Inductance | `L1 N001 0 10m` |
| Diode | Part number → model | `D1 N001 0 1N4148` |
| BJT | Part number → model | `Q1 N001 N002 N003 2N2222` |
| Voltage source | Value as DC | `V1 N001 0 DC 5V` |
| Current source | Value as DC | `I1 0 N001 DC 10mA` |

## Component Model Mapping

For simulation, each component needs a SPICE model. The approach:

1. **Passive components (R, L, C):** Value field contains the numeric value. The exporter
   reads the component's value and writes it directly as the SPICE value parameter.
2. **Semiconductors (D, Q):** The part number in the component's value or refdes prefix
   determines the SPICE model. ngspice includes built-in models (1N4148, 2N2222, etc.).
   User can add `.MODEL` statements via text annotations.
3. **ICs (U):** Represented as subcircuit calls in SPICE. v1 supports basic `.SUBCKT`
   defined in the netlist or from the library.

## File Changes

### New Files

| File | Purpose |
|------|---------|
| `src-tauri/src/model/spice.rs` | SPICE data types |
| `src-tauri/src/io/spice.rs` | SPICE netlist writer |
| `src-tauri/src/engine/spice.rs` | ngspice subprocess runner + raw parser |
| `src-tauri/src/commands/spice.rs` | IPC commands: `export_spice_netlist`, `run_simulation`, `stop_simulation`, `get_simulation_result`, `add_probe`, `remove_probe`, `list_probes` |
| `src/stores/spiceStore.ts` | Zustand store for simulation state |
| `src/components/simulation/SimPanel.tsx` | Simulation configuration panel |
| `src/components/simulation/WaveformViewer.tsx` | Waveform plot canvas |
| `src/components/simulation/WaveformPlot.tsx` | Internal plot rendering (SVG-based) |
| `src/components/simulation/ProbeOverlay.tsx` | Probe markers on schematic canvas |

### Modified Files

| File | Change |
|------|--------|
| `src-tauri/src/lib.rs` | Add `model::spice`, `io::spice`, `engine::spice`, `commands::spice` modules; register new commands in `generate_handler![]` |
| `src-tauri/src/app_state.rs` | Add `SpiceState` field to `AppState` |
| `src-tauri/src/model/mod.rs` | Add `pub mod spice;` |
| `src-tauri/src/io/mod.rs` | Add `pub mod spice;` |
| `src-tauri/src/engine/mod.rs` | Add `pub mod spice;` |
| `src-tauri/src/commands/mod.rs` | Add `pub mod spice;` |
| `src/components/layout/Sidebar.tsx` | Add "Simulation" tab |
| `src/components/layout/Toolbar.tsx` | Add probe mode toggle button |
| `src/components/schematic/SchematicCanvas.tsx` | Handle probe click events in probe mode |
| `src/stores/index.ts` | Export `useSpiceStore` |

## Dependencies

### Rust
- No new crate dependencies for v1. ngspice subprocess is managed via `std::process::Command`.
- Raw output parsing uses standard library string/number parsing.
- The `serde` / `serde_json` crates are already present for serialization.

### Frontend
- No new npm dependencies. Waveform plot uses SVG rendered inline (React components).
- Existing `konva`/`react-konva` used for probe markers on schematic canvas.

### System
- `ngspice` binary must be installed on the user's system and available in PATH.
- The app will detect ngspice availability at startup and display a helpful message if
  not found.
