import React, { useState, useEffect } from "react";
import { Sparkles, Search } from "lucide-react";
import { useAIStore } from "../../stores/aiStore";

export function RecommenderWidget() {
  const [query, setQuery] = useState("");
  const { recommendations, recommend, recLoading } = useAIStore();

  useEffect(() => {
    if (query.length >= 2) {
      const timer = setTimeout(() => recommend(query), 300);
      return () => clearTimeout(timer);
    }
  }, [query, recommend]);

  return (
    <div className="space-y-3">
      <div className="relative">
        <Search size={14} className="absolute left-2 top-1/2 -translate-y-1/2 text-text-muted" />
        <input
          type="text"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          placeholder="Search components..."
          className="w-full bg-bg-deep border border-border rounded pl-7 pr-2 py-1.5 text-xs text-text-primary placeholder-text-muted focus:outline-none focus:border-info"
        />
      </div>

      {recLoading && (
        <p className="text-xs text-text-muted text-center animate-pulse">Searching...</p>
      )}

      {!recLoading && recommendations.length === 0 && query.length === 0 && (
        <div className="text-center py-6">
          <Sparkles size={24} className="mx-auto text-text-muted mb-2" />
          <p className="text-xs text-text-muted">Type a component name (e.g. resistor, led, mcu)</p>
        </div>
      )}

      {recommendations.length > 0 && (
        <div className="space-y-1">
          {recommendations.map((rec, i) => (
            <div
              key={i}
              className="text-xs bg-bg-deep border border-border rounded p-2 hover:bg-bg-elevated transition-colors cursor-pointer"
            >
              {rec}
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
