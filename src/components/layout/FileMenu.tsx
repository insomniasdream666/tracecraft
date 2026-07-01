import React, { useState, useRef, useEffect } from "react";
import { File, FolderOpen, Save, Upload, Download, X } from "lucide-react";
import { open as openDialog, save as saveDialog } from "@tauri-apps/plugin-dialog";
import { useProjectStore } from "../../stores/projectStore";
import { useLibraryStore } from "../../stores/libraryStore";
import { useSchematicStore } from "../../stores/schematicStore";

interface MenuItem {
  label: string;
  icon?: React.ReactNode;
  shortcut?: string;
  onClick: () => void;
  divider?: boolean;
  disabled?: boolean;
}

export function FileMenu() {
  const [open, setOpen] = useState(false);
  const ref = useRef<HTMLDivElement>(null);
  const { create, open: openProject, save, close, current, importKicadSch, exportKicadSch, importKicadPcb, exportKicadPcb, exportGerber, exportIpc2581, exportBom } = useProjectStore();
  const { importFile } = useLibraryStore();

  useEffect(() => {
    const handler = (e: MouseEvent) => {
      if (ref.current && !ref.current.contains(e.target as Node)) setOpen(false);
    };
    document.addEventListener("mousedown", handler);
    return () => document.removeEventListener("mousedown", handler);
  }, []);

  const items: MenuItem[] = [
    {
      label: "New Project", icon: <File size={14} />, shortcut: "Ctrl+N",
      onClick: async () => {
        const name = prompt("Project name:");
        if (name) { await create(name); setOpen(false); }
      },
    },
    {
      label: "Open Project", icon: <FolderOpen size={14} />, shortcut: "Ctrl+O",
      onClick: async () => {
        const path = await openDialog({ filters: [{ name: "TraceCraft Project", extensions: ["tcproj"] }] });
        if (path) { await openProject(path); setOpen(false); }
      },
    },
    {
      label: "Save Project", icon: <Save size={14} />, shortcut: "Ctrl+S",
      onClick: async () => {
        const path = await saveDialog({ filters: [{ name: "TraceCraft Project", extensions: ["tcproj"] }] });
        if (path) { await save(path); setOpen(false); }
      },
      disabled: !current,
    },
    { label: "", onClick: () => {}, divider: true },
    {
      label: "Import KiCad Schematic", icon: <Upload size={14} />,
      onClick: async () => {
        const path = await openDialog({ filters: [{ name: "KiCad Schematic", extensions: ["kicad_sch", "sch"] }] });
        if (path) { await importKicadSch(path); setOpen(false); }
      },
    },
    {
      label: "Import KiCad PCB", icon: <Upload size={14} />,
      onClick: async () => {
        const path = await openDialog({ filters: [{ name: "KiCad PCB", extensions: ["kicad_pcb", "kicad_pcb"] }] });
        if (path) { await importKicadPcb(path); setOpen(false); }
      },
    },
    {
      label: "Import Library (KiCad sym)", icon: <Upload size={14} />,
      onClick: async () => {
        const paths = await openDialog({ multiple: true, filters: [{ name: "KiCad Symbol Library", extensions: ["kicad_sym"] }] });
        if (paths && paths.length > 0) {
          let total = 0;
          for (const p of paths) { total += await importFile(p, "kicad_sym"); }
          alert(`Imported ${total} components from ${paths.length} file(s)`);
          setOpen(false);
        }
      },
    },
    {
      label: "Import Footprint (KiCad .kicad_mod)", icon: <Upload size={14} />,
      onClick: async () => {
        const selected = await openDialog({ multiple: true, filters: [{ name: "KiCad Footprint", extensions: ["kicad_mod"] }] });
        if (selected && selected.length > 0) {
          let total = 0;
          for (const p of selected) { total += await importFile(p, "kicad_mod"); }
          alert(`Imported ${total} footprint(s) from ${selected.length} file(s)`);
          setOpen(false);
        }
      },
    },
    {
      label: "Import Footprint Library (.pretty)", icon: <Upload size={14} />,
      onClick: async () => {
        const selected = await openDialog({ directory: true, title: "Select .pretty directory" });
        if (selected) {
          const count = await importFile(selected, "kicad_pretty");
          alert(`Imported ${count} footprint(s) from ${selected}`);
          setOpen(false);
        }
      },
    },
    {
      label: "Import BOM (CSV)", icon: <Upload size={14} />,
      onClick: async () => {
        const paths = await openDialog({ multiple: true, filters: [{ name: "CSV BOM", extensions: ["csv"] }] });
        if (paths && paths.length > 0) {
          let total = 0;
          for (const p of paths) { total += await importFile(p, "csv_bom"); }
          alert(`Imported ${total} components from ${paths.length} file(s)`);
          setOpen(false);
        }
      },
    },
    { label: "", onClick: () => {}, divider: true },
    {
      label: "Export Schematic Image", icon: <Download size={14} />,
      onClick: async () => {
        await useSchematicStore.getState().exportImage();
        setOpen(false);
      },
      disabled: !current,
    },
    {
      label: "Export KiCad Schematic", icon: <Download size={14} />,
      onClick: async () => {
        const path = await saveDialog({ filters: [{ name: "KiCad Schematic", extensions: ["kicad_sch"] }] });
        if (path) { await exportKicadSch(path); setOpen(false); }
      },
      disabled: !current,
    },
    {
      label: "Export KiCad PCB", icon: <Download size={14} />,
      onClick: async () => {
        const path = await saveDialog({ filters: [{ name: "KiCad PCB", extensions: ["kicad_pcb"] }] });
        if (path) { await exportKicadPcb(path); setOpen(false); }
      },
      disabled: !current,
    },
    {
      label: "Export Gerber", icon: <Download size={14} />,
      onClick: async () => {
        const path = await saveDialog({ filters: [{ name: "Gerber", extensions: ["gbr"] }] });
        if (path) { await exportGerber(path); setOpen(false); }
      },
      disabled: !current,
    },
    {
      label: "Export IPC-2581", icon: <Download size={14} />,
      onClick: async () => {
        const path = await saveDialog({ filters: [{ name: "IPC-2581", extensions: ["xml"] }] });
        if (path) { await exportIpc2581(path); setOpen(false); }
      },
      disabled: !current,
    },
    {
      label: "Export BOM (CSV)", icon: <Download size={14} />,
      onClick: async () => {
        const path = await saveDialog({ filters: [{ name: "CSV", extensions: ["csv"] }] });
        if (path) { await exportBom(path); setOpen(false); }
      },
      disabled: !current,
    },
    { label: "", onClick: () => {}, divider: true },
    {
      label: "Close Project", icon: <X size={14} />,
      onClick: async () => { await close(); setOpen(false); },
      disabled: !current,
    },
  ];

  return (
    <div ref={ref} className="relative">
      <button
        onClick={() => setOpen(!open)}
        className="flex items-center gap-1 px-2 py-1 text-xs font-medium text-text-secondary hover:text-text-primary hover:bg-bg-elevated rounded transition-colors"
      >
        <File size={14} />
        File
      </button>
      {open && (
        <div className="absolute top-full left-0 mt-1 w-56 bg-bg-surface border border-border rounded shadow-lg z-50 py-1">
          {items.map((item, i) =>
            item.divider ? (
              <div key={i} className="h-px bg-border my-1" />
            ) : (
              <button
                key={i}
                onClick={item.onClick}
                disabled={item.disabled}
                className="w-full flex items-center gap-2 px-3 py-1.5 text-xs text-text-secondary hover:bg-bg-elevated hover:text-text-primary disabled:opacity-30 disabled:cursor-not-allowed transition-colors"
              >
                {item.icon && <span className="w-4 flex-shrink-0">{item.icon}</span>}
                <span className="flex-1 text-left">{item.label}</span>
                {item.shortcut && <span className="text-text-muted ml-auto">{item.shortcut}</span>}
              </button>
            )
          )}
        </div>
      )}
    </div>
  );
}
