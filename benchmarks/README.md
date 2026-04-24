# ProcessFox — GGUF-Runtime-Benchmark

Zwei unabhängige Mini-Binaries, die dieselbe GGUF-Datei gegen dieselbe
Frage laufen lassen und die Zahlen ausspucken, die wir für die
Runtime-Entscheidung in Phase 2 brauchen: Load-Zeit, First-Token-Latenz,
Tokens/Sekunde.

Beide Crates sind **nicht Teil der Haupt-App** — sie liegen hier, damit sie
ihre großen Dependencies (candle / mistral.rs) getrennt von `src-tauri/`
kompilieren. Wenn du dich für eine Runtime entschieden hast, wird sie in
`src-tauri/src/core/llm/local_gguf.rs` eingebaut und die Benchmarks können
gelöscht werden.

## Vorbereitung

1. **Modell** — du hast es schon über die App gezogen (z. B.
   `~/Library/Application Support/ProcessFox/models/downloads/Llama-3.2-3B-Instruct-Q4_K_M.gguf`).
2. **Tokenizer** (nur für `bench-candle` nötig) — lädt das zum Modell
   passende `tokenizer.json` von HuggingFace:
   ```sh
   curl -L -o /tmp/llama-3.2-3b-tokenizer.json \
     https://huggingface.co/meta-llama/Llama-3.2-3B-Instruct/resolve/main/tokenizer.json
   ```
   Falls Meta-Gating stört:
   ```sh
   curl -L -o /tmp/llama-3.2-3b-tokenizer.json \
     https://huggingface.co/unsloth/Llama-3.2-3B-Instruct/resolve/main/tokenizer.json
   ```

## candle

```sh
cd benchmarks/bench-candle

cargo run --release -- \
  --model "$HOME/Library/Application Support/ProcessFox/models/downloads/Llama-3.2-3B-Instruct-Q4_K_M.gguf" \
  --tokenizer /tmp/llama-3.2-3b-tokenizer.json \
  --prompt "Schreib mir in drei Sätzen, warum lokale KI-Modelle datenschutzfreundlich sind." \
  --max-tokens 200
```

Erster Build kompiliert candle-core + candle-transformers (~3–5 Minuten auf
Apple Silicon).

## mistral.rs

```sh
cd benchmarks/bench-mistralrs

cargo run --release -- \
  --model-dir "$HOME/Library/Application Support/ProcessFox/models/downloads" \
  --filename Llama-3.2-3B-Instruct-Q4_K_M.gguf \
  --prompt "Schreib mir in drei Sätzen, warum lokale KI-Modelle datenschutzfreundlich sind." \
  --max-tokens 200
```

Erster Build zieht mistralrs inkl. Metal-Kernels (~5–10 Minuten, je nach
Maschine).

## Zahlen, die wir wollen

Jeder Benchmark druckt am Ende einen JSON-Block wie:

```json
{
  "runtime": "candle",
  "load_seconds": 4.21,
  "prompt_tokens": 28,
  "first_token_seconds": 0.93,
  "generated_tokens": 200,
  "generation_seconds": 11.72,
  "tokens_per_second": 17.07
}
```

Schick mir beide Ausgaben, dann entscheiden wir welche Runtime in die App
fest eingebaut wird.
