# ProcessFox — Roadmap

Dieses Dokument bricht den Weg zu v1.0 in sechs Phasen herunter. Jede Phase endet mit einem funktionsfähigen, testbaren Zwischenstand. Nach jeder Phase wird in `main` gemerged.

## Phase 1 — Gerüst (1–2 Wochen)

**Ziel:** Die App startet, zeigt die UI-Struktur, aber kann noch nichts "Echtes".

### Arbeit
- [ ] Tauri v2 Projekt initialisieren (`npm create tauri-app@latest` mit React + TS + Vite)
- [ ] Basis-Layout: dreispaltig (Sidebar, optionaler Preview, Chat-Bereich) mit resizable Panels
- [ ] Agent-Dropdown in Sidebar oben (statisch mit Mock-Daten)
- [ ] Datei-Baum (`react-arborist`) mit Mock-Inhalt
- [ ] Leerer Chat-Bereich mit Textarea unten und "Senden"-Button
- [ ] Rust: Datenmodelle für `Agent`, `Message`, `Skill`, `ToolSchema`
- [ ] Rust: CRUD-Commands für Agenten, Persistenz in `<app-support>/agents/`
- [ ] Rust: `core::storage` mit plattform-spezifischen Pfaden
- [ ] Settings-Modal-Shell mit Tabs (leer): "Modelle", "Cloud-APIs", "Sprache", "Über"
- [ ] Agent-Editor als Modal: Name, Ordner (File-Picker via Tauri-Dialog), System-Prompt, leere Skill-Liste
- [ ] Tailwind + shadcn/ui Setup mit Basis-Theme (Hell & Dunkel, System-Default)
- [ ] GitHub Actions: "CI" Workflow (Rust + Frontend Build-Check, keine Releases)

### Akzeptanzkriterien
- App startet auf macOS, Windows, Linux (dev-mode mindestens auf macOS und einer zweiten Plattform getestet).
- Nutzer kann einen Agenten anlegen und sieht dessen Ordner-Inhalt im Baum.
- Datei-Klick im Baum zeigt Dateinamen im Mittelbereich (noch keine Preview).
- Chat-Textarea akzeptiert Eingaben, "Senden" zeigt die Nachricht als User-Bubble an.
- Settings-Modal öffnet und schließt sauber.

## Phase 2 — LLM-Anbindung (1–2 Wochen)

**Ziel:** Chat funktioniert mit lokalem GGUF-Modell, noch ohne Skills.

### Arbeit
- [ ] Benchmark `candle` vs. `mistral.rs` mit Gemma 4 E4B und einem Referenz-Prompt
- [ ] Entscheidung und Implementierung `LocalGgufProvider`
- [ ] Trait `LlmProvider` mit einheitlichem Streaming-Event-Format
- [ ] `AnthropicProvider`, `OpenAiProvider`, `OpenRouterProvider` (Cloud-optional)
- [ ] API-Key-Storage via Tauri Stronghold oder `keyring`
- [ ] Modell-Download-Flow: HuggingFace-URL, GGUF-Validierung, Progress-Bar, Speicherung
- [ ] Kurator-JSON `models/catalog.json` im Repo mit initial 3–5 empfohlenen Modellen
- [ ] Settings-Tab "Modelle": Katalog-Dropdown + Custom-URL + Download + Liste geladener Modelle
- [ ] First-Run-Detection: Settings-Modal öffnet sich automatisch beim ersten Start
- [ ] Hardware-Check (RAM-Detection, einfache VRAM-Heuristik) mit Modell-Vorschlag
- [ ] Chat sendet Nachrichten an aktiven Provider, streamt Antworten in UI
- [ ] Chat-Verlauf persistieren (`<uuid>.chat.jsonl`)

### Akzeptanzkriterien
- Nutzer kann im Settings-Modal ein GGUF-Modell herunterladen.
- "Hallo, wer bist du?" vom User wird vom Modell beantwortet, Antwort streamt live in den Chat.
- Chat-Verlauf wird persistiert und beim Neustart wiederhergestellt.
- Cloud-Provider (mindestens Anthropic) funktioniert alternativ, wenn API-Key hinterlegt.
- Modell-Wechsel im laufenden Betrieb funktioniert (Entladen + Neuladen).

## Phase 3 — Tool-System + lesende Skills (1–2 Wochen)

**Ziel:** Agent kann Dateien lesen und Antworten darauf basieren.

### Arbeit
- [ ] `trait Tool` + `ToolRegistry` + JSON-Schema-Export für LLM-Function-Calling
- [ ] `core::sandbox::ensure_in_agent_folder` + Unit-Tests (Symlink-Escape, Path-Traversal)
- [ ] Tools: `list_folder`, `read_file`, `grep_in_files`, `read_pdf`, `read_docx`, `read_xlsx_range`
- [ ] Skill-Loader: scannt `skills_builtin/`, parst SKILL.md, baut `SkillRegistry`
- [ ] Prompt-Composer: baut System-Prompt aus Skill-Descriptions + Agent-SystemPrompt
- [ ] Skill: `folder-search` (siehe `docs/skills/folder-search.md`)
- [ ] Skill: `document-read`
- [ ] Skill: `table-read`
- [ ] Skill: `chat-context`
- [ ] Skill: `context-document-read`
- [ ] ReAct-Loop-Implementierung mit Max-Iter-Sicherung
- [ ] Tool-Call-Chips im Chat (Status: running, done, error)
- [ ] Skill-Auswahl im Agent-Editor: Checkbox-Liste aller verfügbaren Skills
- [ ] Skill-Icons unter Agent-Namen im UI
- [ ] JSON-Cleanup-Layer für Tool-Call-Outputs kleiner Modelle

### Akzeptanzkriterien
- Nutzer erstellt Agenten mit Ordner "~/TestPdfs" und 5 PDFs.
- Aktiviert Skills "folder-search" und "document-read".
- Frage: "Welche Dokumente sprechen über Thema X?" führt zu sichtbaren Tool-Calls (`list_folder`, `read_pdf` ×N, eventuell `grep_in_files`) und liefert eine sinnvolle Antwort mit Datei-Referenzen.
- Datei-Preview im Chat per Klick auf referenzierte Datei öffnet sie in der mittleren Spalte.
- Sandbox-Verletzung (Versuch, außerhalb des Agent-Ordners zu lesen) wird als Fehler-Chip angezeigt, Loop bricht sauber ab.

## Phase 4 — Schreibende Skills + HITL (1–2 Wochen)

**Ziel:** Agent kann Dateien erzeugen und ändern, mit Inline-Freigabe.

### Arbeit
- [ ] Tools: `write_docx`, `write_docx_from_template`, `append_to_md`, `update_xlsx_cell`, `llm_extract_structured`, `ask_user`
- [ ] HITL-Mechanik: Tool kann eine Freigabe anfordern, ReAct-Loop pausiert
- [ ] Frontend: `HitlCard`-Komponente mit Diff-Darstellung
  - [ ] Datei-Erstellung: volle Inhalt-Vorschau
  - [ ] Datei-Bearbeitung: Zeilen-Diff (grün/rot)
  - [ ] XLSX-Update: Liste der geplanten Zellen-Änderungen
- [ ] HITL-Flags in SKILL.md-Frontmatter umsetzen, pro-Agent-Override im Agent-Editor
- [ ] Skill: `document-create-docx`
- [ ] Skill: `document-edit`
- [ ] Skill: `document-extend`
- [ ] Skill: `table-update`
- [ ] Template-Handling: Nutzer kann in Agent-Ordner `.docx`-Templates ablegen, Skill findet sie und nutzt Platzhalter
- [ ] Tauri-File-Watcher: Datei-Baum aktualisiert sich live bei Änderungen im Agent-Ordner

### Akzeptanzkriterien
- Referenz-Use-Case "E-Mail → Angebot" läuft: Nutzer paste-t E-Mail, Agent nutzt Template, füllt Felder, zeigt Preview, User gibt frei, DOCX wird geschrieben.
- Referenz-Use-Case "Excel-Lücken füllen": Agent identifiziert leere Zellen, schlägt Werte vor, zeigt Diff-Karte pro Zelle oder gebündelt.
- Ablehnung der HITL-Karte führt zu "ich habe nichts geändert"-Antwort des Agenten und Fortsetzung des Dialogs.
- Bei aktivierter "ohne Rückfrage"-Variante läuft die Aktion direkt durch, Ergebnis wird prominent bestätigt.

## Phase 5 — Polish & Onboarding (1 Woche)

**Ziel:** Die App fühlt sich fertig an.

### Arbeit
- [ ] First-Run-Flow: Willkommen → Modell-Download → erster Agent → Tutorial-Chips
- [ ] Starter-Chips im leeren Chat ("Probier mal: ...")
- [ ] Skill-Editor-UI für User-erstellte Skills (Formular, kein Markdown-Editor)
- [ ] Tastatur-Shortcuts: Cmd/Ctrl+N (Neuer Agent), Cmd/Ctrl+, (Settings), Cmd/Ctrl+Enter (Senden)
- [ ] Fehler-Toasts mit "Logs öffnen"-Button
- [ ] Onboarding-Banner: "Für bessere deutsche Qualität: Modell XY empfohlen"
- [ ] Modell-Empfehlungs-Mitteilung, wenn aktives Modell veraltet
- [ ] Drag-and-Drop von Dateien in den Chat (erzeugt einen Inline-Verweis, der den Agent auf die Datei fokussiert)
- [ ] Copy-Button für Agent-Antworten
- [ ] Diverse Usability-Tests (subjektiv mit 2–3 Testpersonen durchspielen)

### Akzeptanzkriterien
- Erfolgs-Kriterium: ein Einsteiger kann ≤ 5 Minuten nach Installation (inkl. Download eines kleinen Modells) seine erste Frage beantwortet bekommen.
- Alle drei Referenz-Use-Cases sind mit Gemma 4 E4B lokal reproduzierbar.
- Kein Absturz in typischen Nutzungs-Pfaden; nicht-reproduzierbare Bugs sind dokumentiert.

## Phase 6 — Release (1 Woche)

**Ziel:** v1.0.0 auf GitHub Releases, Mac/Win/Linux installierbar.

### Arbeit
- [ ] GitHub Actions `release.yml`: Build-Matrix (macOS, Windows, Linux), Tauri-Bundler
- [ ] Release auf Tag-Push (`v*.*.*`) getriggert
- [ ] Tauri-Updater-Konfiguration (Public-Key im Repo, Signing-Key als GitHub Secret)
- [ ] Release-Notes-Template
- [ ] README mit Download-Links, Screenshots, Quickstart-Anleitung
- [ ] Bekannte Sicherheits-Warnungs-Hinweise dokumentieren (weil noch kein Code-Signing)
- [ ] Post-Release: Issue-Templates, CONTRIBUTING.md, Bug-Report-Template
- [ ] Beta-Tester anschreiben (3–5 Personen aus Netzwerk), Feedback-Kanal (Issues oder Discord)

### Akzeptanzkriterien
- Tag `v1.0.0` triggert Build, Release enthält Artefakte für alle drei Plattformen.
- Installation auf einem jungfräulichen Test-Rechner (VM) führt zur funktionsfähigen App.
- Auto-Updater findet den nächsten Release (getestet mit Point-Release `v1.0.1`).
- Mindestens ein externer Beta-Tester hat erfolgreich einen der drei Referenz-Use-Cases durchgespielt.

## Nach v1.0

Mögliche v1.1+ Themen — Priorität wird nach v1.0-Feedback entschieden:
- Code-Signing (Apple Developer, Windows-EV-Zertifikat)
- Web-Skills (HTTP-Fetch, Suchmaschinen-Integration)
- Skill-Marketplace (Public Index von Community-Skills)
- Englische UI
- Audio-Transkription via Whisper
- OCR auf gescannten PDFs
- Multi-Agenten-Kollaboration
- Auto-Komprimierung langer Chat-Verläufe
- Mobile/Tablet-Variante (iPadOS via Tauri Mobile später)
