---
name: document-create-docx
title: Dokument erstellen (DOCX)
description: Create a new Word document from user content, optionally based on a template in the agent folder. Use when the user asks to produce a new DOCX document such as an offer, memo, letter, or report.
icon: 📝
tools:
  - list_folder
  - read_docx
  - write_docx
  - write_docx_from_template
  - llm_extract_structured
  - ask_user
hitl:
  default: true
  per_tool: {}
language: en
---

# Skill: Dokument erstellen (DOCX)

## Purpose
Generate a new Word document either from freeform content or by filling placeholders in an existing `.docx` template the user has placed in the agent folder.

## When to Use
- User asks to create an offer, memo, letter, report, or similar DOCX deliverable.
- User wants to transform an email, notes, or a text prompt into a formal document.

## How to Use
1. Determine whether a template should be used:
   - Check the agent folder (or a `templates/` subfolder) for `.docx` files.
   - If one plausible template exists, use it. If multiple, ask the user which one.
   - If none, generate a freeform document.
2. For template-based: read placeholders from the template (e.g. `{{customer_name}}`, `{{offer_total}}`), extract the needed values from the user's input using `llm_extract_structured`, and call `write_docx_from_template`.
3. For freeform: draft structure (heading, sections, signature) and use `write_docx`.
4. Choose a sensible filename including date, e.g. `angebot_2026-04-23_mueller-gmbh.docx`.
5. Show the generated document preview as part of the HITL approval.
6. After approval, write the file.
7. Respond in the user's language with the filename and a one-line summary.

## HITL Behavior
Default: true. Before writing, show the user a preview of the full document (rendered text plus structure) as an inline-diff card. Buttons: Approve, Reject, Adjust (reopens editor).

The user can override this per agent to "without approval" for power usage.

## Example Interactions

### Example 1 — offer from email
User paste-s a customer email: "Wir benötigen 50 Stück von Produkt X bis Mitte Mai."
Plan:
- Find `offer_template.docx` in agent folder.
- Extract: customer, items, quantity, deadline.
- Fill template, generate `angebot_2026-04-23_<kunde>.docx`.
- Show preview, await approval.

### Example 2 — freeform memo
User: "Schreib mir ein Memo an das Team über die Umstellung des Rechnungsprozesses."
Plan:
- Ask briefly for missing info (effective date, responsible person) via `ask_user`.
- Draft memo structure: heading, background, changes, next steps.
- Generate `memo_2026-04-23_rechnungsprozess.docx`.
- Preview + approval.

## Anti-Patterns
- Do not overwrite existing files without explicit confirmation.
- Do not fabricate facts. If the user's input lacks a detail, ask via `ask_user` or insert a clear placeholder like `[noch einzutragen]`.
- Do not choose templates silently if multiple options exist.

## Notes for Maintainers
Placeholder syntax should follow `{{snake_case_key}}` by convention. `minijinja` (or equivalent) handles templating. Future versions may support conditional sections and loops; v1 supports simple substitution only.
