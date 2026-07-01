import React from "react";
import { ChatWindow } from "./ChatWindow";
import { RecommenderWidget } from "./RecommenderWidget";
import { Sparkles } from "lucide-react";

export function AIPanel() {
  const [tab, setTab] = React.useState<"chat" | "recommend">("recommend");

  return (
    <div className="flex flex-col h-full">
      <div className="flex border-b border-border">
        <button
          onClick={() => setTab("recommend")}
          className={`flex-1 py-2 text-xs font-medium ${
            tab === "recommend"
              ? "text-copper border-b-2 border-copper"
              : "text-text-secondary"
          }`}
        >
          Suggestions
        </button>
        <button
          onClick={() => setTab("chat")}
          className={`flex-1 py-2 text-xs font-medium ${
            tab === "chat"
              ? "text-copper border-b-2 border-copper"
              : "text-text-secondary"
          }`}
        >
          Chat
        </button>
      </div>
      <div className="flex-1 overflow-y-auto p-3">
        {tab === "recommend" ? <RecommenderWidget /> : <ChatWindow />}
      </div>
    </div>
  );
}
