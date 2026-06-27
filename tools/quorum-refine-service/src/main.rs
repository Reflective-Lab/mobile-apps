//! Local Quorum refine-service — the cloud-fallback tier for the on-device
//! refinement loop (M6 placement). The emulator's native `LlmBackend` POSTs a
//! prompt to `/complete`; this process completes it with Anthropic (Haiku
//! primary) and falls back to OpenAI, then returns the text. API keys are read
//! from the env (direnv populates them from the macOS Keychain), so keys never
//! reach the device. Production is the same shape behind GC Secrets.
//!
//! Self-contained on purpose: direct HTTP to the providers, no Manifold tree.

use std::sync::Arc;
use std::time::Duration;

use axum::{Json, Router, extract::State, routing::post};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Deserialize)]
struct CompleteRequest {
    prompt: String,
}

#[derive(Serialize)]
struct CompleteResponse {
    text: String,
}

struct AppState {
    http: reqwest::Client,
    anthropic_key: Option<String>,
    openai_key: Option<String>,
}

const SYSTEM: &str = "You are a careful editor for a live team-inquiry app. Follow \
the instruction exactly and reply with ONLY the requested text — no preamble, no \
quotes, no explanation.";
const ANTHROPIC_MODEL: &str = "claude-haiku-4-5-20251001";
const OPENAI_MODEL: &str = "gpt-4o-mini";
const MAX_TOKENS: u32 = 256;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let anthropic_key = env_key("ANTHROPIC_API_KEY");
    let openai_key = env_key("OPENAI_API_KEY");
    if anthropic_key.is_none() && openai_key.is_none() {
        anyhow::bail!(
            "no ANTHROPIC_API_KEY or OPENAI_API_KEY in env — run in a shell where \
             direnv has loaded them from the Keychain"
        );
    }
    eprintln!(
        "providers: {}{}",
        if anthropic_key.is_some() {
            "anthropic(claude-haiku-4-5) "
        } else {
            ""
        },
        if openai_key.is_some() {
            "openai(gpt-4o-mini)"
        } else {
            ""
        },
    );

    let state = Arc::new(AppState {
        http: reqwest::Client::builder()
            .timeout(Duration::from_secs(20))
            .build()?,
        anthropic_key,
        openai_key,
    });

    let app = Router::new()
        .route("/complete", post(complete))
        .with_state(state);

    let addr = "0.0.0.0:8765";
    let listener = tokio::net::TcpListener::bind(addr).await?;
    eprintln!("quorum-refine-service listening on http://{addr}  (POST /complete {{\"prompt\":\"...\"}})");
    eprintln!("  iOS simulator -> http://127.0.0.1:8765   Android emulator -> http://10.0.2.2:8765");
    axum::serve(listener, app).await?;
    Ok(())
}

fn env_key(name: &str) -> Option<String> {
    std::env::var(name)
        .ok()
        .map(|k| k.trim().to_owned())
        .filter(|k| !k.is_empty())
}

async fn complete(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CompleteRequest>,
) -> Json<CompleteResponse> {
    // Anthropic primary, OpenAI fallback. Empty text on total failure — the Rust
    // refiner then falls back to its deterministic heuristics, so a draft is
    // always produced.
    let mut text = String::new();

    if let Some(key) = &state.anthropic_key {
        match anthropic_complete(&state.http, key, &req.prompt).await {
            Ok(t) if !t.trim().is_empty() => text = t,
            Ok(_) => {}
            Err(e) => eprintln!("anthropic error: {e}"),
        }
    }
    if text.trim().is_empty() {
        if let Some(key) = &state.openai_key {
            match openai_complete(&state.http, key, &req.prompt).await {
                Ok(t) => text = t,
                Err(e) => eprintln!("openai error: {e}"),
            }
        }
    }

    Json(CompleteResponse { text })
}

async fn anthropic_complete(
    http: &reqwest::Client,
    key: &str,
    prompt: &str,
) -> anyhow::Result<String> {
    let body = http
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", key)
        .header("anthropic-version", "2023-06-01")
        .json(&json!({
            "model": ANTHROPIC_MODEL,
            "max_tokens": MAX_TOKENS,
            "system": SYSTEM,
            "messages": [{ "role": "user", "content": prompt }],
        }))
        .send()
        .await?
        .error_for_status()?
        .json::<serde_json::Value>()
        .await?;
    // content: [{ "type": "text", "text": "..." }]
    let text = body["content"]
        .as_array()
        .and_then(|blocks| blocks.iter().find_map(|b| b["text"].as_str()))
        .unwrap_or_default()
        .to_owned();
    Ok(text)
}

async fn openai_complete(
    http: &reqwest::Client,
    key: &str,
    prompt: &str,
) -> anyhow::Result<String> {
    let body = http
        .post("https://api.openai.com/v1/chat/completions")
        .bearer_auth(key)
        .json(&json!({
            "model": OPENAI_MODEL,
            "max_tokens": MAX_TOKENS,
            "messages": [
                { "role": "system", "content": SYSTEM },
                { "role": "user", "content": prompt },
            ],
        }))
        .send()
        .await?
        .error_for_status()?
        .json::<serde_json::Value>()
        .await?;
    let text = body["choices"][0]["message"]["content"]
        .as_str()
        .unwrap_or_default()
        .to_owned();
    Ok(text)
}
