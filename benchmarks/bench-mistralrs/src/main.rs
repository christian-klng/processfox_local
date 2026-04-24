//! Benchmark the mistral.rs runtime on a local GGUF model.
//!
//! Measures:
//! - Load time (from process start through `GgufModelBuilder::build`)
//! - First-token latency (time until the first streamed chunk arrives)
//! - Generation throughput (tokens/sec)
//!
//! Prints progress text to stderr and a final JSON summary to stdout.

use std::io::Write;
use std::time::Instant;

use anyhow::Result;
use clap::Parser;
use mistralrs::{
    ChatCompletionChunkResponse, ChunkChoice, Delta, GgufModelBuilder, RequestBuilder, Response,
    TextMessageRole,
};
use serde::Serialize;

#[derive(Parser, Debug)]
#[command(about = "Benchmark mistral.rs GGUF inference")]
struct Args {
    /// Directory that contains the GGUF file.
    #[arg(long)]
    model_dir: String,

    /// Filename of the GGUF model inside `model-dir`.
    #[arg(long)]
    filename: String,

    /// Prompt to feed the model.
    #[arg(long)]
    prompt: String,

    /// Number of tokens to generate (max).
    #[arg(long, default_value_t = 200)]
    max_tokens: usize,
}

#[derive(Debug, Serialize)]
struct Summary {
    runtime: &'static str,
    load_seconds: f64,
    first_token_seconds: f64,
    generated_tokens: usize,
    generation_seconds: f64,
    tokens_per_second: f64,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    eprintln!("building model from {}/{}", args.model_dir, args.filename);
    let load_start = Instant::now();
    let model = GgufModelBuilder::new(args.model_dir.clone(), vec![args.filename.clone()])
        .with_logging()
        .build()
        .await?;
    let load_seconds = load_start.elapsed().as_secs_f64();
    eprintln!("loaded in {load_seconds:.2}s");

    let request = RequestBuilder::new()
        .add_message(TextMessageRole::User, &args.prompt)
        .set_sampler_max_len(args.max_tokens);

    let mut stream = model.stream_chat_request(request).await?;

    let gen_start = Instant::now();
    let mut first_token_seconds: Option<f64> = None;
    let mut generated_tokens = 0usize;

    let stdout = std::io::stdout();
    let mut out = std::io::BufWriter::new(stdout.lock());

    while let Some(response) = stream.next().await {
        if let Response::Chunk(ChatCompletionChunkResponse { choices, .. }) = response {
            if let Some(ChunkChoice {
                delta:
                    Delta {
                        content: Some(content),
                        ..
                    },
                ..
            }) = choices.first()
            {
                if first_token_seconds.is_none() {
                    first_token_seconds = Some(gen_start.elapsed().as_secs_f64());
                }
                generated_tokens += 1;
                let _ = out.write_all(content.as_bytes());
            }
        }
    }
    let generation_seconds = gen_start.elapsed().as_secs_f64();
    drop(out);
    eprintln!();

    let summary = Summary {
        runtime: "mistralrs",
        load_seconds: round2(load_seconds),
        first_token_seconds: round2(first_token_seconds.unwrap_or(generation_seconds)),
        generated_tokens,
        generation_seconds: round2(generation_seconds),
        tokens_per_second: round2(generated_tokens as f64 / generation_seconds.max(0.0001)),
    };

    let json = serde_json::to_string_pretty(&summary)?;
    println!("{json}");
    Ok(())
}

fn round2(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}
