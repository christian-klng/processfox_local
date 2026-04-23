---
name: chat-context
title: Gesprächsverlauf einbeziehen
description: Make earlier messages in the current chat available to the model as context. Enable for long conversations where the model should remember what has been discussed earlier in this session.
icon: 💬
tools: []
hitl:
  default: false
  per_tool: {}
language: en
---

# Skill: Gesprächsverlauf einbeziehen

## Purpose
Expand the effective context window by including the full (or windowed) chat history of the current session in each LLM call. Without this skill active, ProcessFox sends only the current user message plus system prompt to save tokens and latency on short interactions.

## When to Use
- Long-running sessions where past steps matter.
- Multi-turn reasoning where the user builds on prior answers.
- Agents that should feel like "they remember our conversation".

## How to Use
This skill does not invoke tools. Its presence in the active skill list changes how the ReAct-Loop composes the prompt:
- Without this skill: last N turns only (default 3).
- With this skill: all turns from the current session, truncated at the model's context budget using a sliding window from the tail.

The skill description in the system prompt is a simple hint to the model that it may rely on earlier parts of the conversation.

## HITL Behavior
None.

## Notes for Maintainers
This is a "pseudo-skill" — it has no tools and serves as a toggle. Consider renaming internally to make the distinction clear (e.g. `ContextAugmentation` flag on the agent) if the Skill abstraction proves too heavy for pure toggles.

In later versions, a summarization skill could compress old turns, but in v1 we do raw windowing only.
