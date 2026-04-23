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
import type { Agent } from "@/types/agent";
import type { Message } from "@/types/message";

type Props = {
  agents: Agent[];
  activeAgent: Agent | null;
  selectedFile: { path: string; name: string } | null;
  messages: Message[];
  onSelectAgent: (agent: Agent) => void;
  onCreateAgent: () => void;
  onEditAgent: () => void;
  onOpenSettings: () => void;
  onSelectFile: (path: string, name: string) => void;
  onClosePreview: () => void;
  onSendMessage: (text: string) => void;
};

export function Main({
  agents,
  activeAgent,
  selectedFile,
  messages,
  onSelectAgent,
  onCreateAgent,
  onEditAgent,
  onOpenSettings,
  onSelectFile,
  onClosePreview,
  onSendMessage,
}: Props) {
  const showPreview = selectedFile !== null;
  const chatDisabled = activeAgent === null;

  return (
    <ResizablePanelGroup
      direction="horizontal"
      className="h-full w-full bg-background"
    >
      {/* Left: sidebar */}
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
          <SkillIconRow skills={activeAgent?.skills ?? []} />
          <div className="flex-1 overflow-hidden border-t border-border">
            <FileTree
              agentId={activeAgent?.id ?? null}
              hasFolder={Boolean(activeAgent?.folder)}
              onSelectFile={onSelectFile}
              onRequestPickFolder={onEditAgent}
            />
          </div>
        </div>
      </ResizablePanel>

      <ResizableHandle />

      {/* Middle: preview (only visible when a file is selected) */}
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

      {/* Right: chat */}
      <ResizablePanel defaultSize={showPreview ? 40 : 78} minSize={30}>
        <ChatPane
          messages={messages}
          disabled={chatDisabled}
          disabledReason="Leg zunächst einen Agenten an."
          onSend={onSendMessage}
        />
      </ResizablePanel>
    </ResizablePanelGroup>
  );
}
