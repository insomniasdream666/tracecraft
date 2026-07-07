import React, { useState } from "react";
import { File, FolderOpen, CircuitBoard } from "lucide-react";
import { useProjectStore } from "../../stores/projectStore";
import { open as openDialog, save as saveDialog } from "@tauri-apps/plugin-dialog";
import toast from "react-hot-toast";

export function WelcomeScreen() {
  const { create, save } = useProjectStore();
  const [showNew, setShowNew] = useState(false);
  const [name, setName] = useState("");

  const handleCreate = async () => {
    if (!name.trim()) return;
    const projectName = name.trim();

    const path = await saveDialog({
      defaultPath: `${projectName}.tcproj`,
      filters: [{ name: "TraceCraft Project", extensions: ["tcproj"] }],
    });

    try {
      await create(projectName);
      if (path) {
        await save(path);
        toast.success(`Project saved to ${path.split("/").pop()}`);
      } else {
        toast.success(`Project "${projectName}" created (not yet saved)`);
      }
    } catch (e) {
      toast.error(String(e));
    }
  };

  const handleOpen = async () => {
    const path = await openDialog({
      filters: [{ name: "TraceCraft Project", extensions: ["tcproj"] }],
    });
    if (path) {
      const { open } = useProjectStore.getState();
      await open(path);
      toast.success("Project opened");
    }
  };

  return (
    <div className="flex-1 flex items-center justify-center bg-bg-canvas">
      <div className="flex flex-col items-center gap-8 max-w-md text-center px-8">
        <div className="w-32 h-32 rounded-2xl bg-gradient-to-br from-copper to-copper-dark flex items-center justify-center shadow-lg shadow-copper/20">
          <CircuitBoard size={56} className="text-white" />
        </div>

        <div>
          <h1 className="text-2xl font-bold text-text-primary">TraceCraft</h1>
          <p className="text-sm text-text-muted mt-1">AI-assisted PCB design tool</p>
        </div>

        {!showNew ? (
          <div className="flex flex-col gap-3 w-64">
            <button
              onClick={() => setShowNew(true)}
              className="flex items-center justify-center gap-2 px-4 py-2.5 rounded-lg bg-copper text-white text-sm font-medium hover:bg-copper-dark transition-colors"
            >
              <File size={16} />
              New Project
            </button>
            <button
              onClick={handleOpen}
              className="flex items-center justify-center gap-2 px-4 py-2.5 rounded-lg bg-bg-surface text-text-primary text-sm font-medium border border-border hover:bg-bg-elevated transition-colors"
            >
              <FolderOpen size={16} />
              Open Project
            </button>
          </div>
        ) : (
          <form
            onSubmit={(e) => { e.preventDefault(); handleCreate(); }}
            className="flex flex-col gap-3 w-64"
          >
            <input
              autoFocus
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="Project name..."
              className="px-3 py-2 rounded-lg bg-bg-elevated border border-border text-text-primary text-sm placeholder-text-muted outline-none focus:border-copper transition-colors"
            />
            <div className="flex gap-2">
              <button
                type="submit"
                className="flex-1 px-4 py-2 rounded-lg bg-copper text-white text-sm font-medium hover:bg-copper-dark transition-colors"
              >
                Create
              </button>
              <button
                type="button"
                onClick={() => { setShowNew(false); setName(""); }}
                className="px-4 py-2 rounded-lg bg-bg-surface text-text-secondary text-sm border border-border hover:text-text-primary transition-colors"
              >
                Cancel
              </button>
            </div>
            <p className="text-xs text-text-muted">You'll be prompted to pick a save location</p>
          </form>
        )}

        <div className="text-xs text-text-muted space-y-1">
          <p>Ctrl+N — New project</p>
          <p>Ctrl+O — Open project</p>
          <p>Ctrl+S — Save project</p>
        </div>
      </div>
    </div>
  );
}
