import { useEffect, useMemo, useRef, useState } from "react";
import { Tree, type NodeApi, type NodeRendererProps } from "react-arborist";
import { ChevronRight, File, Folder, FolderOpen } from "lucide-react";

import { fileApi } from "@/lib/tauri";
import type { FileEntry } from "@/types/file";
import { cn } from "@/lib/utils";

type TreeNode = {
  id: string;
  name: string;
  path: string;
  isDir: boolean;
  children?: TreeNode[];
};

type Props = {
  agentId: string | null;
  hasFolder: boolean;
  onSelectFile: (path: string, name: string) => void;
  onRequestPickFolder: () => void;
};

export function FileTree({
  agentId,
  hasFolder,
  onSelectFile,
  onRequestPickFolder,
}: Props) {
  const [data, setData] = useState<TreeNode[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const containerRef = useRef<HTMLDivElement>(null);
  const [size, setSize] = useState({ width: 260, height: 480 });

  useEffect(() => {
    if (!containerRef.current) return;
    const el = containerRef.current;
    const update = () =>
      setSize({ width: el.clientWidth, height: el.clientHeight });
    update();
    const ro = new ResizeObserver(update);
    ro.observe(el);
    return () => ro.disconnect();
  }, []);

  useEffect(() => {
    if (!agentId || !hasFolder) {
      setData([]);
      return;
    }
    let cancelled = false;
    setLoading(true);
    setError(null);
    fileApi
      .listAgentFolder(agentId)
      .then((entries: FileEntry[]) => {
        if (cancelled) return;
        setData(
          entries.map((e) => ({
            id: e.path,
            name: e.name,
            path: e.path,
            isDir: e.isDir,
            children: e.isDir ? [] : undefined,
          })),
        );
      })
      .catch((err) => {
        if (cancelled) return;
        setError(typeof err === "string" ? err : (err?.message ?? String(err)));
      })
      .finally(() => {
        if (!cancelled) setLoading(false);
      });
    return () => {
      cancelled = true;
    };
  }, [agentId, hasFolder]);

  const content = useMemo(() => {
    if (!agentId) {
      return (
        <EmptyState
          title="Kein Agent ausgewählt"
          description="Leg oben einen Agenten an."
        />
      );
    }
    if (!hasFolder) {
      return (
        <EmptyState
          title="Kein Ordner gewählt"
          description="Verknüpfe einen Ordner mit diesem Agenten."
          action={{ label: "Ordner wählen", onClick: onRequestPickFolder }}
        />
      );
    }
    if (loading) {
      return (
        <div className="px-3 py-2 text-xs text-muted-foreground">Lädt …</div>
      );
    }
    if (error) {
      return (
        <div className="px-3 py-2 text-xs text-destructive">
          Fehler: {error}
        </div>
      );
    }
    if (data.length === 0) {
      return (
        <div className="px-3 py-2 text-xs text-muted-foreground">
          Der Ordner ist leer.
        </div>
      );
    }
    return (
      <Tree<TreeNode>
        data={data}
        width={size.width}
        height={size.height}
        rowHeight={26}
        indent={14}
        openByDefault={false}
        disableMultiSelection
        onActivate={(node: NodeApi<TreeNode>) => {
          if (!node.data.isDir) onSelectFile(node.data.path, node.data.name);
        }}
      >
        {Node}
      </Tree>
    );
  }, [
    agentId,
    hasFolder,
    loading,
    error,
    data,
    size.width,
    size.height,
    onRequestPickFolder,
    onSelectFile,
  ]);

  return (
    <div ref={containerRef} className="h-full w-full overflow-hidden">
      {content}
    </div>
  );
}

function Node({
  node,
  style,
  dragHandle,
}: NodeRendererProps<TreeNode>) {
  const isDir = node.data.isDir;
  return (
    <div
      ref={dragHandle}
      style={style}
      className={cn(
        "flex h-full items-center gap-1.5 rounded-sm px-2 text-sm",
        "cursor-pointer select-none hover:bg-accent/60",
        node.isSelected && "bg-accent",
      )}
      onClick={() => {
        if (isDir) node.toggle();
        else node.activate();
      }}
    >
      {isDir ? (
        <ChevronRight
          className={cn(
            "h-3 w-3 shrink-0 text-muted-foreground transition-transform",
            node.isOpen && "rotate-90",
          )}
        />
      ) : (
        <span className="inline-block w-3" />
      )}
      {isDir ? (
        node.isOpen ? (
          <FolderOpen className="h-3.5 w-3.5 shrink-0 text-muted-foreground" />
        ) : (
          <Folder className="h-3.5 w-3.5 shrink-0 text-muted-foreground" />
        )
      ) : (
        <File className="h-3.5 w-3.5 shrink-0 text-muted-foreground" />
      )}
      <span className="truncate">{node.data.name}</span>
    </div>
  );
}

function EmptyState({
  title,
  description,
  action,
}: {
  title: string;
  description: string;
  action?: { label: string; onClick: () => void };
}) {
  return (
    <div className="flex h-full flex-col items-center justify-center gap-2 px-4 text-center">
      <div className="text-sm font-medium">{title}</div>
      <div className="text-xs text-muted-foreground">{description}</div>
      {action && (
        <button
          onClick={action.onClick}
          className="mt-1 rounded-md border border-border bg-surface px-3 py-1 text-xs shadow-subtle hover:bg-accent"
        >
          {action.label}
        </button>
      )}
    </div>
  );
}
