import React from "react";
import { X, CircuitBoard } from "lucide-react";
import { APP_VERSION } from "../../lib/constants";

interface Props {
  onClose: () => void;
}

export function AboutModal({ onClose }: Props) {
  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60" onClick={onClose}>
      <div
        className="bg-bg-surface rounded-xl border border-border shadow-2xl w-[380px] p-6 text-center relative"
        onClick={(e) => e.stopPropagation()}
      >
        <button
          onClick={onClose}
          className="absolute top-3 right-3 text-text-muted hover:text-text-primary transition-colors"
        >
          <X size={16} />
        </button>
        <div className="w-16 h-16 mx-auto mb-4 rounded-2xl bg-gradient-to-br from-copper to-copper-dark flex items-center justify-center">
          <CircuitBoard size={32} className="text-white" />
        </div>
        <h2 className="text-lg font-bold text-text-primary">TraceCraft</h2>
        <p className="text-xs text-text-muted mt-1">v{APP_VERSION}</p>
        <p className="text-xs text-text-secondary mt-3 leading-relaxed">
          AI-assisted PCB design tool built with Tauri, React, and Rust.
        </p>
        <div className="mt-4 pt-4 border-t border-border text-[10px] text-text-muted">
          <p>Local-first. No cloud accounts required.</p>
          <p className="mt-1">KiCad schematic and PCB import/export (basic subset).</p>
        </div>
      </div>
    </div>
  );
}
