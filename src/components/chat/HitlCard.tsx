import {
  Check,
  FileEdit,
  FilePen,
  FilePlus,
  FileText,
  Sheet as SheetIcon,
  X,
} from "lucide-react";
import { diffLines } from "diff";

import { Button } from "@/components/ui/button";
import type { HitlPreview, PendingHitl } from "@/types/chat";

type Props = {
  hitl: PendingHitl;
  busy?: boolean;
  onApprove: () => void;
  onReject: () => void;
};

export function HitlCard({ hitl, busy, onApprove, onReject }: Props) {
  const { preview } = hitl;
  const heading = headingFor(preview);
  return (
    <div className="flex flex-col gap-2 rounded-md border border-amber-500/40 bg-amber-500/10 p-3 text-xs text-amber-900 dark:text-amber-200">
      <div className="flex items-center gap-2">
        <heading.Icon className="h-3.5 w-3.5" />
        <span className="text-sm font-medium">{heading.label}</span>
        <span className="ml-auto rounded-sm border border-amber-500/40 bg-amber-500/10 px-1.5 py-0.5 font-mono text-[11px]">
          {hitl.toolName}
        </span>
      </div>

      <div className="flex flex-col gap-1">
        <div className="text-[11px] uppercase tracking-wide opacity-70">
          Datei
        </div>
        <div className="rounded-sm border border-amber-500/30 bg-background/40 px-2 py-1 font-mono text-xs">
          {preview.path}
        </div>
      </div>

      <PreviewBody preview={preview} />

      <div className="flex justify-end gap-2 pt-1">
        <Button
          size="sm"
          variant="ghost"
          onClick={onReject}
          disabled={busy}
          className="gap-1.5"
        >
          <X className="h-3.5 w-3.5" />
          Ablehnen
        </Button>
        <Button size="sm" onClick={onApprove} disabled={busy} className="gap-1.5">
          <Check className="h-3.5 w-3.5" />
          Freigeben
        </Button>
      </div>
    </div>
  );
}

function headingFor(preview: HitlPreview): {
  Icon: typeof FilePlus;
  label: string;
} {
  switch (preview.kind) {
    case "appendToFile":
      return preview.createsFile
        ? { Icon: FilePlus, label: "Neue Datei erstellen" }
        : { Icon: FilePen, label: "Inhalt anhängen" };
    case "writeDocx":
      return preview.createsFile
        ? { Icon: FilePlus, label: "Word-Dokument erstellen" }
        : { Icon: FileText, label: "Word-Dokument überschreiben" };
    case "appendToDocx":
      return preview.createsFile
        ? { Icon: FilePlus, label: "Word-Dokument erstellen" }
        : { Icon: FilePen, label: "Word-Dokument erweitern" };
    case "rewriteFile":
      return preview.createsFile
        ? { Icon: FilePlus, label: "Neue Datei schreiben" }
        : { Icon: FileEdit, label: "Datei komplett ersetzen" };
    case "updateCells": {
      const n = preview.changes.length;
      return {
        Icon: SheetIcon,
        label: `${n} ${n === 1 ? "Zelle" : "Zellen"} aktualisieren`,
      };
    }
    case "writeXlsx":
      return preview.createsFile
        ? { Icon: FilePlus, label: "Excel-Tabelle erstellen" }
        : { Icon: SheetIcon, label: "Excel-Tabelle überschreiben" };
  }
}

function PreviewBody({ preview }: { preview: HitlPreview }) {
  switch (preview.kind) {
    case "appendToFile":
      return (
        <>
          {preview.existingTail && (
            <Section label="Bisheriger Inhalt (Ende)" subdued>
              {preview.existingTail}
            </Section>
          )}
          <Section
            label={preview.existingTail ? "Anzuhängender Inhalt" : "Inhalt"}
          >
            {preview.content}
          </Section>
        </>
      );
    case "writeDocx":
      return (
        <>
          <Section
            label={`Inhalt — ${preview.blockCount} ${
              preview.blockCount === 1 ? "Block" : "Blöcke"
            }`}
          >
            {preview.previewText}
          </Section>
          {!preview.createsFile && (
            <p className="text-[11px] opacity-80">
              ⚠ Diese Datei existiert bereits und wird komplett überschrieben.
              Vorhandene Formatierung geht verloren — für Erweiterung benutze
              den Skill „Dokument fortschreiben".
            </p>
          )}
        </>
      );
    case "appendToDocx":
      return (
        <>
          {preview.existingTail && (
            <Section label="Bisheriger Inhalt (Ende, als Text)" subdued>
              {preview.existingTail}
            </Section>
          )}
          <Section
            label={`Anzuhängender Inhalt — ${preview.blockCount} ${
              preview.blockCount === 1 ? "Block" : "Blöcke"
            }`}
          >
            {preview.previewText}
          </Section>
        </>
      );
    case "rewriteFile":
      return <DiffSection before={preview.before} after={preview.after} />;
    case "updateCells":
      return (
        <CellChangesSection
          sheet={preview.sheet}
          changes={preview.changes}
        />
      );
    case "writeXlsx":
      return (
        <WriteXlsxSection
          sheet={preview.sheet}
          rows={preview.rows}
          createsFile={preview.createsFile}
        />
      );
  }
}

function Section({
  label,
  children,
  subdued,
}: {
  label: string;
  children: string;
  subdued?: boolean;
}) {
  return (
    <div className="flex flex-col gap-1">
      <div className="text-[11px] uppercase tracking-wide opacity-70">
        {label}
      </div>
      <pre
        className={`max-h-64 overflow-auto rounded-sm border border-amber-500/30 ${
          subdued ? "bg-background/40 opacity-80" : "bg-background/60"
        } p-2 font-mono text-[11px] whitespace-pre-wrap`}
      >
        {children}
      </pre>
    </div>
  );
}

function WriteXlsxSection({
  sheet,
  rows,
  createsFile,
}: {
  sheet: string;
  rows: string[][];
  createsFile: boolean;
}) {
  const MAX_ROWS = 10;
  const visible = rows.slice(0, MAX_ROWS);
  const hidden = rows.length - visible.length;
  const colCount = rows.reduce((m, r) => Math.max(m, r.length), 0);
  return (
    <>
      <div className="flex flex-col gap-1">
        <div className="text-[11px] uppercase tracking-wide opacity-70">
          Sheet „{sheet}" — {rows.length}{" "}
          {rows.length === 1 ? "Zeile" : "Zeilen"} × {colCount}{" "}
          {colCount === 1 ? "Spalte" : "Spalten"}
        </div>
        <div className="overflow-auto rounded-sm border border-amber-500/30 bg-background/60 font-mono text-[11px]">
          <table className="w-full">
            <tbody>
              {visible.map((row, i) => (
                <tr
                  key={i}
                  className={`border-b border-amber-500/20 last:border-b-0 ${
                    i === 0 ? "font-medium" : ""
                  }`}
                >
                  {Array.from({ length: colCount }).map((_, j) => (
                    <td key={j} className="border-r border-amber-500/10 px-2 py-0.5 last:border-r-0">
                      {row[j] ?? ""}
                    </td>
                  ))}
                </tr>
              ))}
            </tbody>
          </table>
        </div>
        {hidden > 0 && (
          <div className="text-[11px] opacity-70">
            … +{hidden} weitere {hidden === 1 ? "Zeile" : "Zeilen"} (nicht angezeigt)
          </div>
        )}
      </div>
      {!createsFile && (
        <p className="text-[11px] opacity-80">
          ⚠ Diese Datei existiert bereits und wird komplett ersetzt — vorhandene
          Sheets, Formatierung und Formeln gehen verloren. Für gezielte Änderungen
          benutze den Skill „Tabelle ändern".
        </p>
      )}
    </>
  );
}

function CellChangesSection({
  sheet,
  changes,
}: {
  sheet: string;
  changes: { cell: string; before: string; after: string }[];
}) {
  return (
    <div className="flex flex-col gap-1">
      <div className="text-[11px] uppercase tracking-wide opacity-70">
        Sheet „{sheet}" — Änderungen
      </div>
      <div className="overflow-auto rounded-sm border border-amber-500/30 bg-background/60 font-mono text-[11px]">
        <table className="w-full">
          <thead>
            <tr className="border-b border-amber-500/30 text-left opacity-70">
              <th className="px-2 py-1 font-medium">Zelle</th>
              <th className="px-2 py-1 font-medium">Vorher</th>
              <th className="px-2 py-1 font-medium">Nachher</th>
            </tr>
          </thead>
          <tbody>
            {changes.map((c, i) => (
              <tr
                key={`${c.cell}-${i}`}
                className="border-b border-amber-500/20 last:border-b-0"
              >
                <td className="px-2 py-1 font-medium">{c.cell}</td>
                <td className="px-2 py-1 align-top text-rose-700 line-through opacity-80 dark:text-rose-300">
                  {c.before || <span className="opacity-50">(leer)</span>}
                </td>
                <td className="px-2 py-1 align-top text-emerald-700 dark:text-emerald-300">
                  {c.after || <span className="opacity-50">(leer)</span>}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}

function DiffSection({ before, after }: { before: string; after: string }) {
  const parts = diffLines(before, after);
  return (
    <div className="flex flex-col gap-1">
      <div className="text-[11px] uppercase tracking-wide opacity-70">
        Änderungen
      </div>
      <div className="max-h-80 overflow-auto rounded-sm border border-amber-500/30 bg-background/60 font-mono text-[11px]">
        {parts.length === 0 ? (
          <div className="px-2 py-1 italic opacity-70">Keine Änderungen.</div>
        ) : (
          parts.map((p, i) => {
            const cls = p.added
              ? "bg-emerald-500/15 text-emerald-800 dark:text-emerald-200"
              : p.removed
                ? "bg-rose-500/15 text-rose-800 dark:text-rose-200"
                : "opacity-70";
            const prefix = p.added ? "+ " : p.removed ? "- " : "  ";
            const lines = p.value.replace(/\n$/, "").split("\n");
            return (
              <div key={i} className={cls}>
                {lines.map((line, j) => (
                  <div key={j} className="whitespace-pre-wrap px-2">
                    {prefix}
                    {line}
                  </div>
                ))}
              </div>
            );
          })
        )}
      </div>
    </div>
  );
}
