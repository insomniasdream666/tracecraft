import React, { useMemo } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Library, Box, Layers, Sparkles, PanelRightClose, PanelRightOpen, Activity, FoldVertical, Zap, GitBranch, Braces, Shield } from "lucide-react";
import { useUIStore } from "../../stores/uiStore";
import { usePCBStore } from "../../stores/pcbStore";
import { LibraryBrowser } from "../library/LibraryBrowser";
import { PropertyPanel } from "../common/PropertyPanel";
import { AIPanel } from "../ai/AIPanel";
import { LayerPanel } from "../pcb/LayerPanel";
import { BoardProperties } from "../pcb/BoardProperties";
import { SIPanel } from "../pcb/SIPanel";
import { FlexZonePanel } from "../pcb/FlexZonePanel";
import { ImpedancePanel } from "../pcb/ImpedancePanel";
import { LengthTuningPanel } from "../pcb/LengthTuningPanel";
import { BusRoutePanel } from "../pcb/BusRoutePanel";
import { ViaStitchPanel } from "../pcb/ViaStitchPanel";
import { StackupVisualizer } from "../pcb/StackupVisualizer";

const tabs = [
  { id: "properties" as const, icon: <Box size={16} />, label: "Properties" },
  { id: "library" as const, icon: <Library size={16} />, label: "Library" },
  { id: "layers" as const, icon: <Layers size={16} />, label: "Layers" },
  { id: "ai" as const, icon: <Sparkles size={16} />, label: "AI" },
  { id: "si" as const, icon: <Activity size={16} />, label: "SI" },
  { id: "flex" as const, icon: <FoldVertical size={16} />, label: "Flex" },
  { id: "impedance" as const, icon: <Zap size={16} />, label: "Z₀" },
  { id: "tuning" as const, icon: <Braces size={16} />, label: "Tune" },
  { id: "bus" as const, icon: <GitBranch size={16} />, label: "Bus" },
  { id: "stitch" as const, icon: <Shield size={16} />, label: "Stitch" },
  { id: "stackup" as const, icon: <Layers size={16} />, label: "Stackup" },
];

export function Sidebar() {
  const { sidebarTab, setSidebarTab, sidebarOpen, toggleSidebar, mode } = useUIStore();
  const { board, selectedIds, loadBoard } = usePCBStore();

  const selectedProps = useMemo(() => {
    if (mode !== "pcb" || !board || selectedIds.size !== 1) return null;

    const id = [...selectedIds][0];

    if (id.startsWith("trace-")) {
      const traceId = parseInt(id.replace("trace-", ""), 10);
      const trace = board.traces[traceId];
      if (!trace) return null;
      const layerOptions = board.layers.layers.map((l) => ({
        label: l.name,
        value: String(l.id),
      }));
      return {
        title: `Trace #${traceId}`,
        properties: [
          {
            label: "Width", value: trace.width, type: "number" as const,
            onChange: (v: string | number | boolean) => {
              invoke("set_trace_width", { id: traceId, width: Number(v) });
            },
          },
          {
            label: "Layer", value: String(trace.layer), type: "select" as const, options: layerOptions,
            onChange: (v: string | number | boolean) => {
              invoke("set_trace_layer", { id: traceId, layer: Number(v) });
            },
          },
          {
            label: "Net", value: trace.net_id != null ? String(trace.net_id) : "", type: "text" as const,
            onChange: (v: string | number | boolean) => {
              invoke("set_trace_net", { id: traceId, netId: v ? Number(v) : null });
            },
          },
        ],
      };
    }

    const fpId = parseInt(id, 10);
    if (!isNaN(fpId) && board.footprints[fpId]) {
      const fp = board.footprints[fpId];
      return {
        title: `Footprint: ${fp.refdes}`,
        properties: [
          {
            label: "Refdes", value: fp.refdes, type: "text" as const,
            onChange: (v: string | number | boolean) => {
              invoke("set_footprint_refdes", { id: fpId, refdes: String(v) });
            },
          },
          {
            label: "X", value: fp.position.x, type: "number" as const,
            onChange: (v: string | number | boolean) => {
              invoke("move_footprint", { id: fpId, x: Number(v), y: fp.position.y });
            },
          },
          {
            label: "Y", value: fp.position.y, type: "number" as const,
            onChange: (v: string | number | boolean) => {
              invoke("move_footprint", { id: fpId, x: fp.position.x, y: Number(v) });
            },
          },
          {
            label: "Rotation", value: fp.rotation, type: "number" as const,
            onChange: (v: string | number | boolean) => {
              invoke("rotate_footprint", { id: fpId, rotation: Number(v) });
            },
          },
        ],
      };
    }

    const viaId = parseInt(id.replace("via-", ""), 10);
    if (!isNaN(viaId) && board.vias[viaId]) {
      const via = board.vias[viaId];
      return {
        title: `Via #${viaId}`,
        properties: [
          {
            label: "X", value: via.position.x, type: "number" as const,
            onChange: (v: string | number | boolean) => {
              invoke("move_via", { id: viaId, x: Number(v), y: via.position.y });
            },
          },
          {
            label: "Y", value: via.position.y, type: "number" as const,
            onChange: (v: string | number | boolean) => {
              invoke("move_via", { id: viaId, x: via.position.x, y: Number(v) });
            },
          },
          {
            label: "Diameter", value: via.diameter, type: "number" as const,
            onChange: (v: string | number | boolean) => {
              invoke("set_via_diameter", { id: viaId, diameter: Number(v) });
            },
          },
          {
            label: "Drill", value: via.drill, type: "number" as const,
            onChange: (v: string | number | boolean) => {
              invoke("set_via_drill", { id: viaId, drill: Number(v) });
            },
          },
          {
            label: "Net", value: via.net_id != null ? String(via.net_id) : "", type: "text" as const,
            onChange: (v: string | number | boolean) => {
              invoke("set_via_net", { id: viaId, netId: v ? Number(v) : null });
            },
          },
        ],
      };
    }

    return null;
  }, [board, selectedIds, mode]);

  if (!sidebarOpen) {
    return (
      <div className="flex flex-col items-center w-8 bg-bg-surface border-l border-border py-1 gap-1">
        <button
          onClick={toggleSidebar}
          title="Open sidebar"
          className="w-6 h-6 flex items-center justify-center rounded text-text-secondary hover:text-text-primary hover:bg-bg-elevated transition-colors"
        >
          <PanelRightOpen size={14} />
        </button>
        {tabs.map((tab) => (
          <button
            key={tab.id}
            title={tab.label}
            onClick={() => { setSidebarTab(tab.id); toggleSidebar(); }}
            className={`w-6 h-6 flex items-center justify-center rounded transition-colors ${
              sidebarTab === tab.id
                ? "text-copper"
                : "text-text-muted hover:text-text-secondary"
            }`}
          >
            {tab.icon}
          </button>
        ))}
      </div>
    );
  }

  return (
    <div className="w-80 bg-bg-surface border-l border-border flex flex-col transition-all duration-200">
      <div className="flex items-center border-b border-border pr-1">
        {tabs.map((tab) => (
          <button
            key={tab.id}
            onClick={() => setSidebarTab(tab.id)}
            className={`flex items-center gap-1.5 px-3 py-2 text-xs font-medium transition-colors flex-1 justify-center
              ${sidebarTab === tab.id
                ? "text-copper border-b-2 border-copper"
                : "text-text-secondary hover:text-text-primary"
              }`}
          >
            {tab.icon}
            {tab.label}
          </button>
        ))}
        <button
          onClick={toggleSidebar}
          title="Close sidebar"
          className="w-6 h-6 flex items-center justify-center rounded text-text-muted hover:text-text-primary hover:bg-bg-elevated transition-colors ml-1"
        >
          <PanelRightClose size={14} />
        </button>
      </div>
      <div className="flex-1 overflow-y-auto">
        {sidebarTab === "properties" && (
          selectedProps ? (
            <PropertyPanel title={selectedProps.title} properties={selectedProps.properties} />
          ) : mode === "pcb" ? (
            <BoardProperties />
          ) : (
            <PropertyPanel title="Properties" properties={[]} />
          )
        )}
        {sidebarTab === "library" && <LibraryBrowser />}
        {sidebarTab === "layers" && <LayerPanel />}
        {sidebarTab === "ai" && <AIPanel />}
        {sidebarTab === "si" && mode === "pcb" && <SIPanel />}
        {sidebarTab === "flex" && mode === "pcb" && <FlexZonePanel />}
        {sidebarTab === "impedance" && mode === "pcb" && <ImpedancePanel />}
        {sidebarTab === "tuning" && mode === "pcb" && <LengthTuningPanel />}
        {sidebarTab === "bus" && mode === "pcb" && <BusRoutePanel />}
        {sidebarTab === "stitch" && mode === "pcb" && <ViaStitchPanel />}
        {sidebarTab === "stackup" && mode === "pcb" && <StackupVisualizer initialStackup={board?.layers || null} onSave={async (s) => { try { await invoke("set_stackup", { stackup: s }); await loadBoard(); } catch {} }} />}
      </div>
    </div>
  );
}
