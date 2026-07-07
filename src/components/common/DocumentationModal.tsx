import React from "react";
import { X, Book, CircuitBoard, Cpu, Library, FileDown, Keyboard, HelpCircle } from "lucide-react";

interface Props {
  onClose: () => void;
}

const sections = [
  {
    id: "getting-started",
    icon: Book,
    title: "Getting Started",
    content: `TraceCraft is an AI-assisted PCB design tool. Start by creating a new project:

1. Click "New Project" on the welcome screen or press Ctrl+N
2. Enter a project name and choose a save location
3. The schematic editor opens with a blank sheet
4. Drag components from the Library tab onto the canvas
5. Connect them with wires using the Wire tool (W)
6. Switch to PCB view to lay out the board
7. Route traces between pads
8. Export Gerber files for fabrication`,
  },
  {
    id: "schematic",
    icon: CircuitBoard,
    title: "Schematic Editor",
    content: `The schematic editor is where you design the circuit:

• Adding Components: Drag from the Library tab or click "Place Component" and select from the library
• Wiring: Select the Wire tool (W), click a pin to start, click to add corners, double-click to finish
• Net Labels: Use the Label tool to name nets (e.g., VCC, GND). Nets with the same name are connected
• Junctions: Place junctions at wire intersections to connect them
• Buses: Group related nets into buses for cleaner schematics
• Annotation: Auto-assign reference designators (R1, R2, C1, etc.) via Edit > Annotate Schematic
• ERC: Run Electrical Rules Check to find floating pins, unconnected nets, etc.
• Hierarchical Sheets: Create sub-schematics for complex designs

Keyboard shortcuts: Ctrl+Z (undo), Ctrl+S (save), Delete (remove selected)`,
  },
  {
    id: "pcb",
    icon: Cpu,
    title: "PCB Layout",
    content: `The PCB layout editor arranges components and routes traces:

• Board Outline: Set board dimensions in the Properties panel (width x height in mm)
• Placing Footprints: Drag components from the Library onto the PCB canvas
• Routing: Select the Route tool (R), click a pad to start, click to add vertices, double-click to finish
• Vias: Press V during routing to switch layers with a via
• Zones: Draw copper pour zones with the Zone tool
• Ratsnest: Shows unrouted connections as yellow lines (airwires)
• Layers: Toggle layer visibility in the Layers tab
• Autorouting: Use Route > Auto-Route All to automatically route all nets
• DRC: Run Design Rule Check to verify clearances and widths`,
  },
  {
    id: "library",
    icon: Library,
    title: "Component Library",
    content: `Manage components in the Library tab:

• Built-in Components: Basic components (resistors, capacitors, diodes, transistors) come pre-loaded
• Import: Click "Import Library" to add KiCad symbol (.kicad_sym) or CSV BOM files
• Download KiCad Libraries: Click "Download KiCad Libs" to fetch thousands of official KiCad components
• Create Components: Click "New Component" to draw custom symbols and footprints
• Search: Type to search by name, or filter by category in the sidebar
• Edit: Click a component to view details, then edit its symbol or footprint
• Drag & Drop: Drag components from the library onto the schematic or PCB canvas`,
  },
  {
    id: "export",
    icon: FileDown,
    title: "Export & File Operations",
    content: `TraceCraft supports several file formats:

• Save/Load: Projects are saved as .tcproj files (File > Save / Ctrl+S)
• KiCad Export: Export schematics as .kicad_sch and PCBs as .kicad_pcb for KiCad interop
• KiCad Import: Open existing KiCad schematics and PCB files
• Gerber Export: Generate RS-274X Gerber files for PCB fabrication (one file per layer)
• IPC-2581: Export to the industry-standard IPC-2581 format
• BOM Export: Generate Bill of Materials as CSV or XLSX
• Image Export: Export schematics as PNG or SVG`,
  },
  {
    id: "shortcuts",
    icon: Keyboard,
    title: "Keyboard Shortcuts",
    content: `Global shortcuts:
Ctrl+N        New project
Ctrl+O        Open project
Ctrl+S        Save project
Ctrl+Z        Undo
Ctrl+Shift+Z  Redo
Ctrl++        Zoom in
Ctrl+-        Zoom out

Schematic tools (press key to activate):
S             Select tool
W             Wire tool
C             Component placement
L             Net label tool
J             Junction tool

PCB tools (press key to activate):
S             Select tool
R             Route trace tool
Z             Zone tool
V             Add via during routing
Delete        Remove selected element

Canvas controls:
Scroll wheel  Zoom in/out
Middle click  Pan
Click+drag    Select (in select mode)`,
  },
];

export function DocumentationModal({ onClose }: Props) {
  const [activeSection, setActiveSection] = React.useState(sections[0].id);

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60" onClick={onClose}>
      <div
        className="bg-bg-surface rounded-xl border border-border shadow-2xl w-[800px] max-h-[80vh] flex overflow-hidden"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="w-48 border-r border-border p-2 space-y-0.5 overflow-y-auto shrink-0">
          <div className="px-2 py-2 text-xs font-semibold text-text-muted uppercase tracking-wider flex items-center gap-1.5">
            <HelpCircle size={13} />
            Help
          </div>
          {sections.map((s) => (
            <button
              key={s.id}
              onClick={() => setActiveSection(s.id)}
              className={`w-full flex items-center gap-2 text-left px-2 py-1.5 text-xs rounded transition-colors ${
                activeSection === s.id
                  ? "bg-copper text-white"
                  : "text-text-secondary hover:bg-bg-elevated hover:text-text-primary"
              }`}
            >
              <s.icon size={13} />
              {s.title}
            </button>
          ))}
        </div>
        <div className="flex-1 flex flex-col min-w-0">
          <div className="flex items-center justify-between px-4 py-3 border-b border-border">
            <h2 className="text-sm font-semibold text-text-primary">
              {sections.find((s) => s.id === activeSection)?.title}
            </h2>
            <button onClick={onClose} className="text-text-muted hover:text-text-primary transition-colors">
              <X size={16} />
            </button>
          </div>
          <div className="flex-1 overflow-y-auto p-4">
            <div className="text-xs text-text-secondary leading-relaxed whitespace-pre-line font-sans">
              {sections.find((s) => s.id === activeSection)?.content}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
