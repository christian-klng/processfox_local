import { Wrench } from "lucide-react";

import { DynamicIcon } from "@/components/ui/DynamicIcon";
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import type { Skill } from "@/types/skill";

type Props = {
  activeSkillNames: string[];
  skills: Skill[];
};

export function SkillIconRow({ activeSkillNames, skills }: Props) {
  if (activeSkillNames.length === 0) {
    return (
      <div className="px-3 pb-2 text-xs text-muted-foreground">
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
        <Tooltip key={name}>
          <TooltipTrigger asChild>
            <span
              aria-label={skill?.title ?? name}
              className="flex h-6 w-6 cursor-help items-center justify-center rounded-md border border-border bg-muted/50 text-muted-foreground transition-colors hover:bg-muted hover:text-foreground"
            >
              <DynamicIcon
                name={skill?.icon}
                fallback={Wrench}
                className="h-3.5 w-3.5"
              />
            </span>
          </TooltipTrigger>
          <TooltipContent>
            <div className="font-medium">{skill?.title ?? name}</div>
            {skill?.description && (
              <div className="mt-0.5 text-muted-foreground">
                {skill.description}
              </div>
            )}
          </TooltipContent>
        </Tooltip>
      ))}
    </div>
  );
}
