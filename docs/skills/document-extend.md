---
name: document-extend
title: Dokument ergänzen
description: Append content to an existing MD or TXT document. Useful for rolling logs, long-running memory notes, journal-like files, and any file that should grow over time without overwriting prior content.
icon: ➕
tools:
  - read_file
  - append_to_md
  - llm_extract_structured
hitl:
  default: false
  per_tool:
    append_to_md: optional
language: en
---

# Skill: Dokument ergänzen

## Purpose
Add new content to the end of an existing Markdown or text document without modifying earlier content. This is how ProcessFox implements simple long-term memory (append-only notes), rolling logs, and journal-style files.

## When to Use
- User says "merk dir das", "notiere bitte", "füge zum Journal hinzu", "log this".
- Agent wants to persist a piece of information for later retrieval.
- Summaries or decisions that should stack over time (daily notes, changelog, etc.).

## How to Use
1. Determine the target file from context. If unclear, list plausible candidates with `read_file` and/or ask via `ask_user`.
2. Format the new entry consistently (timestamped heading or bullet, as fits the existing document style — read the file first to match tone).
3. Append via `append_to_md`, which adds a separator (blank line) and the new block.
4. Confirm in the user's language, citing the file and the new section.

## HITL Behavior
Default: false — appending is low-risk and the file is never overwritten. However, the skill supports two operational variants via per-agent override:
- **Without approval (default):** append silently.
- **With approval:** show the new entry in an HITL card before appending.

A parallel skill variant "Dokument ergänzen (mit Rückfrage)" may be exposed in the UI for users who prefer explicit confirmation.

## Example Interactions

### Example 1 — memory update
User: "Merk dir, dass Kunde Müller GmbH 10 % Rabatt bekommt."
Plan: check agent for a known memory file (e.g. `memory.md` in root) → append entry with date.

### Example 2 — session log
Agent finishes a task, wants to log it: append to `journal.md`.

## Anti-Patterns
- Do not edit prior content. This skill only appends.
- Do not create a new file by accident. If the target file doesn't exist, ask the user first or use `document-create-docx` / a MD creator.

## Notes for Maintainers
The "memory document" is whatever MD file the user designates. There is no hidden `.processfox/memory.md`. The user owns the file location.
