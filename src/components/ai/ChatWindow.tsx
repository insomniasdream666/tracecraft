import React, { useState, useRef, useEffect } from "react";
import { Send, Trash2 } from "lucide-react";
import { useAIStore } from "../../stores/aiStore";

const SUGGESTIONS = [
  "What clearance for 5V nets?",
  "What trace width for 1A?",
  "How to design a ground plane?",
  "Is my board ready for fabrication?",
];

export function ChatWindow() {
  const [input, setInput] = useState("");
  const { messages, sendMessage, loading, clearChat } = useAIStore();
  const bottomRef = useRef<HTMLDivElement>(null);

  useEffect(() => { bottomRef.current?.scrollIntoView({ behavior: "smooth" }); }, [messages]);

  const handleSend = async () => {
    if (!input.trim() || loading) return;
    const msg = input.trim();
    setInput("");
    await sendMessage(msg);
  };

  return (
    <div className="flex flex-col h-full">
      <div className="flex items-center justify-between mb-2">
        <span className="text-xs font-medium text-text-secondary">AI Assistant</span>
        {messages.length > 0 && (
          <button onClick={clearChat} className="text-text-muted hover:text-text p-1">
            <Trash2 size={12} />
          </button>
        )}
      </div>

      <div className="flex-1 overflow-y-auto space-y-2 mb-3">
        {messages.length === 0 && (
          <div className="space-y-2">
            <p className="text-xs text-text-muted text-center py-2">Ask me about PCB design</p>
            {SUGGESTIONS.map((s) => (
              <button
                key={s}
                onClick={() => { setInput(s); }}
                className="block w-full text-left text-xs text-copper bg-bg-deep/50 hover:bg-bg-deep border border-border rounded p-2 transition-colors"
              >
                {s}
              </button>
            ))}
          </div>
        )}
        {messages.map((msg, i) => (
          <div
            key={i}
            className={`text-xs p-2 rounded whitespace-pre-wrap ${
              msg.role === "user"
                ? "bg-bg-elevated text-text-primary ml-4"
                : "bg-bg-deep text-text-secondary mr-4"
            }`}
          >
            {msg.content}
          </div>
        ))}
        {loading && (
          <div className="text-xs text-text-muted text-center py-1 animate-pulse">Thinking...</div>
        )}
        <div ref={bottomRef} />
      </div>

      <div className="flex gap-2">
        <input
          type="text"
          value={input}
          onChange={(e) => setInput(e.target.value)}
          onKeyDown={(e) => e.key === "Enter" && handleSend()}
          placeholder="Ask about PCB design..."
          className="flex-1 bg-bg-deep border border-border rounded px-2 py-1.5 text-xs text-text-primary placeholder-text-muted focus:outline-none focus:border-info"
          disabled={loading}
        />
        <button
          onClick={handleSend}
          disabled={loading || !input.trim()}
          className="p-1.5 bg-copper text-white rounded hover:bg-copper-dark disabled:opacity-40 transition-colors"
        >
          <Send size={14} />
        </button>
      </div>
    </div>
  );
}
