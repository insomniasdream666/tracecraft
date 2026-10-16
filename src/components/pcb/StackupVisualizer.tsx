import React, { useEffect, useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { usePCBStore } from "../../stores/pcbStore";
import { LayerStack, Dielectric, LayerDef } from "../../types/pcb";
import toast from "react-hot-toast";

interface StackupVisualizerProps {
  initialStackup?: LayerStack | null;
  onSave?: (stackup: LayerStack) => void;
  editable?: boolean;
}

export function StackupVisualizer({ initialStackup, onSave, editable = true }: StackupVisualizerProps) {
  const { board, loadBoard } = usePCBStore();
  const [stackup, setStackup] = useState<LayerStack | null>(initialStackup || null);
  const [editingLayer, setEditingLayer] = useState<number | null>(null);
  const [editForm, setEditForm] = useState<{ name: string; color: string; kind: "Top" | "Bottom" | "Inner" | "SolderMask" | "Silkscreen" | "Paste" }>({ name: "", color: "", kind: "Top" });
  const [addingDielectric, setAddingDielectric] = useState(false);
  const [dielectricForm, setDielectricForm] = useState({ name: "FR-4", er: 4.5, thickness_mm: 0.1, loss_tangent: 0.02 });

  useEffect(() => {
    if (initialStackup) {
      setStackup(initialStackup);
    } else if (board?.layers) {
      setStackup(board.layers);
    }
  }, [initialStackup, board]);

  const updateStackup = useCallback((newStackup: LayerStack) => {
    setStackup(newStackup);
    if (onSave) onSave(newStackup);
  }, [onSave]);

  const handleLayerEdit = (layer: LayerDef) => {
    setEditingLayer(layer.id);
    setEditForm({ name: layer.name, color: layer.color, kind: layer.kind });
  };

  const saveLayerEdit = async () => {
    if (!stackup || editingLayer === null) return;
    const newLayers = stackup.layers.map(l =>
      l.id === editingLayer ? { ...l, ...editForm } : l
    );
    updateStackup({ ...stackup, layers: newLayers });
    setEditingLayer(null);
  };

  const addDielectric = async () => {
    if (!stackup) return;
    const newDielectric: Dielectric = { ...dielectricForm };
    updateStackup({ ...stackup, dielectrics: [...stackup.dielectrics, newDielectric] });
    setAddingDielectric(false);
    setDielectricForm({ name: "FR-4", er: 4.5, thickness_mm: 0.1, loss_tangent: 0.02 });
  };

  const removeDielectric = (index: number) => {
    if (!stackup) return;
    const newDielectrics = stackup.dielectrics.filter((_, i) => i !== index);
    updateStackup({ ...stackup, dielectrics: newDielectrics });
  };

  const addSignalLayer = () => {
    if (!stackup) return;
    const newId = Math.max(...stackup.layers.map(l => l.id)) + 1;
    const newLayer: LayerDef = {
      id: newId,
      name: `Signal ${newId}`,
      kind: "Inner" as const,
      color: "#D47A2F",
    };
    updateStackup({ ...stackup, layers: [...stackup.layers, newLayer] });
  };

  const removeLayer = (id: number) => {
    if (!stackup) return;
    const newLayers = stackup.layers.filter(l => l.id !== id);
    updateStackup({ ...stackup, layers: newLayers });
  };

  const saveStackup = async () => {
    if (!stackup) return;
    try {
      await invoke("set_stackup", { stackup });
      toast.success("Stackup saved");
      await loadBoard();
    } catch (e: any) {
      toast.error(`Failed: ${e}`);
    }
  };

  const importKiCadStackup = async () => {
    try {
      const path = await invoke<string>("import_stackup_file", {});
      if (path) {
        const result = await invoke<LayerStack>("parse_kicad_stackup", { path });
        setStackup(result);
        if (onSave) onSave(result);
        toast.success("KiCad stackup imported");
      }
    } catch (e: any) {
      toast.error(`Failed: ${e}`);
    }
  };

  if (!stackup) return <div className="p-3 text-xs text-text-muted">No stackup loaded</div>;

  const sortedLayers = [...stackup.layers].sort((a, b) => a.id - b.id);
  const totalThickness = stackup.dielectrics.reduce((sum, d) => sum + d.thickness_mm, 0);

  return (
    <div className="p-3 text-xs text-text-primary space-y-3">
      <div className="flex items-center justify-between">
        <h3 className="font-semibold text-sm">Stackup Visualizer</h3>
        {editable && (
          <div className="flex gap-1">
            <button onClick={addSignalLayer} className="px-2 py-1 bg-copper/20 border border-copper/30 rounded text-xs hover:bg-copper/30">
              + Signal Layer
            </button>
            <button onClick={() => setAddingDielectric(true)} className="px-2 py-1 bg-copper/20 border border-copper/30 rounded text-xs hover:bg-copper/30">
              + Dielectric
            </button>
            <button onClick={saveStackup} className="px-2 py-1 bg-copper text-white rounded text-xs hover:opacity-90">
              Save
            </button>
            <button onClick={importKiCadStackup} className="px-2 py-1 bg-bg-elevated border border-border rounded text-xs hover:bg-bg-surface">
              Import KiCad
            </button>
          </div>
        )}
      </div>

      {/* Total thickness */}
      <div className="bg-bg-elevated rounded p-2 text-center">
        <span className="font-mono text-lg text-copper">{totalThickness.toFixed(3)} mm</span>
        <span className="text-text-muted ml-2">Total board thickness</span>
      </div>

      {/* Stackup visualization - top to bottom */}
      <div className="space-y-0.5">
        {sortedLayers.map((layer, layerIdx) => {
          const dielectricBelow = stackup.dielectrics[layerIdx] || null;
          return (
            <div key={layer.id} className="space-y-0.5">
              {/* Signal/Copper layer */}
              <div
                className="flex items-center gap-2 px-2 py-1 rounded"
                style={{ backgroundColor: layer.color + "20", borderLeft: `3px solid ${layer.color}` }}
              >
                <div style={{ width: 12, height: 12, backgroundColor: layer.color, borderRadius: 2 }} />
                <span className="font-mono text-xs">{layer.name}</span>
                <span className="text-[10px] text-text-muted px-1.5 py-0.5 bg-bg-surface rounded">{layer.kind}</span>
                <span className="text-[10px] text-text-muted ml-auto">{layer.id}</span>
                {editable && (
                  <div className="flex gap-1">
                    <button onClick={() => handleLayerEdit(layer)} className="text-copper hover:text-copper/70 text-[10px]">Edit</button>
                    <button onClick={() => removeLayer(layer.id)} className="text-red-500 hover:text-red-400 text-[10px]">Del</button>
                  </div>
                )}
              </div>

              {/* Dielectric layer below */}
              {dielectricBelow && (
                <div
                  className="flex items-center gap-2 px-2 py-1 rounded bg-blue-900/20 border-l-3 border-blue-500"
                  style={{ opacity: 0.7 }}
                >
                  <div style={{ width: 12, height: 12, backgroundColor: "#3B82F6", borderRadius: 2 }} />
                  <span className="font-mono text-xs">{dielectricBelow.name}</span>
                  <span className="text-[10px] text-text-muted">
                    Er={dielectricBelow.er}, {dielectricBelow.thickness_mm} mm, tanδ={dielectricBelow.loss_tangent}
                  </span>
                  <span className="text-[10px] text-text-muted ml-auto">{dielectricBelow.thickness_mm.toFixed(3)} mm</span>
                  {editable && (
                    <button onClick={() => removeDielectric(layerIdx)} className="text-red-500 hover:text-red-400 text-[10px]">Del</button>
                  )}
                </div>
              )}
            </div>
          );
        })}

        {/* Last dielectric if more dielectrics than signal layers */}
        {stackup.dielectrics.length > sortedLayers.length && (
          <div className="flex items-center gap-2 px-2 py-1 rounded bg-blue-900/20 border-l-3 border-blue-500" style={{ opacity: 0.7 }}>
            <div style={{ width: 12, height: 12, backgroundColor: "#3B82F6", borderRadius: 2 }} />
            <span className="font-mono text-xs">{stackup.dielectrics[sortedLayers.length].name}</span>
            <span className="text-[10px] text-text-muted">
              Er={stackup.dielectrics[sortedLayers.length].er}, {stackup.dielectrics[sortedLayers.length].thickness_mm} mm
            </span>
            <span className="text-[10px] text-text-muted ml-auto">{stackup.dielectrics[sortedLayers.length].thickness_mm.toFixed(3)} mm</span>
            {editable && (
              <button onClick={() => removeDielectric(sortedLayers.length)} className="text-red-500 hover:text-red-400 text-[10px]">Del</button>
            )}
          </div>
        )}
      </div>

      {/* Edit forms */}
      {editingLayer !== null && (
        <div className="bg-bg-elevated rounded p-2 space-y-2 border border-copper/30">
          <div className="font-medium text-copper">Edit Layer {editingLayer}</div>
          <label className="flex flex-col gap-1 text-[11px]">
            <span>Name</span>
            <input value={editForm.name} onChange={e => setEditForm({...editForm, name: e.target.value})} className="px-2 py-1 bg-bg-surface border border-border rounded text-xs" />
          </label>
          <label className="flex flex-col gap-1 text-[11px]">
            <span>Color</span>
            <input type="color" value={editForm.color} onChange={e => setEditForm({...editForm, color: e.target.value})} className="w-10 h-6" />
          </label>
          <label className="flex flex-col gap-1 text-[11px]">
            <span>Kind</span>
            <select value={editForm.kind} onChange={e => setEditForm({...editForm, kind: e.target.value as "Top" | "Bottom" | "Inner" | "SolderMask" | "Silkscreen" | "Paste"})} className="px-2 py-1 bg-bg-surface border border-border rounded text-xs">
              <option value="Top">Top</option>
              <option value="Bottom">Bottom</option>
              <option value="Inner">Inner</option>
              <option value="SolderMask">SolderMask</option>
              <option value="Silkscreen">Silkscreen</option>
              <option value="Paste">Paste</option>
            </select>
          </label>
          <div className="flex gap-2">
            <button onClick={saveLayerEdit} className="flex-1 px-2 py-1 bg-copper text-white rounded text-xs">Save</button>
            <button onClick={() => setEditingLayer(null)} className="px-2 py-1 bg-bg-surface border border-border rounded text-xs">Cancel</button>
          </div>
        </div>
      )}

      {addingDielectric && (
        <div className="bg-bg-elevated rounded p-2 space-y-2 border border-blue-500/30">
          <div className="font-medium text-blue-400">Add Dielectric</div>
          <label className="flex flex-col gap-1 text-[11px]">
            <span>Name</span>
            <input value={dielectricForm.name} onChange={e => setDielectricForm({...dielectricForm, name: e.target.value})} className="px-2 py-1 bg-bg-surface border border-border rounded text-xs" />
          </label>
          <label className="flex flex-col gap-1 text-[11px]">
            <span>Er (Dk)</span>
            <input type="number" step="0.1" value={dielectricForm.er} onChange={e => setDielectricForm({...dielectricForm, er: Number(e.target.value)})} className="px-2 py-1 bg-bg-surface border border-border rounded text-xs" />
          </label>
          <label className="flex flex-col gap-1 text-[11px]">
            <span>Thickness (mm)</span>
            <input type="number" step="0.01" value={dielectricForm.thickness_mm} onChange={e => setDielectricForm({...dielectricForm, thickness_mm: Number(e.target.value)})} className="px-2 py-1 bg-bg-surface border border-border rounded text-xs" />
          </label>
          <label className="flex flex-col gap-1 text-[11px]">
            <span>Loss Tangent</span>
            <input type="number" step="0.001" value={dielectricForm.loss_tangent} onChange={e => setDielectricForm({...dielectricForm, loss_tangent: Number(e.target.value)})} className="px-2 py-1 bg-bg-surface border border-border rounded text-xs" />
          </label>
          <div className="flex gap-2">
            <button onClick={addDielectric} className="flex-1 px-2 py-1 bg-blue-600 text-white rounded text-xs">Add</button>
            <button onClick={() => setAddingDielectric(false)} className="px-2 py-1 bg-bg-surface border border-border rounded text-xs">Cancel</button>
          </div>
        </div>
      )}

      {/* Summary table */}
      <div className="bg-bg-surface rounded p-2 max-h-48 overflow-y-auto">
        <table className="w-full text-[10px] border-collapse">
          <thead>
            <tr className="border-b border-border text-left text-text-muted">
              <th className="p-1">#</th>
              <th className="p-1">Layer</th>
              <th className="p-1">Type</th>
              <th className="p-1">Thickness</th>
              <th className="p-1">Er / Color</th>
            </tr>
          </thead>
          <tbody>
            {sortedLayers.map((layer, layerIdx) => (
              <React.Fragment key={layer.id}>
                <tr className="border-b border-border/30">
                  <td className="p-1 font-mono">{layer.id}</td>
                  <td className="p-1">{layer.name}</td>
                  <td className="p-1"><span className="text-[9px] px-1 py-0.5 bg-bg-elevated rounded">{layer.kind}</span></td>
                  <td className="p-1 font-mono">~0.035 mm</td>
                  <td className="p-1"><span style={{ display: "inline-block", width: 8, height: 8, backgroundColor: layer.color, borderRadius: 1, verticalAlign: "middle", marginRight: 4 }} /></td>
                </tr>
                {stackup.dielectrics[layerIdx] && (
                  <tr className="border-b border-border/30 bg-blue-900/10">
                    <td className="p-1">—</td>
                    <td className="p-1">{stackup.dielectrics[layerIdx].name}</td>
                    <td className="p-1"><span className="text-[9px] px-1 py-0.5 bg-blue-900/30 rounded">Dielectric</span></td>
                    <td className="p-1 font-mono">{stackup.dielectrics[layerIdx].thickness_mm} mm</td>
                    <td className="p-1">Er: {stackup.dielectrics[layerIdx].er}</td>
                  </tr>
                )}
              </React.Fragment>
            ))}
            {stackup.dielectrics.length > sortedLayers.length && (
              <tr>
                <td className="p-1">—</td>
                <td className="p-1">{stackup.dielectrics[sortedLayers.length].name}</td>
                <td className="p-1"><span className="text-[9px] px-1 py-0.5 bg-blue-900/30 rounded">Dielectric</span></td>
                <td className="p-1 font-mono">{stackup.dielectrics[sortedLayers.length].thickness_mm} mm</td>
                <td className="p-1">Er: {stackup.dielectrics[sortedLayers.length].er}</td>
              </tr>
            )}
          </tbody>
        </table>
      </div>
    </div>
  );
}