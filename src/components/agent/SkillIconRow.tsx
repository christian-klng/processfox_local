import type { Skill } from "@/types/skill";

type Props = {
  activeSkillNames: string[];
  skills: Skill[];
};

export function SkillIconRow({ activeSkillNames, skills }: Props) {
  if (activeSkillNames.length === 0) {
    return (
      <div className="px-3 pb-2 text-[11px] text-muted-foreground">
        Keine Skills aktiv.
      </div>
    );
  }
  const active = activeSkillNames.map((name) => ({
    name,
    skill: skills.find((s) => s.name === name),
  }));
  return (
    <div className="flex flex-wrap items-center gap-1 px-3 pb-2">
      {active.map(({ name, skill }) => (
        <span
          key={name}
          title={skill?.title ?? name}
          className="flex items-center gap-1 rounded-sm border border-border bg-muted/50 px-1.5 py-0.5 text-[11px] text-muted-foreground"
        >
          <span className="text-sm leading-none">{skill?.icon ?? "🔧"}</span>
          <span>{skill?.title ?? name}</span>
        </span>
      ))}
    </div>
  );
}
