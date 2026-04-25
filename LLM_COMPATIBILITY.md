# LLM-Kompatibilität in ProcessFox

Welche lokalen LLMs sich in ProcessFox als „lokales Modell" laden lassen und wie zuverlässig sie für unseren Tool-Calling-Use-Case funktionieren.

Stand: April 2026. Lokale Inferenz läuft über `llama-cpp-2` (Rust-Bindings zu llama.cpp). Cloud-Provider (Anthropic, OpenAI, OpenRouter) sind hiervon nicht betroffen — sie nehmen jedes Modell, das ihre API kennt.

## Pflicht-Anforderungen (sonst lädt's nicht)

1. **Format: GGUF**. Andere Formate (SafeTensors, PyTorch, ONNX) werden nicht unterstützt. Beim Download prüfen wir die Magic-Bytes `GGUF` am Dateianfang; eine falsche Datei wird abgelehnt, bevor sie auf der Festplatte landet.
2. **Architektur kompatibel mit llama.cpp**. Konkret unterstützt: Llama (1/2/3/3.1/3.2), Gemma (2/3/4), Qwen (2/2.5/3), Mistral, Phi (2/3/3.5), Mixtral, DeepSeek, Falcon, MPT, GPT-NeoX, Starcoder, BLOOM und einige mehr. Die Liste wächst mit jedem llama.cpp-Release. Sehr neue Architekturen funktionieren erst nach einem ProcessFox-Update auf eine neuere `llama-cpp-2`-Version.
3. **Eingebettetes Chat-Template**. Wir rufen `apply_chat_template_oaicompat` auf, das den Template-String aus den GGUF-Metadaten liest. Faustregel: Modelle mit Suffix `-it`, `-instruct`, `-chat` haben Templates; reine `-base`-Modelle nicht. Ohne Template lehnt der Loader das Modell ab.
4. **Quantisierung**: alles, was llama.cpp kennt — Q2_K, Q3_K_*, Q4_0, Q4_K_*, Q5_K_*, Q6_K, Q8_0, IQ-Varianten, BF16, F16. **Standard-Sweet-Spot**: `Q4_K_M`.

## Soft-Anforderungen für gute UX

5. **Tool-Calling-fähiges Chat-Template** — ProcessFox' Kern. Im Jinja-Template muss der Tools-Slot vorhanden sein (`{%- if tools is defined %}` oder ähnlich), sonst werden die übergebenen Tools stillschweigend verworfen. Verifiziert: Llama 3+, Gemma 4, Qwen 2.5+, Mistral Nemo+, Hermes-3-Tunes, Phi-3.5+. Schlecht oder gar nicht: Gemma 2, Llama 2, einfache Mistral-7B-Instructs.
6. **Modellgröße passend zum RAM**. Faustregel Q4: Modellgröße auf Disk × 1.3 ≈ RAM-Bedarf. Plus 1–2 GB für KV-Cache. Plus OS, Browser, App. Für 8 GB Gesamt-RAM: Modelle bis ~3.5 GB Q4-Größe (≈ 5B Parameter total).
7. **Parameter-Größe ≥ 3B für Tool-Calling**. Empirisch: 1B-Modelle weigern sich oft selbstständig, Tools zu nutzen. 3B beginnt zu funktionieren (Llama 3.2 3B zaghaft, Hermes-3-3B zuverlässig). 7B+ ist verlässlich. **Gemma 4 E2B** (2.3B effektiv, 5.1B total) ist die zuverlässigste kleine Ausnahme — explizit auf Function-Calling trainiert.
8. **Sprach-Coverage** falls deutschsprachige Antworten wichtig sind. Schwächer: Llama 3.2 3B, kleine Phi-Varianten. Stärker: Gemma-Familie, Qwen 2.5+, Mistral-Familie.

## Sanity-Checks für eine vom Nutzer eingegebene HuggingFace-URL

Vor dem Download solltest du visuell prüfen:

- URL endet auf `.gguf` (sonst lehnt unser Custom-URL-Tool sie ab)
- Im Repo-Namen oder Modellnamen taucht **eines** dieser Wörter auf: `instruct`, `it`, `chat`, `hermes`, `functionary` — Indiz für vorhandenes Chat-Template
- Auf der HF-Modellkarte steht „compatible with llama.cpp" oder ähnliche llama.cpp-Erwähnung
- Quantisierung **Q4_K_M** oder besser. Q3 nur akzeptabel wenn RAM knapp ist und Qualität egal.
- Modellgröße passt zum RAM (siehe Faustregel oben)

## Bekannte Problemfälle

- **Reines Base-Modell ohne Instruction-Tuning** → kein Chat-Template, lädt aber halluziniert sofort oder bricht ab. Nutze nie Repos mit Endung `-base` als Chat-Modell.
- **Multimodale Modelle** (Vision, Audio) → laden teilweise, nutzen aber nur den Text-Pfad. Bilder/Audio werden ignoriert.
- **Sehr neue Architekturen** (jünger als die llama.cpp-Version, gegen die wir gebaut wurden) → Loader meldet `Unknown architecture`. Lösung: ProcessFox-Update abwarten.
- **MoE-Forschungsmodelle** mit eigenwilligen Sparsity-Layouts → oft nicht in llama.cpp portiert.

## Reasoning-Modelle (Chain-of-Thought)

Modelle wie **Gemma 4** (mit `<|channel>thought`-Tags), **DeepSeek-R1** (`<think>`-Blöcke) oder **OpenAI-o1-Klone** geben ihren Denkprozess in einem separaten Kanal aus. ProcessFox extrahiert das automatisch (`reasoning_format=auto`) und zeigt's als kollabierbaren „Gedanken"-Chip im Chat — getrennt vom finalen Text. Das funktioniert ohne Konfiguration, sobald das Modell den Standardpfad nutzt.

**Sprach-Hinweis**: Die meisten Reasoning-Modelle denken intern auf Englisch und antworten dann in der Sprache des Users. Das ist kein Bug, sondern Folge der englisch-lastigen Reasoning-Trainingsdaten.

## Verifizierte Modelle (Stand April 2026)

| Modell | Größe Q4 | Tool-Calling | Reasoning | Anmerkung |
|---|---|---|---|---|
| Gemma 4 E2B Instruct | 3.46 GB | ✅ zuverlässig | ja | Beste Wahl für 8–16 GB RAM mit Tool-Calling |
| Llama 3.2 3B Instruct | 2.0 GB | ⚠ zaghaft | nein | Klein, schnell, aber lehnt Tools oft ab |
| Hermes 3 Llama 3.2 3B | 2.0 GB | ✅ zuverlässig | nein | Llama-Tune speziell für Function-Calling |
| Qwen 2.5 7B Instruct | 4.4 GB | ✅ zuverlässig | nein | Robust für die meisten Aufgaben |
| Gemma 2 9B Instruct | 5.5 GB | ⚠ schwach | nein | Sehr gute deutsche Sprache, aber kein Tool-Slot im Template |

Alle Werte oben gelten für `Q4_K_M`. Bei IQ3- oder Q3-Quantisierung sinkt die Tool-Calling-Verlässlichkeit messbar.

## Was tun, wenn ein Modell nicht funktioniert

1. **Lädt nicht**: prüfe Architektur (siehe Liste oben) und ob das Repo Chat-Template-Tags im Namen hat.
2. **Lädt, aber antwortet komisch**: vermutlich Base-Modell ohne Instruction-Tuning oder fehlendes Chat-Template. Anderes Repo desselben Modells in Instruct-Variante suchen.
3. **Lädt, antwortet gut, aber ignoriert Tools**: Modell zu klein oder Chat-Template ohne Tools-Slot. Auf größere Variante (7B+) oder Hermes-Tune wechseln.
4. **Lädt, ist aber spürbar zäh**: Q3 oder noch kleiner ausprobieren — auf Kosten der Antwortqualität — oder ein kleineres Basismodell wählen.
