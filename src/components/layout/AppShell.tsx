import React from "react";
import { Toolbar } from "./Toolbar";
import { Sidebar } from "./Sidebar";
import { StatusBar } from "./StatusBar";
import { SchematicCanvas } from "../schematic/SchematicCanvas";
import { PcbCanvas } from "../pcb/PcbCanvas";
import { Preview3D } from "../preview/Preview3D";
import { useUIStore } from "../../stores/uiStore";

export function AppShell() {
  const { mode } = useUIStore();

  const renderCanvas = () => {
    switch (mode) {
      case "schematic":
        return <SchematicCanvas />;
      case "pcb":
        return <PcbCanvas />;
      case "preview":
        return <Preview3D />;
      default:
        return null;
    }
  };

  return (
    <div className="h-screen flex flex-col bg-bg-deep">
      <Toolbar />
      <div className="flex flex-1 overflow-hidden">
        <div className="flex-1 relative overflow-hidden">
          {renderCanvas()}
        </div>
        <Sidebar />
      </div>
      <StatusBar />
    </div>
  );
}
