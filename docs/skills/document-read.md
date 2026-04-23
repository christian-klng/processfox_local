---
name: document-read
title: Dokument lesen & zusammenfassen
description: Read a single document (PDF, DOCX, MD, TXT) and summarize it or extract specific information on request. Use when the user references a single file and asks about its contents.
icon: 📄
tools:
  - read_file
  - read_pdf
  - read_docx
  - llm_extract_structured
hitl:
  default: false
  per_tool: {}
language: en
---

# Skill: Dokument lesen & zusammenfassen

## Purpose
Read one document and answer questions about it. Handle paging for long documents, structure preservation, and targeted extraction.

## When to Use
- The user points at a single file ("fasse mir `angebot.pdf` zusammen").
- The user asks a specific question about a known document.
- Another skill delegates single-document reading.

## How to Use
1. Determine file type by extension.
2. Use the matching reader tool: `read_pdf` for PDF, `read_docx` for DOCX, `read_file` for MD/TXT.
3. If the file is long, read in sections (PDF: by page range; DOCX: structural chunks).
4. For structured questions ("what is the due date, who is the customer"), use `llm_extract_structured` with an appropriate schema.
5. Answer in the user's language. Quote relevant passages when helpful.

## HITL Behavior
None.

## Example Interactions

### Example 1 — summarization
User: "Fasse mir das Angebot.pdf zusammen."
Plan: `read_pdf` for the whole file (or paged if > 50 pages) → summarize in German.

### Example 2 — targeted extraction
User: "Was ist das Lieferdatum in bestellung.docx?"
Plan: `read_docx` → `llm_extract_structured` with schema `{ delivery_date: string }`.

## Anti-Patterns
- Do not try to read binary images/scans without a real reader tool.
- Do not make up answers if the file doesn't contain the information.

## Notes for Maintainers
OCR is NOT part of this skill in v1. Scanned PDFs will return minimal text. Document this limitation in user-facing error messages.
