---
name: context-document-read
title: Kontext-Dokument lesen
description: Automatically load one or more designated context files (e.g. company info, customer list, style guide) at the beginning of every request so the model always has access to them without the user having to reference them.
icon: 📌
tools:
  - read_file
hitl:
  default: false
  per_tool: {}
language: en
---

# Skill: Kontext-Dokument lesen

## Purpose
Give the agent persistent, reliable background knowledge by pre-loading one or more user-designated files into the prompt before each request. Typical context: company profile, customer list, stylistic guidelines, reusable boilerplate, memory log from `document-extend`.

## When to Use
- The user wants the agent to "know" certain baseline facts in every session.
- There is a shared reference file whose content affects many tasks.
- Combined with `document-extend`, this forms a simple long-term memory loop: extend writes, context-read always reads.

## How to Use
This skill is configured per-agent with a list of file paths (relative to the agent folder). At the start of each ReAct loop:
1. Read each configured file via `read_file`.
2. Prepend a structured context block to the user's message: "Background (from `<file>`): <content>".
3. Skip silently if a file no longer exists, and log a warning in app logs.

Total size of context is capped (default 8k tokens across all context files); if exceeded, the skill truncates from the end and annotates "[truncated]".

## HITL Behavior
None.

## Notes for Maintainers
The file list is part of the agent configuration (`skillSettings["context-document-read"].files`). The Agent Editor UI must expose a multi-select for files in the agent folder.

In v1, no advanced reference resolution (no including-by-link or wildcards). Keep it literal: user picks specific files.
