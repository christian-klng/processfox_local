import { useCallback, useEffect, useRef, useState } from "react";

import { chatApi } from "@/lib/tauri";
import type { Agent } from "@/types/agent";
import type { ChatMessage } from "@/types/chat";
import type { Settings } from "@/types/settings";

export type EffectiveModel = {
  provider: string;
  modelId: string;
};

export function resolveAgentModel(
  agent: Agent | null,
  settings: Settings | null,
): EffectiveModel | null {
  if (!agent) return null;
  if (agent.model && agent.model.type === "cloud") {
    if (agent.model.id.trim().length === 0) return null;
    return { provider: agent.model.provider, modelId: agent.model.id };
  }
  if (agent.model && agent.model.type === "local") {
    // Local provider comes in Etappe C.
    return null;
  }
  const provider = settings?.defaultProvider;
  if (!provider) return null;
  const modelId = settings?.defaultModels?.[provider];
  if (!modelId) return null;
  return { provider, modelId };
}

type StreamState = {
  runId: string;
  assistantMessageId: string;
  buffer: string;
};

export type ChatState = {
  messages: ChatMessage[];
  sending: boolean;
  streamingText: string | null;
  error: string | null;
};

export function useAgentChat(
  agent: Agent | null,
  effectiveModel: EffectiveModel | null,
) {
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [sending, setSending] = useState(false);
  const [streamingText, setStreamingText] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const streamRef = useRef<StreamState | null>(null);
  const unlistenRef = useRef<(() => void) | null>(null);

  const resetStream = useCallback(() => {
    if (unlistenRef.current) {
      unlistenRef.current();
      unlistenRef.current = null;
    }
    streamRef.current = null;
    setStreamingText(null);
    setSending(false);
  }, []);

  // Load history whenever the active agent changes.
  useEffect(() => {
    resetStream();
    setError(null);
    if (!agent) {
      setMessages([]);
      return;
    }
    let cancelled = false;
    chatApi
      .listMessages(agent.id)
      .then((msgs) => {
        if (!cancelled) setMessages(msgs);
      })
      .catch((e) => {
        if (!cancelled) setError(String((e as { message?: string })?.message ?? e));
      });
    return () => {
      cancelled = true;
    };
  }, [agent, resetStream]);

  const send = useCallback(
    async (text: string) => {
      if (!agent || !effectiveModel || sending) return;
      setError(null);

      // Optimistic user bubble.
      const tempUserId = crypto.randomUUID();
      const tempUserMsg: ChatMessage = {
        id: tempUserId,
        role: "user",
        content: text,
        createdAt: new Date().toISOString(),
      };
      setMessages((prev) => [...prev, tempUserMsg]);
      setSending(true);
      setStreamingText("");

      let started;
      try {
        started = await chatApi.sendMessage({
          agentId: agent.id,
          provider: effectiveModel.provider,
          modelId: effectiveModel.modelId,
          text,
        });
      } catch (e) {
        setMessages((prev) => prev.filter((m) => m.id !== tempUserId));
        setSending(false);
        setStreamingText(null);
        setError(String((e as { message?: string })?.message ?? e));
        return;
      }

      streamRef.current = {
        runId: started.runId,
        assistantMessageId: started.assistantMessageId,
        buffer: "",
      };

      try {
        const unlisten = await chatApi.subscribeRun(started.runId, (event) => {
          const state = streamRef.current;
          if (!state || state.runId !== started.runId) return;
          if (event.type === "delta") {
            state.buffer += event.text;
            setStreamingText(state.buffer);
          } else if (event.type === "finish") {
            // The backend persisted the final message with `event.message`.
            // Replace our optimistic user bubble id reconciliation: the
            // backend also persisted the user message with a different id,
            // so refresh from disk for consistency.
            setMessages((prev) => {
              const withoutTempUser = prev.filter((m) => m.id !== tempUserId);
              return [...withoutTempUser];
            });
            // Append the assistant message.
            setMessages((prev) => [...prev, event.message]);
            // Refresh history to replace temp user msg with the persisted one.
            chatApi
              .listMessages(agent.id)
              .then(setMessages)
              .catch(() => {});
            resetStream();
          } else if (event.type === "error") {
            setError(`${event.code}: ${event.message}`);
            setMessages((prev) => prev.filter((m) => m.id !== tempUserId));
            // Re-sync with backend to get the persisted user message.
            chatApi
              .listMessages(agent.id)
              .then(setMessages)
              .catch(() => {});
            resetStream();
          }
        });
        unlistenRef.current = unlisten;
      } catch (e) {
        resetStream();
        setError(String((e as { message?: string })?.message ?? e));
      }
    },
    [agent, effectiveModel, sending, resetStream],
  );

  const cancel = useCallback(async () => {
    const state = streamRef.current;
    if (!state) return;
    try {
      await chatApi.cancelRun(state.runId);
    } catch (e) {
      console.warn("cancel failed", e);
    }
  }, []);

  // Unsubscribe on unmount.
  useEffect(() => {
    return () => {
      if (unlistenRef.current) unlistenRef.current();
    };
  }, []);

  return {
    messages,
    sending,
    streamingText,
    error,
    send,
    cancel,
    clearError: () => setError(null),
  } as const;
}
