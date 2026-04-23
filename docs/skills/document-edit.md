---
name: document-edit
title: Dokument bearbeiten
description: Edit an existing DOCX, MD, or TXT file. Use when the user asks to change, fix, or rewrite a portion of an existing document. Presents a diff preview before writing.
icon: ✏️
tools:
  - read_file
  - read_docx
  - write_docx
  - llm_extract_structured
hitl:
  default: true
  per_tool: {}
language: en
---

# Skill: Dokument bearbeiten

## Purpose
Modify an existing document. Produce a precise diff and require approval before saving.

## When to Use
- User asks to change, correct, or rewrite a section of a file that already exists.
- User asks to restructure or rename headings.
- User wants spelling/grammar passes or tone adjustments.

## How to Use
1. Read the current file with the appropriate reader tool.
2. Based on the user's instruction, produce the full new version of the content (not just the delta — keep full reconstruction for safety).
3. Compute a line-level or paragraph-level diff between old and new.
4. Show diff in the HITL card. Approve → write via `write_docx` or the appropriate writer; Reject → do nothing; Adjust → let the user edit before writing.
5. After writing, confirm in the user's language and reference the file.

## HITL Behavior
Default: true. Always show a diff. The user can disable HITL per-agent for aggressive editing workflows.

## Example Interactions

### Example 1 — typo fix
User: "In meinem Angebot steht 'Rchnung' — bitte korrigieren."
Plan: read → find typo → rewrite full document → show diff (one line) → approve → write.

### Example 2 — tone adjustment
User: "Mach den ganzen Absatz 'Zahlungsbedingungen' etwas freundlicher."
Plan: read → rewrite that section → diff → approve → write.

## Anti-Patterns
- Do not silently drop unrelated content. Always preserve everything you don't explicitly change.
- Do not guess about content not visible in the diff — if you need more context, read more of the file first.

## Notes for Maintainers
For DOCX, preserving formatting exactly during a "rewrite-from-text" is tricky. In v1, the approach is: read structured text from DOCX, edit, write back with basic formatting preserved (headings, paragraphs, bold/italic where detectable). Advanced formatting (images, tables, styles) is preserved if not touched; if the edit spans those, warn the user.
