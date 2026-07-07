import React from "react";
import { AlertTriangle, RefreshCw } from "lucide-react";

interface Props {
  children: React.ReactNode;
  label?: string;
}

interface State {
  error: Error | null;
}

export class ErrorBoundary extends React.Component<Props, State> {
  constructor(props: Props) {
    super(props);
    this.state = { error: null };
  }

  static getDerivedStateFromError(error: Error) {
    return { error };
  }

  render() {
    if (this.state.error) {
      return (
        <div className="flex-1 flex items-center justify-center bg-bg-canvas">
          <div className="flex flex-col items-center gap-4 max-w-md text-center px-8">
            <AlertTriangle size={32} className="text-error" />
            <div>
              <h2 className="text-lg font-semibold text-text-primary">{this.props.label || "Something went wrong"}</h2>
              <p className="text-sm text-text-muted mt-1 font-mono">{this.state.error.message}</p>
            </div>
            <button
              onClick={() => this.setState({ error: null })}
              className="flex items-center gap-2 px-4 py-2 rounded-lg bg-bg-surface text-text-primary border border-border hover:bg-bg-elevated text-sm transition-colors"
            >
              <RefreshCw size={14} />
              Retry
            </button>
          </div>
        </div>
      );
    }
    return this.props.children;
  }
}
