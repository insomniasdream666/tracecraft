import React from "react";
import { Loader2 } from "lucide-react";

interface LoadingOverlayProps {
  loading: boolean;
  message?: string;
}

export function LoadingOverlay({ loading, message }: LoadingOverlayProps) {
  if (!loading) return null;
  return (
    <div className="absolute inset-0 bg-bg-canvas/60 flex items-center justify-center z-50">
      <div className="flex flex-col items-center gap-2">
        <Loader2 size={24} className="text-copper animate-spin" />
        {message && <span className="text-xs text-text-secondary">{message}</span>}
      </div>
    </div>
  );
}
