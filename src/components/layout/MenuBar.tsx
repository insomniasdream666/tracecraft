import React, { useState, useRef, useEffect } from "react";
import { open as openDialog, save as saveDialog } from "@tauri-apps/plugin-dialog";
import { useProjectStore } from "../../stores/projectStore";
import { useLibraryStore } from "../../stores/libraryStore";
import { useSchematicStore } from "../../stores/schematicStore";
import { usePCBStore } from "../../stores/pcbStore";
import { useUIStore } from "../../stores/uiStore";
import toast from "react-hot-toast";
import { DocumentationModal } from "../common/DocumentationModal";
import { ShortcutsModal } from "../common/ShortcutsModal";
import { AboutModal } from "../common/AboutModal";

interface MenuItem {
  label: string;
  shortcut?: string;
  onClick: () => void;
  divider?: boolean;
  disabled?: boolean;
}

interface MenuDef {
  label: string;
  items: MenuItem[];
}

export function MenuBar() {
  const [activeMenu, setActiveMenu] = useState<string | null>(null);
  const [showDocs, setShowDocs] = useState(false);
  const [showShortcuts, setShowShortcuts] = useState(false);
  const [showAbout, setShowAbout] = useState(false);
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const handler = (e: MouseEvent) => {
      if (ref.current && !ref.current.contains(e.target as Node)) setActiveMenu(null);
    };
    document.addEventListener("mousedown", handler);
    return () => document.removeEventListener("mousedown", handler);
  }, []);

  const { create, open: openProject, save, close, current,
    importKicadSch, importKicadPcb, exportKicadSch, exportKicadPcb,
    exportGerber, exportIpc2581, exportBom, exportCpl, exportSchematicImage, exportSpiceNetlist } = useProjectStore();
  const { importFile } = useLibraryStore();

  const menus: MenuDef[] = [
    {
      label: "File",
      items: [
        {
          label: "New Project", shortcut: "Ctrl+N",
          onClick: async () => {
            const name = prompt("Project name:");
            if (name) { await create(name.trim()); setActiveMenu(null); }
          },
        },
        {
          label: "Open Project", shortcut: "Ctrl+O",
          onClick: async () => {
            const path = await openDialog({ filters: [{ name: "TraceCraft Project", extensions: ["tcproj"] }] });
            if (path) { await openProject(path); toast.success("Project opened"); setActiveMenu(null); }
          },
        },
        {
          label: "Save Project", shortcut: "Ctrl+S",
          onClick: async () => {
            const path = await saveDialog({ filters: [{ name: "TraceCraft Project", extensions: ["tcproj"] }] });
            if (path) { await save(path); toast.success("Project saved"); setActiveMenu(null); }
          },
          disabled: !current,
        },
        { label: "", onClick: () => {}, divider: true },
        {
          label: "Import KiCad Schematic",
          onClick: async () => {
            const path = await openDialog({ filters: [{ name: "KiCad Schematic", extensions: ["kicad_sch", "sch"] }] });
            if (path) { await importKicadSch(path); toast.success("Schematic imported"); setActiveMenu(null); }
          },
        },
        {
          label: "Import KiCad PCB",
          onClick: async () => {
            const path = await openDialog({ filters: [{ name: "KiCad PCB", extensions: ["kicad_pcb"] }] });
            if (path) { await importKicadPcb(path); toast.success("PCB imported"); setActiveMenu(null); }
          },
        },
        {
          label: "Import Library",
          onClick: async () => {
            const paths = await openDialog({ multiple: true, filters: [{ name: "KiCad Symbol Library", extensions: ["kicad_sym"] }] });
            if (paths && paths.length > 0) {
              let total = 0;
              for (const p of paths) total += await importFile(p, "kicad_sym");
              toast.success(`Imported ${total} components`); setActiveMenu(null);
            }
          },
        },
        { label: "", onClick: () => {}, divider: true },
        {
          label: "Export KiCad Schematic",
          onClick: async () => {
            const path = await saveDialog({ filters: [{ name: "KiCad Schematic", extensions: ["kicad_sch"] }] });
            if (path) { await exportKicadSch(path); toast.success("Exported"); setActiveMenu(null); }
          },
          disabled: !current,
        },
        {
          label: "Export KiCad PCB",
          onClick: async () => {
            const path = await saveDialog({ filters: [{ name: "KiCad PCB", extensions: ["kicad_pcb"] }] });
            if (path) { await exportKicadPcb(path); toast.success("Exported"); setActiveMenu(null); }
          },
          disabled: !current,
        },
        {
          label: "Export Gerber",
          onClick: async () => {
            const path = await saveDialog({ filters: [{ name: "Gerber", extensions: ["gbr"] }] });
            if (path) { await exportGerber(path); toast.success("Gerber exported"); setActiveMenu(null); }
          },
          disabled: !current,
        },
        { label: "", onClick: () => {}, divider: true },
        {
          label: "Export Image (Schematic)",
          onClick: async () => {
            const path = await saveDialog({ filters: [{ name: "PNG Image", extensions: ["png"] }] });
            if (path) { await exportSchematicImage(path); toast.success("Image exported"); setActiveMenu(null); }
          },
          disabled: !current,
        },
        {
          label: "Export BOM CSV",
          onClick: async () => {
            const path = await saveDialog({ filters: [{ name: "CSV", extensions: ["csv"] }] });
            if (path) { await exportBom(path); toast.success("BOM exported"); setActiveMenu(null); }
          },
          disabled: !current,
        },
        {
          label: "Export Pick-and-Place (CPL)",
          onClick: async () => {
            const path = await saveDialog({ filters: [{ name: "CSV", extensions: ["csv"] }] });
            if (path) { await exportCpl(path); toast.success("CPL exported"); setActiveMenu(null); }
          },
          disabled: !current,
        },
        { label: "", onClick: () => {}, divider: true },
        {
          label: "Export SPICE Netlist",
          onClick: async () => {
            const path = await saveDialog({ filters: [{ name: "SPICE Netlist", extensions: ["cir"] }] });
            if (path) { await exportSpiceNetlist(path); toast.success("SPICE netlist exported"); setActiveMenu(null); }
          },
          disabled: !current,
        },
        { label: "", onClick: () => {}, divider: true },
        {
          label: "Close Project",
          onClick: async () => { await close(); setActiveMenu(null); },
          disabled: !current,
        },
      ],
    },
    {
      label: "Edit",
      items: [
        {
          label: "Undo", shortcut: "Ctrl+Z",
          onClick: () => {
            const { mode } = useUIStore.getState();
            if (mode === "pcb") usePCBStore.getState().undo();
            else if (mode === "schematic") useSchematicStore.getState().undo();
            setActiveMenu(null);
          },
          disabled: !current,
        },
        {
          label: "Redo", shortcut: "Ctrl+Shift+Z",
          onClick: () => {
            const { mode } = useUIStore.getState();
            if (mode === "pcb") usePCBStore.getState().redo();
            else if (mode === "schematic") useSchematicStore.getState().redo();
            setActiveMenu(null);
          },
          disabled: !current,
        },
        { label: "", onClick: () => {}, divider: true },
        {
          label: "Annotate Schematic",
          onClick: () => { useSchematicStore.getState().annotate(); setActiveMenu(null); },
          disabled: !current,
        },
        {
          label: "Run ERC",
          onClick: async () => {
            const violations = await useSchematicStore.getState().runERC();
            if (violations.length === 0) toast.success("No ERC violations");
            else toast.error(`${violations.length} ERC violation(s) found`);
            setActiveMenu(null);
          },
          disabled: !current,
        },
      ],
    },
    {
      label: "View",
      items: [
        {
          label: "Zoom In", shortcut: "Ctrl++",
          onClick: () => {
            const { zoom, setZoom } = useUIStore.getState();
            setZoom(zoom * 1.2); setActiveMenu(null);
          },
        },
        {
          label: "Zoom Out", shortcut: "Ctrl+-",
          onClick: () => {
            const { zoom, setZoom } = useUIStore.getState();
            setZoom(zoom / 1.2); setActiveMenu(null);
          },
        },
        {
          label: "Zoom to Fit",
          onClick: async () => {
            const { mode, canvasApi } = useUIStore.getState();
            if (!canvasApi) return;
            try {
              const { invoke } = await import("@tauri-apps/api/core");
              if (mode === "pcb") {
                const bounds = await invoke<any>("get_board_bounds");
                canvasApi.zoomToBounds(bounds);
              } else if (mode === "schematic") {
                const sheet = useSchematicStore.getState().sheet;
                if (sheet) {
                  const bounds = await invoke<any>("get_schematic_bounds", { sheetId: sheet.id });
                  canvasApi.zoomToBounds(bounds);
                }
              }
            } catch {}
            setActiveMenu(null);
          },
        },
        { label: "", onClick: () => {}, divider: true },
        {
          label: "Toggle Sidebar",
          onClick: () => { useUIStore.getState().toggleSidebar(); setActiveMenu(null); },
        },
      ],
    },
    {
      label: "Help",
      items: [
        {
          label: "Documentation",
          onClick: () => { setShowDocs(true); setActiveMenu(null); },
        },
        {
          label: "Keyboard Shortcuts",
          onClick: () => { setShowShortcuts(true); setActiveMenu(null); },
        },
        { label: "", onClick: () => {}, divider: true },
        {
          label: "About TraceCraft",
          onClick: () => { setShowAbout(true); setActiveMenu(null); },
        },
      ],
    },
  ];

  return (
    <>
    <div ref={ref} className="flex items-center h-7 bg-bg-deep border-b border-border px-1 gap-0">
      {menus.map((menu) => (
        <div key={menu.label} className="relative">
          <button
            onClick={() => setActiveMenu(activeMenu === menu.label ? null : menu.label)}
            className={`px-3 py-0.5 text-xs rounded transition-colors ${
              activeMenu === menu.label
                ? "bg-bg-elevated text-text-primary"
                : "text-text-secondary hover:bg-bg-surface hover:text-text-primary"
            }`}
          >
            {menu.label}
          </button>
          {activeMenu === menu.label && (
            <div className="absolute top-full left-0 mt-0.5 w-52 bg-bg-surface border border-border rounded shadow-lg z-50 py-1">
              {menu.items.map((item, i) =>
                item.divider ? (
                  <div key={i} className="h-px bg-border my-1" />
                ) : (
                  <button
                    key={i}
                    onClick={item.onClick}
                    disabled={item.disabled}
                    className="w-full flex items-center gap-2 px-3 py-1.5 text-xs text-text-secondary hover:bg-bg-elevated hover:text-text-primary disabled:opacity-30 disabled:cursor-not-allowed transition-colors text-left"
                  >
                    <span className="flex-1">{item.label}</span>
                    {item.shortcut && <span className="text-text-muted text-[10px]">{item.shortcut}</span>}
                  </button>
                )
              )}
            </div>
          )}
        </div>
      ))}
    </div>
      {showDocs && <DocumentationModal onClose={() => setShowDocs(false)} />}
      {showShortcuts && <ShortcutsModal onClose={() => setShowShortcuts(false)} />}
      {showAbout && <AboutModal onClose={() => setShowAbout(false)} />}
    </>
  );
}
