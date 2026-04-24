//! Benchmark the candle runtime on a local GGUF model.
//!
//! Measures:
//! - Load time (from `file::open` through `ModelWeights::from_gguf`)
//! - Prompt-processing time (first-token latency)
//! - Generation throughput (tokens/sec over `--max-tokens` generated tokens)
//!
//! Prints progress text to stderr and a final JSON summary to stdout.

use std::io::Write;
use std::path::PathBuf;
use std::time::Instant;

use anyhow::{Context, Result};
use candle_core::quantized::gguf_file;
use candle_core::{Device, Tensor};
use candle_transformers::generation::{LogitsProcessor, Sampling};
use candle_transformers::models::quantized_llama::ModelWeights;
use clap::Parser;
use serde::Serialize;
use tokenizers::Tokenizer;

#[derive(Parser, Debug)]
#[command(about = "Benchmark candle GGUF inference")]
struct Args {
    /// Path to the .gguf model file.
    #[arg(long)]
    model: PathBuf,

    /// Path to the tokenizer.json file matching the model.
    #[arg(long)]
    tokenizer: PathBuf,

    /// Prompt to feed the model.
    #[arg(long)]
    prompt: String,

    /// Number of tokens to generate.
    #[arg(long, default_value_t = 200)]
    max_tokens: usize,

    /// Sampling temperature. 0.0 = greedy.
    #[arg(long, default_value_t = 0.7)]
    temperature: f64,

    /// Top-p nucleus sampling parameter.
    #[arg(long, default_value_t = 0.9)]
    top_p: f64,

    /// Random seed.
    #[arg(long, default_value_t = 299792458)]
    seed: u64,

    /// Force CPU even if Metal is available.
    #[arg(long, default_value_t = false)]
    cpu: bool,
}

#[derive(Debug, Serialize)]
struct Summary {
    runtime: &'static str,
    load_seconds: f64,
    prompt_tokens: usize,
    first_token_seconds: f64,
    generated_tokens: usize,
    generation_seconds: f64,
    tokens_per_second: f64,
}

fn pick_device(cpu: bool) -> Result<Device> {
    if cpu {
        return Ok(Device::Cpu);
    }
    match Device::new_metal(0) {
        Ok(dev) => Ok(dev),
        Err(e) => {
            eprintln!("(metal unavailable, falling back to CPU: {e})");
            Ok(Device::Cpu)
        }
    }
}

fn main() -> Result<()> {
    let args = Args::parse();
    let device = pick_device(args.cpu)?;
    eprintln!("device: {device:?}");

    // --- Load ---
    let load_start = Instant::now();
    let mut file = std::fs::File::open(&args.model)
        .with_context(|| format!("opening model {:?}", args.model))?;
    let content = gguf_file::Content::read(&mut file)?;
    let tensor_count = content.tensor_infos.len();
    let mut model = ModelWeights::from_gguf(content, &mut file, &device)?;
    let load_seconds = load_start.elapsed().as_secs_f64();
    eprintln!(
        "loaded {tensor_count} tensors in {load_seconds:.2}s",
    );

    // --- Tokenize prompt ---
    let tokenizer = Tokenizer::from_file(&args.tokenizer).map_err(anyhow::Error::msg)?;
    let encoded = tokenizer
        .encode(args.prompt.as_str(), true)
        .map_err(anyhow::Error::msg)?;
    let prompt_tokens: Vec<u32> = encoded.get_ids().to_vec();
    let prompt_len = prompt_tokens.len();
    eprintln!("prompt tokens: {prompt_len}");

    let mut logits_processor = LogitsProcessor::from_sampling(
        args.seed,
        if args.temperature <= 0.0 {
            Sampling::ArgMax
        } else {
            Sampling::TopP {
                p: args.top_p,
                temperature: args.temperature,
            }
        },
    );

    // --- Prompt processing (one forward pass for the whole prompt) ---
    let prompt_start = Instant::now();
    let input = Tensor::new(prompt_tokens.as_slice(), &device)?.unsqueeze(0)?;
    let logits = model.forward(&input, 0)?;
    let logits = logits.squeeze(0)?;
    let mut next_token = logits_processor.sample(&logits)?;
    let first_token_seconds = prompt_start.elapsed().as_secs_f64();
    eprintln!("first token after {first_token_seconds:.2}s");

    // Try to detect an EOS token for early exit. Llama 3.x uses <|end_of_text|>
    // or <|eot_id|>. Fallback: generate the full budget.
    let vocab = tokenizer.get_vocab(true);
    let eos_token = ["<|eot_id|>", "<|end_of_text|>", "</s>"]
        .iter()
        .find_map(|t| vocab.get(*t).copied());

    // --- Generation loop ---
    let mut generated: Vec<u32> = Vec::with_capacity(args.max_tokens);
    generated.push(next_token);

    let gen_start = Instant::now();
    for step in 0..args.max_tokens.saturating_sub(1) {
        let input = Tensor::new(&[next_token], &device)?.unsqueeze(0)?;
        let logits = model.forward(&input, prompt_len + step + 1)?;
        let logits = logits.squeeze(0)?;
        next_token = logits_processor.sample(&logits)?;
        generated.push(next_token);
        if Some(next_token) == eos_token {
            eprintln!("(eos at step {step})");
            break;
        }
    }
    let generation_seconds = gen_start.elapsed().as_secs_f64();

    // Print generated text to stderr for eyeball-inspection.
    if let Ok(text) = tokenizer.decode(&generated, true) {
        eprintln!("--- generated ---\n{text}\n--- end ---");
    }

    let summary = Summary {
        runtime: "candle",
        load_seconds: round2(load_seconds),
        prompt_tokens: prompt_len,
        first_token_seconds: round2(first_token_seconds),
        generated_tokens: generated.len(),
        generation_seconds: round2(generation_seconds),
        tokens_per_second: round2(generated.len() as f64 / generation_seconds),
    };

    let json = serde_json::to_string_pretty(&summary)?;
    println!("{json}");
    std::io::stdout().flush()?;
    Ok(())
}

fn round2(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}
