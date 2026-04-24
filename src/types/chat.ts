export type ChatMessageRole = "user" | "assistant" | "system" | "tool";

export interface ChatMessage {
  id: string;
  role: ChatMessageRole;
  content: string;
  createdAt: string;
}

export interface RunStarted {
  runId: string;
  assistantMessageId: string;
}

export type RunEvent =
  | { type: "delta"; text: string }
  | {
      type: "finish";
      reason: "stop" | "max_tokens" | "cancelled" | "error";
      message: ChatMessage;
    }
  | { type: "error"; code: string; message: string };
