import { invoke } from "@tauri-apps/api/core";
import { useCallback, useState } from "react";

export function useTauriCommand<TResult, TArgs = void>() {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const execute = useCallback(
    async (command: string, args?: TArgs): Promise<TResult | null> => {
      setLoading(true);
      setError(null);
      try {
        return await invoke<TResult>(command, args as any);
      } catch (e) {
        const msg = typeof e === "string" ? e : String(e);
        setError(msg);
        return null;
      } finally {
        setLoading(false);
      }
    },
    []
  );

  return { execute, loading, error };
}
