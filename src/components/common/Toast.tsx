import { useEffect } from "react";
import toast, { Toaster } from "react-hot-toast";
import { useProjectStore } from "../../stores/projectStore";
import { usePCBStore } from "../../stores/pcbStore";
import { useSchematicStore } from "../../stores/schematicStore";
import { useLibraryStore } from "../../stores/libraryStore";
import { useAIStore } from "../../stores/aiStore";

export function ToastSetup() {
  const projectErr = useProjectStore((s) => s.error);
  const pcbErr = usePCBStore((s) => s.error);
  const schematicErr = useSchematicStore((s) => s.error);
  const libErr = useLibraryStore((s) => s.error);
  const aiErr = useAIStore((s) => s.error);

  const error = projectErr || pcbErr || schematicErr || libErr || aiErr;

  useEffect(() => {
    if (error) toast.error(error, { id: "store-error" });
  }, [error]);

  return (
    <Toaster
      position="bottom-right"
      toastOptions={{
        className: "bg-bg-surface text-text-primary border border-border text-sm",
        duration: 4000,
        style: {
          background: "#232540",
          color: "#E8E9F3",
          border: "1px solid #343760",
          borderRadius: "8px",
          fontSize: "13px",
        },
        error: {
          iconTheme: { primary: "#E6454A", secondary: "#232540" },
        },
        success: {
          iconTheme: { primary: "#42BE65", secondary: "#232540" },
        },
      }}
    />
  );
}
