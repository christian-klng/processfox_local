import { AgentSwitcher } from "@/components/agent/AgentSwitcher";
import { SkillIconRow } from "@/components/agent/SkillIconRow";
import { ChatPane } from "@/components/chat/ChatPane";
import { FileTree } from "@/components/filetree/FileTree";
import { PreviewPane } from "@/components/preview/PreviewPane";
import {
  ResizableHandle,
  ResizablePanel,
  ResizablePanelGroup,
} from "@/components/ui/resizable";
import type { PendingToolCall } from "@/hooks/useAgentChat";
import type { Agent } from "@/types/agent";
import type { ChatMessage } from "@/types/chat";
import type { Skill } from "@/types/skill";

type Props = {
  agents: Agent[];
  activeAgent: Agent | null;
  selectedFile: { path: string; name: string } | null;
  messages: ChatMessage[];
  streamingText: string | null;
  pendingTools: PendingToolCall[];
  sending: boolean;
  chatError: string | null;
  chatDisabled: boolean;
  chatDisabledReason: string | undefined;
  skills: Skill[];
  fileTreeRefresh: number;
  onSelectAgent: (agent: Agent) => void;
  onCreateAgent: () => void;
  onEditAgent: () => void;
  onOpenSettings: () => void;
  onSelectFile: (path: string, name: string) => void;
  onClosePreview: () => void;
  onSendMessage: (text: string) => void;
  onCancelRun: () => void;
  onDismissChatError: () => void;
};

export function Main({
  agents,
  activeAgent,
  selectedFile,
  messages,
  streamingText,
  pendingTools,
  sending,
  chatError,
  chatDisabled,
  chatDisabledReason,
  skills,
  fileTreeRefresh,
  onSelectAgent,
  onCreateAgent,
  onEditAgent,
  onOpenSettings,
  onSelectFile,
  onClosePreview,
  onSendMessage,
  onCancelRun,
  onDismissChatError,
}: Props) {
  const showPreview = selectedFile !== null;

  return (
    <ResizablePanelGroup
      direction="horizontal"
      className="h-full w-full bg-background"
    >
      <ResizablePanel defaultSize={22} minSize={16} maxSize={36}>
        <div className="flex h-full flex-col border-r border-border bg-surface">
          <AgentSwitcher
            agents={agents}
            activeAgent={activeAgent}
            onSelect={onSelectAgent}
            onCreate={onCreateAgent}
            onEdit={onEditAgent}
            onOpenSettings={onOpenSettings}
          />
          <SkillIconRow activeSkillNames={activeAgent?.skills ?? []} skills={skills} />
          <div className="flex-1 overflow-hidden border-t border-border">
            <FileTree
              agentId={activeAgent?.id ?? null}
              hasFolder={Boolean(activeAgent?.folder)}
              refreshSignal={fileTreeRefresh}
              onSelectFile={onSelectFile}
              onRequestPickFolder={onEditAgent}
            />
          </div>
        </div>
      </ResizablePanel>

      <ResizableHandle />

      {showPreview && (
        <>
          <ResizablePanel defaultSize={38} minSize={20}>
            <PreviewPane
              fileName={selectedFile?.name ?? null}
              filePath={selectedFile?.path ?? null}
              onClose={onClosePreview}
            />
          </ResizablePanel>
          <ResizableHandle />
        </>
      )}

      <ResizablePanel defaultSize={showPreview ? 40 : 78} minSize={30}>
        <ChatPane
          messages={messages}
          streamingText={streamingText}
          pendingTools={pendingTools}
          sending={sending}
          error={chatError}
          disabled={chatDisabled}
          disabledReason={chatDisabledReason}
          onSend={onSendMessage}
          onCancel={onCancelRun}
          onDismissError={onDismissChatError}
          onOpenSettings={onOpenSettings}
        />
      </ResizablePanel>
    </ResizablePanelGroup>
  );
}
