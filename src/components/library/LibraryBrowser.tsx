import React, { useEffect, useState, useCallback, useRef } from "react";
import { Search, Upload, Download, X, Folder, Edit3, Plus } from "lucide-react";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useLibraryStore } from "../../stores/libraryStore";
import { ComponentDetail } from "./ComponentDetail";
import { CategoryTree } from "./CategoryTree";
import { SymbolEditor } from "./SymbolEditor";
import { FootprintEditor } from "./FootprintEditor";
import { Component, SymbolDef, FootprintDef } from "../../types/library";
import toast from "react-hot-toast";

interface DownloadProgress {
  phase: string;
  current: number;
  total: number;
  message: string;
}

function LoadingSkeleton() {
  return (
    <div className="grid grid-cols-2 gap-2 p-2">
      {Array.from({ length: 6 }).map((_, i) => (
        <div key={i} className="bg-bg-deep border border-border-subtle rounded p-2 animate-pulse">
          <div className="h-3 bg-bg-elevated rounded w-3/4 mb-2" />
          <div className="h-2 bg-bg-elevated rounded w-1/2 mb-1" />
          <div className="h-2 bg-bg-elevated rounded w-1/3" />
        </div>
      ))}
    </div>
  );
}

export function LibraryBrowser() {
  const { results, search, select, selected, loadCategories, categories, selectedCategory, setCategory, loading, error, detailLoading, importFile, placeSymbol, placeFootprint, clearSelection, clearError, saveSymbolDef, editingFootprintId, setEditingFootprint, saveFootprintDef } = useLibraryStore();
  const [searchText, setSearchText] = useState("");
  const [showSymbolEditor, setShowSymbolEditor] = useState(false);
  const [editComponent, setEditComponent] = useState<Component | null>(null);
  const [focusedIndex, setFocusedIndex] = useState(-1);
  const listRef = useRef<HTMLDivElement>(null);
  const itemRefs = useRef<(HTMLDivElement | null)[]>([]);
  const [downloadProgress, setDownloadProgress] = useState<DownloadProgress | null>(null);
  const [downloading, setDownloading] = useState(false);
  const [showNewComponent, setShowNewComponent] = useState(false);
  const [newCompName, setNewCompName] = useState("");
  const [newCompCat, setNewCompCat] = useState("");

  useEffect(() => {
    loadCategories();
    search("");
  }, []);

  useEffect(() => {
    const unlisten = listen<DownloadProgress>("kicad-download-progress", (e) => {
      setDownloadProgress(e.payload);
      if (e.payload.phase === "done") {
        setDownloading(false);
        toast.success(e.payload.message);
        search(searchText);
      } else if (e.payload.phase === "cancelled") {
        setDownloading(false);
        toast.error("Download cancelled");
      }
    });
    return () => { unlisten.then((fn) => fn()); };
  }, [search, searchText]);

  const handleDownloadLibraries = async () => {
    setDownloading(true);
    setDownloadProgress({ phase: "start", current: 0, total: 1, message: "Starting download..." });
    try {
      const result = await invoke<string>("download_kicad_libraries");
      toast.success(result);
      setDownloading(false);
      setDownloadProgress(null);
      search(searchText);
    } catch (e) {
      toast.error(String(e));
      setDownloading(false);
      setDownloadProgress(null);
    }
  };

  useEffect(() => {
    itemRefs.current = itemRefs.current.slice(0, results.length);
  }, [results]);

  useEffect(() => {
    if (focusedIndex >= 0 && itemRefs.current[focusedIndex]) {
      itemRefs.current[focusedIndex]?.scrollIntoView({ block: "nearest" });
    }
  }, [focusedIndex]);

  const handleSearch = useCallback(
    (text: string) => {
      setSearchText(text);
      setFocusedIndex(-1);
      const timer = setTimeout(() => search(text), 300);
      return () => clearTimeout(timer);
    },
    [search]
  );

  const handleKeyDown = useCallback((e: React.KeyboardEvent) => {
    if (results.length === 0) return;
    if (e.key === "ArrowDown") {
      e.preventDefault();
      setFocusedIndex((prev) => (prev < results.length - 1 ? prev + 1 : 0));
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      setFocusedIndex((prev) => (prev > 0 ? prev - 1 : results.length - 1));
    } else if (e.key === "Enter" && focusedIndex >= 0) {
      select(results[focusedIndex].id);
    }
  }, [results, focusedIndex, select]);

  const handleImport = async () => {
    try {
      const selected = await openDialog({
        multiple: false,
        filters: [
          { name: "KiCad Symbol Library", extensions: ["kicad_sym"] },
          { name: "CSV BOM", extensions: ["csv"] },
          { name: "All supported", extensions: ["kicad_sym", "csv"] },
        ],
      });
      if (!selected) return;
      const format = selected.endsWith(".csv") ? "csv_bom" : "kicad_sym";
      const count = await importFile(selected, format);
      alert(`Imported ${count} component(s)`);
      search(searchText);
    } catch (e) {
      console.error("Import failed:", e);
    }
  };

  const handleDragStart = (componentId: string) => (e: React.DragEvent) => {
    e.dataTransfer.setData("componentId", componentId);
  };

  const handleEditSymbol = (comp: Component) => {
    setEditComponent(comp);
    setShowSymbolEditor(true);
  };

  const handleSaveSymbol = async (symbol: SymbolDef) => {
    if (editComponent) {
      await saveSymbolDef(editComponent.id, symbol);
      await select(editComponent.id);
    }
    setShowSymbolEditor(false);
    setEditComponent(null);
  };

  const handleEditFootprint = (compId: string) => {
    setEditingFootprint(compId);
  };

  const handleSaveFootprint = async (footprint: FootprintDef) => {
    if (editingFootprintId) {
      await saveFootprintDef(editingFootprintId, footprint);
      await select(editingFootprintId);
    }
    setEditingFootprint(null);
  };

  const handleCreateComponent = async () => {
    if (!newCompName.trim()) { toast.error("Component name required"); return; }
    try {
      const id = await invoke<string>("create_component", {
        name: newCompName.trim(),
        description: "",
        manufacturer: null,
        category: newCompCat.trim() || null,
      });
      const comp = await invoke<Component | null>("get_component_detail", { id });
      if (comp) {
        setEditComponent(comp);
        setShowNewComponent(false);
        setShowSymbolEditor(true);
        setNewCompName("");
        setNewCompCat("");
      }
    } catch (e) {
      toast.error(String(e));
    }
  };

  if (showNewComponent) {
    return (
      <div className="flex flex-col h-full p-4 space-y-3">
        <h3 className="text-xs font-semibold text-text-primary uppercase tracking-wider">New Component</h3>
        <div className="space-y-2">
          <label className="text-xs text-text-secondary">Name</label>
          <input
            autoFocus
            value={newCompName}
            onChange={(e) => setNewCompName(e.target.value)}
            placeholder="e.g. NE555 Timer"
            className="w-full px-2 py-1.5 rounded bg-bg-elevated border border-border text-text-primary text-xs outline-none focus:border-copper"
            onKeyDown={(e) => e.key === "Enter" && handleCreateComponent()}
          />
        </div>
        <div className="space-y-2">
          <label className="text-xs text-text-secondary">Category (optional)</label>
          <select
            value={newCompCat}
            onChange={(e) => setNewCompCat(e.target.value)}
            className="w-full px-2 py-1.5 rounded bg-bg-elevated border border-border text-text-primary text-xs outline-none focus:border-copper"
          >
            <option value="">None</option>
            {categories
              .filter((c, i, arr) => arr.findIndex((x) => x.name === c.name) === i && !c.parent_id)
              .sort((a, b) => a.name.localeCompare(b.name))
              .map((c) => (
                <option key={c.id} value={c.name}>{c.name}</option>
              ))}
          </select>
        </div>
        <div className="flex gap-2 pt-2">
          <button
            onClick={handleCreateComponent}
            className="flex-1 px-3 py-1.5 rounded bg-copper text-white text-xs font-medium hover:bg-copper-dark transition-colors"
          >
            Create & Edit Symbol
          </button>
          <button
            onClick={() => { setShowNewComponent(false); setNewCompName(""); setNewCompCat(""); }}
            className="px-3 py-1.5 rounded bg-bg-surface text-text-secondary text-xs border border-border hover:text-text-primary transition-colors"
          >
            Cancel
          </button>
        </div>
      </div>
    );
  }

  if (showSymbolEditor) {
    return (
      <div className="flex flex-col h-full">
        <SymbolEditor
          initial={editComponent?.symbol}
          onSave={handleSaveSymbol}
          onClose={() => { setShowSymbolEditor(false); setEditComponent(null); }}
          width={700}
          height={500}
        />
      </div>
    );
  }

  if (editingFootprintId) {
    return (
      <div className="flex flex-col h-full">
        <FootprintEditor
          componentId={editingFootprintId}
          initial={selected?.footprint}
          onSave={handleSaveFootprint}
          onClose={() => setEditingFootprint(null)}
        />
      </div>
    );
  }

  return (
    <div className="flex flex-1 h-full overflow-hidden">
      <div className="w-44 flex-shrink-0 border-r border-border flex flex-col bg-bg-surface">
        <div className="p-3 text-xs font-semibold text-text-muted uppercase tracking-wider border-b border-border">
          Categories
        </div>
        <div className="flex-1 overflow-y-auto p-2 space-y-0.5">
          <button
            onClick={() => setCategory(null)}
            className={`w-full flex items-center gap-1.5 text-left px-2 py-1 text-xs rounded transition-colors ${
              selectedCategory === null
                ? "bg-copper text-white"
                : "text-text-secondary hover:bg-bg-elevated hover:text-text-primary"
            }`}
          >
            <Folder size={12} />
            All Components
          </button>
          <CategoryTree
            categories={categories}
            selected={selectedCategory}
            onSelect={(categoryId) => setCategory(categoryId)}
          />
        </div>
      </div>
      <div className="flex flex-col flex-1 min-w-0">
        <div className="p-3 border-b border-border space-y-2">
          <div className="relative">
            <Search size={14} className="absolute left-2 top-2 text-text-muted" />
            <input
              type="text"
              placeholder="Search components..."
              value={searchText}
              onChange={(e) => handleSearch(e.target.value)}
              className="w-full bg-bg-deep border border-border rounded pl-7 pr-2 py-1.5 text-xs text-text-primary placeholder-text-muted focus:outline-none focus:border-info"
            />
          </div>
          <button
            onClick={handleImport}
            className="w-full flex items-center justify-center gap-1.5 px-2 py-1.5 bg-copper text-white text-xs rounded hover:bg-copper-dark transition-colors"
          >
            <Upload size={14} />
            Import Library
          </button>
          <button
            onClick={() => setShowNewComponent(true)}
            className="w-full flex items-center justify-center gap-1.5 px-2 py-1.5 bg-bg-elevated text-text-primary text-xs rounded border border-border hover:bg-bg-surface transition-colors"
          >
            <Plus size={14} />
            New Component
          </button>
          <button
            onClick={handleDownloadLibraries}
            disabled={downloading}
            className="w-full flex items-center justify-center gap-1.5 px-2 py-1.5 bg-bg-elevated text-text-primary text-xs rounded border border-border hover:bg-bg-surface disabled:opacity-40 disabled:cursor-not-allowed transition-colors"
          >
            <Download size={14} />
            {downloading ? "Downloading..." : "Download KiCad Libs"}
          </button>
          {downloading && downloadProgress && (
            <div className="space-y-1">
              <div className="h-1.5 bg-bg-deep rounded-full overflow-hidden">
                <div
                  className="h-full bg-copper rounded-full transition-all duration-300"
                  style={{ width: `${downloadProgress.total > 0 ? (downloadProgress.current / downloadProgress.total) * 100 : 0}%` }}
                />
              </div>
              <p className="text-[10px] text-text-muted text-center truncate">{downloadProgress.message}</p>
            </div>
          )}
        </div>
        <div className="flex-1 overflow-y-auto p-2" ref={listRef} tabIndex={0} onKeyDown={handleKeyDown}>
          {error && (
            <div className="mx-3 mb-2 px-2 py-1 text-xs bg-red-900/80 text-red-200 rounded border border-red-700 flex items-center gap-2">
              <span className="flex-1">{error}</span>
              <button
                onClick={() => { clearError(); search(searchText); }}
                className="text-red-200 hover:text-white shrink-0 text-xs font-medium"
              >
                Retry
              </button>
            </div>
          )}
          {loading ? (
            <LoadingSkeleton />
          ) : results.length === 0 ? (
            <div className="text-xs text-text-muted text-center py-8">
              No components found
            </div>
          ) : (
            <div className="grid grid-cols-2 gap-2">
              {results.map((comp, i) => (
                <div
                  key={comp.id}
                  ref={(el) => { itemRefs.current[i] = el; }}
                  draggable
                  onDragStart={handleDragStart(comp.id)}
                  onClick={() => { select(comp.id); setFocusedIndex(i); }}
                  className={`bg-bg-deep border rounded p-2 cursor-pointer hover:border-border hover:bg-bg-elevated transition-colors ${
                    selected?.id === comp.id ? "border-copper" : focusedIndex === i ? "border-info" : "border-border-subtle"
                  }`}
                >
                  <div className="text-xs font-medium text-text-primary truncate">{comp.name}</div>
                  <div className="text-[10px] text-text-muted truncate">{comp.category}</div>
                  <div className="text-[10px] text-text-muted truncate">{comp.manufacturer}</div>
                </div>
              ))}
            </div>
          )}
        </div>
      </div>

      {(selected || detailLoading) && (
        <div className="w-80 border-l border-border bg-bg-surface flex flex-col overflow-hidden">
          <div className="flex-1 overflow-y-auto">
            <ComponentDetail
              component={selected}
              loading={detailLoading}
              error={error}
              onPlaceSymbol={placeSymbol}
              onPlaceFootprint={placeFootprint}
              onEditFootprint={handleEditFootprint}
              onClose={clearSelection}
              onEditSymbol={handleEditSymbol}
            />
          </div>
        </div>
      )}
    </div>
  );
}
