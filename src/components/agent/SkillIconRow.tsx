type Props = {
  skills: string[];
};

/**
 * In Phase 1 we only know skill names, not their icons. Placeholder row —
 * Phase 3 will replace these with real icons fetched from the SkillRegistry.
 */
export function SkillIconRow({ skills }: Props) {
  if (skills.length === 0) {
    return (
      <div className="px-3 pb-2 text-[11px] text-muted-foreground">
        Keine Skills aktiv.
      </div>
    );
  }
  return (
    <div className="flex flex-wrap items-center gap-1 px-3 pb-2">
      {skills.map((s) => (
        <span
          key={s}
          className="rounded-sm border border-border bg-muted/50 px-1.5 py-0.5 text-[11px] text-muted-foreground"
        >
          {s}
        </span>
      ))}
    </div>
  );
}
