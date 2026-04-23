---
name: table-update
title: Tabelle aktualisieren
description: Update cells in an XLSX or CSV file. Use when the user asks to fix, fill, or change specific cells or ranges. Presents a detailed cell-by-cell diff before writing.
icon: 🧮
tools:
  - read_xlsx_range
  - update_xlsx_cell
  - llm_extract_structured
  - ask_user
hitl:
  default: true
  per_tool: {}
language: en
---

# Skill: Tabelle aktualisieren

## Purpose
Change specific cell values in a spreadsheet safely, with a detailed preview that shows exactly which cells change and from what to what.

## When to Use
- User asks to fill in missing values in a spreadsheet.
- User asks to correct wrong values.
- User wants to normalize or clean values (e.g. "alle Telefonnummern ins Format +49 ..." bringen).

## How to Use
1. Read the target sheet's headers and a sample of data to understand structure.
2. Identify affected cells. For fill-gaps tasks: scan the target column, list empty cells with their row.
3. For each affected cell, determine the proposed new value:
   - If derivable from context, propose confidently.
   - If ambiguous, either ask `ask_user` per cell (for few cells) or present a batch proposal (for many cells).
4. Build a structured change list: `[ { sheet, row, col, oldValue, newValue } ... ]`.
5. Emit HITL card with full change list. Support per-cell approval or bulk approval.
6. Apply approved changes via `update_xlsx_cell`.
7. Confirm in the user's language.

## HITL Behavior
Default: true. Always show the complete list of planned cell changes with old→new values. Users may approve all, reject all, or approve selectively.

A "without approval" variant may be enabled per-agent for experienced users; even then, a compact summary of changes is shown after write.

## Example Interactions

### Example 1 — fill gaps
User: "Meine Mitgliederliste hat Lücken — finde sie und schlag Werte vor."
Plan: read → identify empty cells per column → for "Postleitzahl" columns, propose based on "Ort" column if present; otherwise mark as "nicht herleitbar" → HITL.

### Example 2 — bulk normalization
User: "Einheitliches Datumsformat YYYY-MM-DD für Spalte 'Beitrittsdatum'."
Plan: read column → convert values → show change list → HITL.

## Anti-Patterns
- Do not silently change values you are not asked about.
- Do not overwrite an entire sheet. Changes are cell-scoped.
- Do not guess values without making the guess visible and approvable.

## Notes for Maintainers
XLSX writing preserves formulas and formatting in untouched cells. Tool `update_xlsx_cell` must not rewrite the file wholesale; use `rust_xlsxwriter` in incremental mode if possible, or use `calamine` + copy-modified-cells strategy.
