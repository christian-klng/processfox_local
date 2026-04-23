---
name: folder-search
title: Ordner durchsuchen
description: Search and scan files in the agent folder by name, extension, and content. Use when the user asks what is in their folder or wants to find files mentioning a topic.
icon: 🔍
tools:
  - list_folder
  - read_file
  - grep_in_files
  - llm_extract_structured
hitl:
  default: false
  per_tool: {}
language: en
---

# Skill: Ordner durchsuchen

## Purpose
Help the agent explore the user's folder efficiently. Combine directory listing, filtered reads, and content search to answer questions like "what PDFs do I have about topic X" or "find any document that mentions company Y".

## When to Use
- The user asks to find or scan files.
- The user asks a question whose answer might lie in multiple documents.
- Before running another skill that needs to know which files exist.

## How to Use
1. Start with `list_folder` at the agent root to get a map of available files (respect reasonable depth, default 3 levels).
2. Filter to relevant extensions (PDF, DOCX, XLSX, MD, TXT, CSV) based on the user's question.
3. If searching by keyword: use `grep_in_files` with a concise regex or keyword list.
4. For candidate matches, use `read_file` or a dedicated reader tool (`read_pdf`, `read_docx`) to confirm relevance.
5. Summarize findings with clear file references (relative paths from the agent root).
6. Respond in the user's language.

## HITL Behavior
None. This skill only reads.

## Example Interactions

### Example 1 — file discovery
User: "What PDFs do I have in here that talk about funding?"
Plan: list root → filter to `.pdf` → grep for "Förder|funding|Zuschuss" → read matches briefly.
Result: "Ich habe drei PDFs gefunden, die Förder-Themen behandeln: `..."

### Example 2 — structured scan
User: "Find me every Excel row that references customer 'Müller GmbH'."
Plan: list root → filter to `.xlsx` → delegate to `table-read` skill or use `grep_in_files` with a lenient match, then follow up with a spreadsheet read.

## Anti-Patterns
- Do not read every file end-to-end before narrowing down — that is wasteful.
- Do not search outside the agent folder. The sandbox will block you anyway.
- Do not invent file contents when unsure. Cite the file and quote.

## Notes for Maintainers
This is the "discovery" skill. Keep it atomic and robust. If you find yourself wanting file-type-specific extraction logic, push it into a dedicated skill instead.
