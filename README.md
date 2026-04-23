# ProcessFox

**Lokale KI-Agenten für Einsteiger.**

ProcessFox ist eine plattformübergreifende Desktop-App (macOS, Windows, Linux), die kleinen Unternehmen, NGOs und Einzelnutzer:innen den Einstieg in die lokale Nutzung von KI-Sprachmodellen erleichtert. Statt eines komplexen Workflow-Builders setzt ProcessFox auf einfache, agentische Assistenten, die in einem vom Nutzer gewählten Ordner arbeiten und dort mit Dokumenten (DOCX, PDF, XLSX, CSV, MD, TXT) umgehen können.

Die App orientiert sich am Bedien-Paradigma von Obsidian: linke Sidebar mit Datei-Baum und Agenten-Dropdown, Chat als zentraler Interaktionsraum, klickbare Datei-Vorschau. Alle Daten bleiben standardmäßig lokal auf dem Rechner des Nutzers.

## Kern-Prinzipien

- **Lokal zuerst.** Lokale LLMs im GGUF-Format sind der Standard. Cloud-APIs (Anthropic, OpenAI, OpenRouter) sind optional hinterlegbar.
- **Agent statt Thread.** Die App kennt keine Chat-History-Sidebar. Alles lebt in benannten Agenten mit eigenem Ordner, Modell und Skill-Set.
- **Skills statt Workflows.** Fähigkeiten sind atomar und werden vom Agenten selbst ausgewählt — keine Prozessketten, die der Nutzer selbst bauen muss.
- **Einsteiger im Fokus.** Ein Einsteiger soll innerhalb von 5 Minuten nach Installation seine Dateien mit einem LLM bearbeiten können.
- **Regulatorisch vertretbar.** Strikte Ordner-Sandbox, kein Netzwerk-Zugriff für Skills in v1 außer den konfigurierten LLM-Endpunkten.

## Technologie

- Desktop: **Tauri v2**
- Frontend: **React + Vite + TypeScript**
- Backend: **Rust** (pure Rust, keine Python-Abhängigkeit)
- Lokale LLM-Runtime: **Rust-native** (candle oder mistral.rs, finale Wahl im ersten Prototypen)
- Distribution: **GitHub Releases** mit Auto-Updater via GitHub Actions

## Status

Sehr früh. v1.0-Konzept steht, Entwicklung beginnt. Siehe [CONCEPT.md](CONCEPT.md) für die vollständige Vision und [docs/roadmap.md](docs/roadmap.md) für die Phasen.

## Schnellstart für Mitentwickler

```bash
# Projekt initialisieren (einmalig, wenn kein src-tauri/ vorhanden)
npm create tauri-app@latest

# Danach
npm install
npm run tauri dev
```

Siehe [CLAUDE.md](CLAUDE.md) für Arbeits-Anweisungen, wenn du Claude Code zur Entwicklung nutzt.

## Lizenz

MIT — siehe [LICENSE](LICENSE).
