import React, { useEffect, useState, useCallback, useRef } from "react";
import { Search, Upload, X, Folder, Edit3, Plus } from "lucide-react";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { useLibraryStore } from "../../stores/libraryStore";
import { ComponentDetail } from "./ComponentDetail";
import { SymbolEditor } from "./SymbolEditor";
import { FootprintEditor } from "./FootprintEditor";
import { Component, SymbolDef, FootprintDef } from "../../types/library";

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

  useEffect(() => {
    loadCategories();
    search("");
  }, []);

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
          {categories.map((cat) => (
            <button
              key={cat.id}
              onClick={() => setCategory(cat.name)}
              className={`w-full flex items-center gap-1.5 text-left px-2 py-1 text-xs rounded transition-colors ${
                selectedCategory === cat.name
                  ? "bg-copper text-white"
                  : "text-text-secondary hover:bg-bg-elevated hover:text-text-primary"
              }`}
            >
              <Folder size={12} />
              {cat.name}
            </button>
          ))}
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
            onClick={() => { setEditComponent(null); setShowSymbolEditor(true); }}
            className="w-full flex items-center justify-center gap-1.5 px-2 py-1.5 bg-bg-elevated text-text-primary text-xs rounded border border-border hover:bg-bg-surface transition-colors"
          >
            <Plus size={14} />
            New Symbol
          </button>
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
