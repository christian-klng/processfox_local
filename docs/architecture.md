# ProcessFox — Technische Architektur

Dieses Dokument skizziert die technische Architektur von ProcessFox v1.0. Es ergänzt [`../CONCEPT.md`](../CONCEPT.md) um die Implementierungs-Sicht.

## 1. Systemüberblick

```
┌─────────────────────────────────────────────────────────────┐
│                    Tauri v2 Application                     │
│                                                             │
│  ┌────────────────────────┐      ┌────────────────────┐     │
│  │   Frontend (React)     │◄────►│  Backend (Rust)    │     │
│  │   - UI (Obsidian-like) │      │  - Agents          │     │
│  │   - Chat-Renderer      │ IPC  │  - ReAct-Loop      │     │
│  │   - File-Tree/Preview  │      │  - Tool-Registry   │     │
│  │   - Settings-Modal     │      │  - Skill-Registry  │     │
│  └────────────────────────┘      │  - LLM-Runtime     │     │
│                                  │  - Sandbox         │     │
│                                  │  - Storage         │     │
│                                  └────────────────────┘     │
└─────────────────────────────────────────────────────────────┘
                                             │
           ┌─────────────────────────────────┼──────────────────┐
           ▼                                 ▼                  ▼
   ┌───────────────┐               ┌────────────────┐    ┌─────────────┐
   │ Agent-Ordner  │               │ App-Support-   │    │ Cloud-LLM-  │
   │ (User-Files)  │               │ Ordner         │    │ Provider    │
   │ XLSX,DOCX,PDF │               │ agents/        │    │ (optional)  │
   │               │               │ skills/        │    │             │
   │               │               │ models/        │    │             │
   │               │               │ settings.json  │    │             │
   └───────────────┘               └────────────────┘    └─────────────┘
```

## 2. Datenfluss: ein Nutzer-Auftrag

```
User sendet Chat-Nachricht
      │
      ▼
Frontend ruft invoke("send_message", { agentId, message })
      │
      ▼
Backend: core::react_loop::run_loop(agent, message)
      │
      ├─► Lädt Agent, aktive Skills, Chat-Verlauf
      │
      ├─► Baut Prompt zusammen:
      │     [SystemPrompt] + [Skill-Descriptions] + [ChatVerlauf] + [UserMessage]
      │
      ├─► LLM-Provider.generate(...)
      │     ├─► Streamt TextDelta per event "chat/delta"
      │     └─► Bei ToolCall: Return-Event
      │
      ├─► Falls ToolCall:
      │     ├─► Tool-Registry: sucht Tool per Name
      │     ├─► Sandbox-Check (Pfad im Agent-Ordner?)
      │     ├─► Wenn schreibend & HITL: Event "hitl/request"
      │     │     └─► Frontend zeigt Inline-Diff-Karte, wartet auf User-Freigabe
      │     ├─► Tool::execute(...)
      │     ├─► Event "tool/status" mit Fortschritt
      │     └─► Ergebnis zurück in Loop
      │
      └─► Loop wiederholt, bis Finish oder Max-Iter
            │
            ▼
      Event "chat/finished"
            │
            ▼
      Frontend zeigt finale Antwort, persistiert Chat-Verlauf
```

## 3. Kern-Module (Rust)

### `core::agent`
Verwaltet Agent-Datensätze. CRUD auf `<app-support>/agents/<uuid>.json`. Lädt/speichert Chat-Verlauf als JSONL (append-only für Stabilität).

### `core::skill`
Scannt `src-tauri/skills_builtin/` und `<app-support>/skills/user/`. Parsed SKILL.md-Frontmatter via `gray_matter` + `serde_yaml`. Hält eine `SkillRegistry` im Speicher. Stellt Skills als System-Prompt-Fragment bereit (Name + Description + Tool-Liste).

### `core::tool`
Globale Tool-Registry. Trait-basiert: jedes Tool implementiert `trait Tool`. Stellt JSON-Schema für LLM-Function-Calling bereit. Dispatcher führt Tool-Calls aus, inklusive Sandbox-Check.

### `core::react_loop`
Orchestriert den Agent-Loop. Führt Chat-Iterationen, dispatcht Tool-Calls, emittiert Events für Frontend. Max-Iter-Sicherung (Default 12, in Agent-Config überschreibbar).

### `core::sandbox`
Zentrale Pfad-Validierung. Alle Tool-Input-Pfade laufen durch `ensure_in_agent_folder`. Verhindert Symlink-Ausbruch via `canonicalize`. Denylist für Spezialdateien.

### `core::storage`
Wissen über Plattform-spezifische App-Support-Pfade (`dirs` crate). Verwaltet `settings.json`, Modell-Katalog, Logs.

### `runtime::llm`
Abstraktion `trait LlmProvider`. Implementierungen:
- `LocalGgufProvider` — wraps candle oder mistral.rs
- `AnthropicProvider` — Messages-API
- `OpenAiProvider` — Chat-Completions
- `OpenRouterProvider` — OpenAI-kompatibel

Einheitliches Streaming-Event-Format:
```rust
enum LlmEvent {
    TextDelta(String),
    ToolCall { id: String, name: String, arguments: serde_json::Value },
    Finish { reason: FinishReason },
    Error(String),
}
```

**Lifecycle des lokalen Modells:** `LocalGgufProvider` hält genau ein Modell zur Zeit im RAM. Es bleibt zwischen Generations geladen, wird aber nach **10 Minuten ohne Aktivität** automatisch entladen — der RAM (mehrere GB plus KV-Cache) wird also nicht dauerhaft belegt, wenn die Nutzerin auf einen Cloud-Provider wechselt oder den Chat ruhen lässt. Ein Modellwechsel (anderer `filename`) triggert einen sofortigen Unload-und-Reload. Der nächste Prompt nach einem Idle-Unload kostet einmalig den Reload (~1–3 s SSD-Read bei einem 2-GB-Modell). Watcher-Implementierung in `core/llm/local_gguf.rs` (`ensure_idle_watcher`).

### `tools::*`
Einzelne Tool-Implementierungen:
- `list_folder` — Listet Dateien/Ordner (rekursiv, mit Filter).
- `read_file` — Liest Text-Datei (mit Größen-Limit).
- `grep_in_files` — Textsuche in mehreren Dateien via `ignore` + `grep-regex`.
- `read_pdf` — Via `pdfium-render`. Optional Page-Range.
- `read_docx` — Via `docx-rs`, extrahiert strukturierten Text.
- `read_xlsx_range` — Via `calamine`, gibt Bereich als 2D-Array.
- `write_docx` — Via `docx-rs` / Template-basiert.
- `write_docx_from_template` — Lädt Template, ersetzt Platzhalter via `minijinja`.
- `append_to_md` — Hängt Text an MD-/TXT-Datei an.
- `update_xlsx_cell` — Via `rust_xlsxwriter` (überschreibt oder erzeugt).
- `ask_user` — Erzeugt HITL-Event, wartet auf Antwort.
- `llm_extract_structured` — Ruft LLM mit Schema auf, für strukturierte Extraktion.

## 4. Kern-Module (Frontend)

### `views/Main.tsx`
Haupt-Layout. Dreispaltig (resizable): Sidebar, optional Preview, Chat.

### `components/agent/AgentDropdown.tsx`
Oben in Sidebar. Zeigt aktive Agent, Dropdown zum Wechsel, "Neuer Agent"-Eintrag.

### `components/agent/AgentSkillIcons.tsx`
Horizontal scrollbare Icon-Leiste unter dem Agent-Namen. Tooltip mit Skill-Name bei Hover.

### `components/filetree/FileTree.tsx`
Datei-Baum basiert auf `react-arborist`. Unterstützt Expand/Collapse, Icons nach Datei-Typ, Click-Handler für Preview.

### `components/preview/FilePreview.tsx`
Router basierend auf Datei-Endung:
- `.md`, `.txt` → `MarkdownEditor` (CodeMirror 6, editierbar)
- `.pdf` → `PdfViewer` (via `pdfjs-dist`)
- `.docx` → `DocxPreview` (via `mammoth.js` zu HTML, readonly)
- `.xlsx` → `XlsxPreview` (via SheetJS, tabellarisch)
- `.png`, `.jpg`, etc. → `ImageViewer`

### `components/chat/ChatView.tsx`
Scrollender Chat. Rendert User- und Agent-Messages, Tool-Call-Chips, HITL-Karten, Streaming-Text.

### `components/chat/ToolCallChip.tsx`
Kleiner Status-Chip (z. B. "🔍 ordner durchsuchen..." mit Spinner, dann "✓ 12 Dateien gefunden").

### `components/chat/HitlCard.tsx`
Inline-Freigabe-Karte. Zeigt Diff, Buttons "Freigeben", "Ablehnen", "Anpassen".

### `components/settings/SettingsModal.tsx`
Tabs: Modelle, Cloud-APIs, Sprache, Theme, About.

### `components/agent/AgentEditor.tsx`
Formular: Name, Icon, Ordner (File-Picker), System-Prompt (Textarea), Modell-Auswahl (Dropdown), Skills (Checkbox-Liste mit Icons).

### `components/skill/SkillEditor.tsx`
Formular zum Erstellen eigener Skills: Name, Beschreibung, Icon, Tool-Auswahl (Multi-Select aus verfügbaren Tools), HITL-Default.

## 5. Agent-Loop im Detail

```rust
pub async fn run_loop(
    agent: &Agent,
    user_message: String,
    app: tauri::AppHandle,
) -> Result<AssistantMessage> {
    let mut messages = load_chat_history(agent)?;
    messages.push(Message::User(user_message));

    let skills = load_active_skills(agent);
    let tools = build_tool_schemas(&skills);
    let system_prompt = compose_system_prompt(agent, &skills);

    let mut provider = select_llm_provider(agent);

    for iteration in 0..agent.max_iterations.unwrap_or(12) {
        let mut stream = provider
            .generate(&system_prompt, &messages, &tools)
            .await?;

        let mut tool_calls = Vec::new();
        let mut text_buffer = String::new();

        while let Some(event) = stream.next().await {
            match event? {
                LlmEvent::TextDelta(delta) => {
                    app.emit("chat/delta", &delta)?;
                    text_buffer.push_str(&delta);
                }
                LlmEvent::ToolCall { id, name, arguments } => {
                    tool_calls.push((id, name, arguments));
                }
                LlmEvent::Finish { reason } => {
                    if tool_calls.is_empty() {
                        messages.push(Message::Assistant(text_buffer));
                        persist_chat_history(agent, &messages)?;
                        return Ok(text_buffer.into());
                    }
                    break;
                }
                LlmEvent::Error(e) => return Err(Error::Llm(e)),
            }
        }

        for (id, name, args) in tool_calls {
            app.emit("tool/status", ToolStatus::running(&name))?;
            let tool = tool_registry.get(&name).ok_or(Error::UnknownTool)?;
            let ctx = ToolContext::new(agent, app.clone());
            let result = tool.execute(args, &ctx).await?;
            app.emit("tool/status", ToolStatus::done(&name))?;
            messages.push(Message::ToolResult { id, content: result });
        }
    }

    Err(Error::MaxIterationsReached)
}
```

## 6. Persistenz-Format

### `agents/<uuid>.json`
Siehe `CONCEPT.md` §7.

### `agents/<uuid>.chat.jsonl`
Append-only, eine Message pro Zeile:
```jsonl
{"role":"user","content":"Fasse mir die PDFs zusammen","timestamp":"2026-04-23T10:00:00Z"}
{"role":"assistant","content":"Ich schaue mir die Dateien an...","toolCalls":[{"id":"t1","name":"list_folder","args":{"path":"./"}}],"timestamp":"2026-04-23T10:00:02Z"}
{"role":"tool","toolCallId":"t1","content":"[...]"}
{"role":"assistant","content":"Ich habe 10 PDFs gefunden..."}
```

Vorteile JSONL: Append-only-sicher bei Crash, leicht tailbar für Debugging, einfach zu parsen im Frontend.

### `settings.json`
```json
{
  "language": "de",
  "theme": "system",
  "activeModelId": "google/gemma-4-e4b-gguf:Q4_K_M",
  "cloudProviders": {
    "anthropic": { "keyRef": "keychain://anthropic-key" },
    "openai": { "keyRef": "keychain://openai-key" }
  },
  "modelCatalogVersion": "2026-04-01"
}
```

## 7. LLM-Runtime-Wahl: candle vs. mistral.rs

Wird finale in Phase 2 entschieden. Benchmark-Kriterien:
- GGUF-Loading-Zeit.
- Tool-Calling-Robustheit mit Gemma 4 E4B (Referenz-Prompts).
- Streaming-Latenz.
- Speicherverbrauch.
- Cross-Platform-Build-Stabilität.

## 8. Sandbox-Model (v1)

### Pfad-Sandbox
- Jeder schreibende oder lesende Tool-Input-Pfad wird gegen den Agent-Ordner validiert (`core::sandbox::ensure_in_agent_folder`).
- Relative Pfade werden gegen Agent-Ordner aufgelöst.
- Absolute Pfade außerhalb des Agent-Ordners werden abgelehnt.
- Symlinks werden via `canonicalize` aufgelöst, der kanonische Pfad muss im Agent-Ordner liegen.

### Ausführungs-Sandbox (Infrastruktur vorbereitet)
- Eingebaute Skills, die Scripts ausführen müssten, laufen in einem begrenzten Rust-Kontext ohne Netzwerk-Zugriff.
- Für v1: kein User-Script-Support. Die Infrastruktur (Capability-System) wird aufgebaut, aber nicht exponiert.

## 9. Fehler-Handling-Strategie

- **Recoverable Errors** (z. B. Datei nicht gefunden, ungültiger Pfad, LLM-Timeout): werden in den Chat als `Assistant-Nachricht` gespielt ("Ich konnte die Datei X nicht öffnen: ...") und der Loop läuft weiter oder endet sauber.
- **Unrecoverable Errors** (z. B. Modell nicht geladen, Agent-Config korrupt): werden als Toast im UI angezeigt, Loop wird abgebrochen.
- **Logs:** Alles Schwere geht nach `<app-support>/logs/processfox.log` mit Timestamp, Agent-ID, Fehler-Details. Frontend hat einen "Logs öffnen"-Button in den Settings.

## 10. Performance-Ziele

- App-Start (ohne Modell-Load): ≤ 3 Sekunden Cold-Start.
- Modell-Load (GGUF, warm cache): ≤ 30 Sekunden.
- Tool-Call ohne LLM-Zeit: ≤ 1 Sekunde.
- Datei-Baum für Agent-Ordner mit 1000 Dateien: ≤ 500 ms.
- Chat-Nachricht senden bis erstes Token: ≤ 2 Sekunden (lokal), ≤ 4 Sekunden (Cloud).

## 11. Erweiterbarkeits-Punkte für spätere Versionen

Explizit offen gelassen im Design, damit spätere Features ohne Umbau andocken können:
- **Web-Skills:** Tool-Trait erlaubt in Zukunft HTTP-Tools; Tool-Context kann Capability-Flags tragen.
- **User-Scripts:** Sandbox-Module ist auf Capability-basiertes Modell ausgelegt, kein grundsätzlicher Umbau nötig.
- **Multi-Agent-Kollaboration:** Agent-IDs in Tool-Calls erlauben später Verweise auf andere Agenten.
- **Skill-Marketplace:** Skill-Ordner-Struktur erlaubt simple Download-/Installations-Flows.
- **Internationalisierung:** UI-Strings sind in `src/lib/strings.ts` zentralisiert, leicht auf i18n-Lib migrierbar.
