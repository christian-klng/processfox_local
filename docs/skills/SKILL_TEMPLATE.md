---
name: skill-slug
title: Skill Title (Display)
description: One-line description shown to the model to help it decide when to use this skill.
icon: 🔧
tools:
  - tool_name_1
  - tool_name_2
hitl:
  default: false
  per_tool: {}
language: en
---

# Skill: Skill Title

## Purpose
What this skill does, in one paragraph. Written in English — the model reads this.

## When to Use
Concrete signals that should trigger the model to reach for this skill.
- Signal 1
- Signal 2

## How to Use
Step-by-step guidance for the model. Prefer short, imperative sentences.

1. First, do X using `tool_name_1`.
2. If Y, then do Z using `tool_name_2`.
3. Always respond to the user in the user's language, regardless of the instruction language here.

## HITL Behavior
Describe when the skill asks the user for approval, if ever. Explicit about side-effects.

## Example Interactions

### Example 1 — typical case
User: "..."
Assistant plan: ...
Tool calls: ...
Result: ...

### Example 2 — edge case
...

## Anti-Patterns
Things the model should NOT do when using this skill.
- Do not …
- Avoid …

## Notes for Maintainers
Anything future contributors should know when evolving this skill.
