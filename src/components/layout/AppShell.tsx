import { lazy, Suspense } from "react";
import { Toolbar } from "./Toolbar";
import { Sidebar } from "./Sidebar";
import { StatusBar } from "./StatusBar";
import { SchematicCanvas } from "../schematic/SchematicCanvas";
import { PcbCanvas } from "../pcb/PcbCanvas";
import { WelcomeScreen } from "../common/WelcomeScreen";
import { ToastSetup } from "../common/Toast";
import { ErrorBoundary } from "../common/ErrorBoundary";
import { MenuBar } from "./MenuBar";
import { useUIStore } from "../../stores/uiStore";
import { useProjectStore } from "../../stores/projectStore";

const Preview3D = lazy(() =>
  import("../preview/Preview3D").then((module) => ({ default: module.Preview3D }))
);

export function AppShell() {
  const { mode } = useUIStore();
  const current = useProjectStore((s) => s.current);

  const renderCanvas = () => {
    const projectKey = current?.id;
    switch (mode) {
      case "schematic":
        return <SchematicCanvas key={projectKey} />;
      case "pcb":
        return <PcbCanvas key={projectKey} />;
      case "preview":
        return (
          <Suspense fallback={<div className="h-full flex items-center justify-center text-text-muted">Loading 3D preview...</div>}>
            <Preview3D key={projectKey} />
          </Suspense>
        );
      default:
        return null;
    }
  };

  if (!current) {
    return (
      <div className="h-screen flex flex-col bg-bg-deep">
        <MenuBar />
        <Toolbar />
        <WelcomeScreen />
        <StatusBar />
        <ToastSetup />
      </div>
    );
  }

  return (
    <div className="h-screen flex flex-col bg-bg-deep">
      <MenuBar />
      <Toolbar />
      <div className="flex flex-1 overflow-hidden">
        <div className="flex-1 relative overflow-hidden">
          <ErrorBoundary label="Canvas error">
            {renderCanvas()}
          </ErrorBoundary>
        </div>
        <Sidebar />
      </div>
      <StatusBar />
      <ToastSetup />
    </div>
  );
}
