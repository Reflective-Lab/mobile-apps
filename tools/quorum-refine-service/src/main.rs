//! Local Quorum refine-service — the cloud-fallback tier for the on-device
//! refinement loop (M6 placement). The emulator's native `LlmBackend` POSTs a
//! prompt to `/complete`; this process completes it with Manifold's
//! `ResilientChatBackend` (Anthropic Haiku primary, OpenAI fallback) and returns
//! the text. API keys are read by Manifold's secret provider from the env
//! (direnv populates them from the macOS Keychain), so keys never reach the
//! device. Production is the same shape behind GC Secrets.

use std::sync::Arc;

use axum::{Json, Router, extract::State, routing::post};
use converge_provider::{ChatBackend, ChatMessage, ChatRequest, ChatRole, ResponseFormat};
use manifold::{AnthropicBackend, OpenAiBackend, ResilientChatBackend};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct CompleteRequest {
    prompt: String,
}

#[derive(Serialize)]
struct CompleteResponse {
    text: String,
}

struct AppState {
    backend: ResilientChatBackend,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Anthropic Haiku primary (fast + cheap, right size for spell/grammar
    // polish); OpenAI fallback if its key is also present. Keys via Manifold's
    // default secret provider (env, populated from Keychain by direnv).
    let anthropic = AnthropicBackend::from_env()
        .map_err(|e| anyhow::anyhow!("anthropic init (need ANTHROPIC_API_KEY): {e}"))?
        .with_model("claude-haiku-4-5-20251001");
    let mut backend = ResilientChatBackend::new(Arc::new(anthropic), "anthropic-haiku");
    match OpenAiBackend::from_env() {
        Ok(openai) => {
            backend = backend
                .with_fallback(Arc::new(openai.with_model("gpt-4o-mini")), "openai-4o-mini");
            eprintln!("fallback: openai gpt-4o-mini");
        }
        Err(e) => eprintln!("no openai fallback ({e}); anthropic only"),
    }

    let state = Arc::new(AppState { backend });
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

async fn complete(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CompleteRequest>,
) -> Json<CompleteResponse> {
    let chat = ChatRequest {
        messages: vec![ChatMessage {
            role: ChatRole::User,
            content: req.prompt,
            tool_calls: Vec::new(),
            tool_call_id: None,
        }],
        system: Some(
            "You are a careful editor for a live team-inquiry app. Follow the \
             instruction exactly and reply with ONLY the requested text — no \
             preamble, no quotes, no explanation."
                .to_owned(),
        ),
        tools: Vec::new(),
        response_format: ResponseFormat::Text,
        max_tokens: Some(256),
        temperature: Some(0.2),
        stop_sequences: Vec::new(),
        model: None,
    };

    // On any model error, return empty text — the Rust refiner then falls back
    // to its deterministic heuristics, so a draft is always produced.
    let text = match state.backend.chat(chat).await {
        Ok(response) => response.content,
        Err(e) => {
            eprintln!("llm error: {e}");
            String::new()
        }
    };
    Json(CompleteResponse { text })
}
