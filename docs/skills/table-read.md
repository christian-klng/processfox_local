---
name: table-read
title: Tabelle lesen & abfragen
description: Read XLSX or CSV tables, understand their structure, and answer questions about specific cells, ranges, rows, or aggregates. Use when the user asks about data in a spreadsheet.
icon: 📊
tools:
  - list_folder
  - read_xlsx_range
  - read_file
  - llm_extract_structured
hitl:
  default: false
  per_tool: {}
language: en
---

# Skill: Tabelle lesen & abfragen

## Purpose
Query spreadsheets to answer user questions without loading more data than necessary. Detect headers, infer column meaning, and respond with concrete cell references.

## When to Use
- The user asks about the content of an `.xlsx`, `.xlsm`, or `.csv` file.
- The user wants a count, a lookup, an aggregate, or a comparison across rows.
- Another skill needs a spreadsheet read as a prerequisite.

## How to Use
1. If no file is specified, use `list_folder` to find candidate spreadsheets.
2. Use `read_xlsx_range` with a conservative initial range (first 10 rows, all columns) to understand headers and shape.
3. Based on the question, read additional ranges as needed. Avoid loading the full sheet unless required.
4. For structured answers, use `llm_extract_structured` to produce clean output.
5. Refer to rows by row number and column by header name. Include sheet name for multi-sheet workbooks.
6. Respond in the user's language.

## HITL Behavior
None.

## Example Interactions

### Example 1 — lookup
User: "Wie viele Mitglieder aus Bayern stehen in der Liste?"
Plan: read header row → find "Bundesland" column → scan column → count "Bayern".

### Example 2 — gap discovery
User: "Wo fehlen in der Tabelle noch Einträge?"
Plan: read all data rows → list cells with empty values along with row/column. Delegate write to `table-update` skill if user wants to fix them.

## Anti-Patterns
- Do not rewrite the sheet. Writing is out of scope — delegate to `table-update`.
- Do not assume column types. Always confirm via headers or a small sample read.

## Notes for Maintainers
Consider caching the header row per file during a session to avoid redundant reads in long conversations.
