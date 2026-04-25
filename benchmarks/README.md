# ProcessFox — GGUF-Runtime-Benchmark (historisch)

Diese Benchmark-Crate ist ein **historisches Artefakt** aus Phase 2c, als
wir zwischen drei Runtime-Optionen entscheiden mussten:

- **candle** (HuggingFace, pure Rust) — gemessen, lief, aber kein
  Chat-Template-Handling
- **mistral.rs** — gewählt, später wieder ersetzt, weil keine Gemma-Family
  im GGUF-Loader
- **llama.cpp via `llama-cpp-2`** — der letztgewählte Pfad und das, was die
  App heute nutzt

Die `bench-candle`-Crate liegt hier nur noch als Vergleichsreferenz; sie
spielt im Live-Build der Haupt-App keine Rolle mehr.

## Run

```sh
cd benchmarks/bench-candle

cargo run --release -- \
  --model "$HOME/Library/Application Support/ProcessFox/models/downloads/Llama-3.2-3B-Instruct-Q4_K_M.gguf" \
  --tokenizer /tmp/llama-3.2-3b-tokenizer.json \
  --prompt "Schreib mir in drei Sätzen, warum lokale KI-Modelle datenschutzfreundlich sind." \
  --max-tokens 200
```

Den Tokenizer holst du dir mit:

```sh
curl -L -o /tmp/llama-3.2-3b-tokenizer.json \
  https://huggingface.co/unsloth/Llama-3.2-3B-Instruct/resolve/main/tokenizer.json
```
