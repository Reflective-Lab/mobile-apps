//! On-device local signal-refinement loop — the device half of the hybrid loop.
//!
//! Runs a Converge fixed-point *formation* over the raw participant capture to
//! produce a structured draft (summary / latent need / contradiction /
//! confidence). It sharpens the participant's OWN signal; it never promotes
//! facts or computes collective/consensus state — that stays server-side
//! (ADR 0002, the "device proposes; server promotes" boundary).
//!
//! The formation is three suggestors over one shared [`Context`], run to a
//! fixed point by [`converge_core::Engine`]:
//!
//! - seed: the raw capture (via [`SeedSuggestor`]) → `Seeds`
//! - summary: `Seeds` → `Hypotheses`
//! - latent need: `Hypotheses` → `Signals` (depends on summary, so the loop
//!   genuinely takes more than one cycle to converge)
//! - contradiction: `Seeds` → `Disagreements`
//!
//! How each suggestor turns text into text is delegated to a [`RefineBackend`].
//! v1 ships a deterministic [`HeuristicBackend`]; the device-embedded-LLM
//! backend (iOS Foundation Models / Android Gemini Nano) and Manifold cloud
//! backends slot in behind this same seam via converge-provider's `ChatBackend`
//! (M6 compute placement) without touching the suggestors.
//!
//! Infallible at this boundary: any engine error degrades to a low-confidence
//! heuristic draft rather than panicking across the FFI.

use std::sync::Arc;

use async_trait::async_trait;
use converge_core::suggestors::SeedSuggestor;
use converge_core::{
    AgentEffect, ContextKey, ContextState, ConvergeError, ConvergeResult, Engine, ProposedFact,
    Suggestor, TextPayload,
};

const SEED_ID: &str = "raw-capture";
const SUMMARY_ID: &str = "summary";
const LATENT_NEED_ID: &str = "latent-need";
const CONTRADICTION_ID: &str = "contradiction";

const TENSION_MARKERS: [&str; 5] = ["but", "however", "although", "yet", "while"];

/// The refined fields the formation converges to.
#[derive(Clone, Debug, PartialEq)]
pub struct RefinedSignal {
    pub summary: String,
    pub latent_need: String,
    pub contradiction: String,
    pub confidence: f32,
}

/// The text-generation seam the refinement suggestors call. Kept sync + pure so
/// the v1 loop is deterministic and panic-free across the FFI. The device LLM
/// and cloud backends arrive behind converge-provider's `ChatBackend` later.
pub trait RefineBackend: Send + Sync {
    fn summarize(&self, raw: &str) -> String;
    fn latent_need(&self, raw: &str, summary: &str) -> String;
    fn contradiction(&self, raw: &str) -> String;
}

/// Deterministic, offline, dependency-free refinement. Also the fallback tier
/// for phones without an embedded model.
#[derive(Clone, Copy, Debug, Default)]
pub struct HeuristicBackend;

impl RefineBackend for HeuristicBackend {
    fn summarize(&self, raw: &str) -> String {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return "Empty capture needs participant clarification".to_owned();
        }
        let first = trimmed
            .split(['.', '!', '?'])
            .next()
            .unwrap_or(trimmed)
            .trim();
        let base = if first.is_empty() { trimmed } else { first };
        // Bounded at 96 chars — a pipeline invariant the draft contract relies on
        // (see quorum_tests `draft_invariants_hold_for_any_input`).
        base.chars().take(96).collect()
    }

    fn latent_need(&self, raw: &str, _summary: &str) -> String {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return default_latent_need();
        }
        format!(
            "surface the unspoken need behind: \"{}\"",
            first_words(trimmed, 8)
        )
    }

    fn contradiction(&self, raw: &str) -> String {
        let lower = raw.to_lowercase();
        match TENSION_MARKERS
            .iter()
            .find(|m| lower.contains(&format!(" {m} ")))
        {
            Some(marker) => format!(
                "tension marked by \"{marker}\": {}",
                first_words(raw.trim(), 16)
            ),
            None => no_contradiction(),
        }
    }
}

/// Refine a raw capture with the default deterministic backend.
#[must_use]
pub fn refine_capture(raw_capture: &str) -> RefinedSignal {
    refine_capture_with(raw_capture, Arc::new(HeuristicBackend))
}

/// Refine a raw capture, driving the Converge formation with `backend`.
#[must_use]
pub fn refine_capture_with(raw_capture: &str, backend: Arc<dyn RefineBackend>) -> RefinedSignal {
    match run_formation(raw_capture, backend) {
        Ok(result) => extract(raw_capture, &result),
        Err(_) => degraded(raw_capture),
    }
}

/// Build and run the static formation to a fixed point. Separated so tests can
/// assert convergence (`converged` / `cycles`) directly.
fn run_formation(
    raw_capture: &str,
    backend: Arc<dyn RefineBackend>,
) -> Result<ConvergeResult, ConvergeError> {
    let mut engine = Engine::new();
    engine.register_suggestor(SeedSuggestor::new(SEED_ID, raw_capture));
    engine.register_suggestor(SummarySuggestor {
        backend: backend.clone(),
    });
    engine.register_suggestor(LatentNeedSuggestor {
        backend: backend.clone(),
    });
    engine.register_suggestor(ContradictionSuggestor { backend });
    pollster::block_on(engine.run(ContextState::new()))
}

fn extract(raw_capture: &str, result: &ConvergeResult) -> RefinedSignal {
    let summary = read_field(&result.context, ContextKey::Hypotheses, SUMMARY_ID);
    let latent = read_field(&result.context, ContextKey::Signals, LATENT_NEED_ID);
    let contradiction = read_field(&result.context, ContextKey::Disagreements, CONTRADICTION_ID);
    let present = [summary.is_some(), latent.is_some(), contradiction.is_some()];

    RefinedSignal {
        confidence: confidence_for(raw_capture, result.converged, present),
        summary: summary.unwrap_or_else(|| HeuristicBackend.summarize(raw_capture)),
        latent_need: latent.unwrap_or_else(default_latent_need),
        contradiction: contradiction.unwrap_or_else(no_contradiction),
    }
}

fn read_field(ctx: &ContextState, key: ContextKey, id: &str) -> Option<String> {
    ctx.get(key)
        .iter()
        .find(|fact| fact.id().as_str() == id)
        .and_then(|fact| fact.text())
        .map(str::to_owned)
}

fn confidence_for(raw_capture: &str, converged: bool, present: [bool; 3]) -> f32 {
    let mut score = 0.35_f32;
    if converged {
        score += 0.2;
    }
    score += present.iter().filter(|p| **p).count() as f32 * 0.1;
    let words = raw_capture.split_whitespace().count().min(20) as f32;
    score += (words / 20.0) * 0.15;
    score.clamp(0.0, 1.0)
}

fn degraded(raw_capture: &str) -> RefinedSignal {
    RefinedSignal {
        summary: HeuristicBackend.summarize(raw_capture),
        latent_need: default_latent_need(),
        contradiction: no_contradiction(),
        confidence: 0.2,
    }
}

fn default_latent_need() -> String {
    "needs earlier visibility into organizational ambiguity".to_owned()
}

fn no_contradiction() -> String {
    "no explicit contradiction surfaced".to_owned()
}

fn first_words(text: &str, n: usize) -> String {
    text.split_whitespace()
        .take(n)
        .collect::<Vec<_>>()
        .join(" ")
}

/// Reads the seed capture out of a live `Context` during a cycle. Falls back to
/// the first seed fact if the id differs, so a backend swap can't strand it.
fn seed_capture(ctx: &dyn converge_core::Context) -> String {
    let seeds = ctx.get(ContextKey::Seeds);
    seeds
        .iter()
        .find(|fact| fact.id().as_str() == SEED_ID)
        .or_else(|| seeds.iter().next())
        .and_then(|fact| fact.text())
        .unwrap_or_default()
        .to_owned()
}

fn live_field(ctx: &dyn converge_core::Context, key: ContextKey, id: &str) -> Option<String> {
    ctx.get(key)
        .iter()
        .find(|fact| fact.id().as_str() == id)
        .and_then(|fact| fact.text())
        .map(str::to_owned)
}

struct SummarySuggestor {
    backend: Arc<dyn RefineBackend>,
}

#[async_trait]
impl Suggestor for SummarySuggestor {
    fn name(&self) -> &str {
        "quorum.refine.summary"
    }

    fn dependencies(&self) -> &[ContextKey] {
        &[ContextKey::Seeds]
    }

    fn accepts(&self, ctx: &dyn converge_core::Context) -> bool {
        !seed_capture(ctx).is_empty()
            && live_field(ctx, ContextKey::Hypotheses, SUMMARY_ID).is_none()
    }

    async fn execute(&self, ctx: &dyn converge_core::Context) -> AgentEffect {
        let summary = self.backend.summarize(&seed_capture(ctx));
        AgentEffect::with_proposal(ProposedFact::new(
            ContextKey::Hypotheses,
            SUMMARY_ID,
            TextPayload::new(summary),
            self.provenance(),
        ))
    }
}

struct LatentNeedSuggestor {
    backend: Arc<dyn RefineBackend>,
}

#[async_trait]
impl Suggestor for LatentNeedSuggestor {
    fn name(&self) -> &str {
        "quorum.refine.latent-need"
    }

    fn dependencies(&self) -> &[ContextKey] {
        &[ContextKey::Hypotheses]
    }

    fn accepts(&self, ctx: &dyn converge_core::Context) -> bool {
        live_field(ctx, ContextKey::Hypotheses, SUMMARY_ID).is_some()
            && live_field(ctx, ContextKey::Signals, LATENT_NEED_ID).is_none()
    }

    async fn execute(&self, ctx: &dyn converge_core::Context) -> AgentEffect {
        let summary = live_field(ctx, ContextKey::Hypotheses, SUMMARY_ID).unwrap_or_default();
        let latent = self.backend.latent_need(&seed_capture(ctx), &summary);
        AgentEffect::with_proposal(ProposedFact::new(
            ContextKey::Signals,
            LATENT_NEED_ID,
            TextPayload::new(latent),
            self.provenance(),
        ))
    }
}

struct ContradictionSuggestor {
    backend: Arc<dyn RefineBackend>,
}

#[async_trait]
impl Suggestor for ContradictionSuggestor {
    fn name(&self) -> &str {
        "quorum.refine.contradiction"
    }

    fn dependencies(&self) -> &[ContextKey] {
        &[ContextKey::Seeds]
    }

    fn accepts(&self, ctx: &dyn converge_core::Context) -> bool {
        !seed_capture(ctx).is_empty()
            && live_field(ctx, ContextKey::Disagreements, CONTRADICTION_ID).is_none()
    }

    async fn execute(&self, ctx: &dyn converge_core::Context) -> AgentEffect {
        let contradiction = self.backend.contradiction(&seed_capture(ctx));
        AgentEffect::with_proposal(ProposedFact::new(
            ContextKey::Disagreements,
            CONTRADICTION_ID,
            TextPayload::new(contradiction),
            self.provenance(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formation_converges_in_multiple_cycles() {
        let result = run_formation(
            "Sales says the rollout is fine, but support sees confusion in every pilot.",
            Arc::new(HeuristicBackend),
        )
        .expect("formation should converge");
        assert!(result.converged, "loop should reach a fixed point");
        assert!(
            result.cycles >= 2,
            "latent-need depends on summary, so it takes >1 cycle (got {})",
            result.cycles
        );
    }

    #[test]
    fn refines_a_tension_capture() {
        let refined = refine_capture(
            "Sales says the rollout is fine, but support sees confusion in every pilot.",
        );
        assert!(!refined.summary.is_empty());
        assert!(
            refined.contradiction.contains("tension") || refined.contradiction.contains("but"),
            "expected the tension marker to be surfaced, got: {}",
            refined.contradiction
        );
        assert!(refined.confidence > 0.0 && refined.confidence <= 1.0);
    }

    #[test]
    fn empty_capture_degrades_gracefully() {
        let refined = refine_capture("   ");
        assert!(!refined.summary.is_empty());
        assert!((0.0..=1.0).contains(&refined.confidence));
    }

    #[test]
    fn refinement_is_deterministic() {
        let input = "Leadership wants speed, although the team flags real quality risk.";
        assert_eq!(refine_capture(input), refine_capture(input));
    }
}
