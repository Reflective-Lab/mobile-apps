// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT

use reflective_mobile_core::director::{
    BlockingState as DomainBlockingState, Choice as DomainChoice,
    ContextLevel as DomainContextLevel, DirectorApiConfig, DirectorFrame as DomainDirectorFrame,
    DirectorIntent as DomainDirectorIntent, DirectorPrompt as DomainDirectorPrompt,
    DirectorSnapshot as DomainDirectorSnapshot, DirectorSnapshotSource, GateId,
    GateVerdict as DomainGateVerdict, NowTask as DomainNowTask, PresenceHint as DomainPresenceHint,
    PrimaryAction as DomainPrimaryAction, ReviewStance as DomainReviewStance,
    SecondaryAction as DomainSecondaryAction, WaitingFor as DomainWaitingFor,
    gate_condition_wire_label, quorum_director_fixture_snapshot, resolve_director_snapshot,
    start_director_sse_listener, stop_director_sse_listener, submit_director_intent,
    apply_local_director_intent, wait_director_snapshot_version,
};
use std::sync::Mutex;
use std::time::Duration;

// ---------------------------------------------------------------------------
// UniFFI wire types (mirrors schemas/quorum-mobile.udl).
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GateVerdict {
    Approve,
    Reject,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReviewStance {
    Agree,
    Disagree,
    NeedMoreContext,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContextLevel {
    Task,
    LocalContext,
    Session,
    Formation,
    Organization,
    Everything,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlockingState {
    NotBlocking,
    BlocksFormation,
    BlocksSession,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WaitingForKind {
    Nobody,
    Participants,
    Server,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DirectorPromptKind {
    Judgment,
    Gate,
    Review,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DirectorIntentKind {
    OpenTask,
    SubmitJudgment,
    RespondGate,
    SubmitReview,
    RequestContext,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FfiWaitingFor {
    pub kind: WaitingForKind,
    pub actor_labels: Option<Vec<String>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FfiNowTask {
    pub objective: String,
    pub needed_from_user: Option<String>,
    pub estimated_minutes: Option<u32>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FfiChoice {
    pub choice_id: String,
    pub label: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FfiJudgmentPrompt {
    pub question: String,
    pub body: String,
    pub choices: Vec<FfiChoice>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FfiGatePrompt {
    pub gate_id: String,
    pub reason: String,
    pub consequence: String,
    pub deadline_ms: Option<u64>,
    pub condition_kind: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FfiReviewPrompt {
    pub title: String,
    pub primary_evidence: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FfiDirectorIntent {
    pub kind: DirectorIntentKind,
    pub frame_id: Option<String>,
    pub choice_id: Option<String>,
    pub gate_id: Option<String>,
    pub gate_verdict: Option<GateVerdict>,
    pub review_stance: Option<ReviewStance>,
    pub context_level: Option<ContextLevel>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FfiDirectorPrompt {
    pub kind: DirectorPromptKind,
    pub judgment: Option<FfiJudgmentPrompt>,
    pub gate: Option<FfiGatePrompt>,
    pub review: Option<FfiReviewPrompt>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FfiPrimaryAction {
    pub label: String,
    pub intent: FfiDirectorIntent,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FfiSecondaryAction {
    pub label: String,
    pub intent: FfiDirectorIntent,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FfiPresenceHint {
    pub actor_label: String,
    pub status: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FfiDirectorFrame {
    pub frame_id: String,
    pub title: String,
    pub subtitle: Option<String>,
    pub now: Option<FfiNowTask>,
    pub waiting_for: FfiWaitingFor,
    pub primary: FfiPrimaryAction,
    pub secondary: Vec<FfiSecondaryAction>,
    pub prompt: Option<FfiDirectorPrompt>,
    pub presence: Vec<FfiPresenceHint>,
    pub context_trail: Vec<ContextLevel>,
    pub blocking: BlockingState,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FfiDirectorSnapshot {
    pub version: u64,
    pub frame: FfiDirectorFrame,
}

static LAST_DIRECTOR_INTENT: Mutex<Option<DomainDirectorIntent>> = Mutex::new(None);
static DIRECTOR_API: Mutex<Option<DirectorApiConfig>> = Mutex::new(None);
static LAST_SNAPSHOT_SOURCE: Mutex<String> = Mutex::new(String::new());

/// Point the director boundary at a Quorum HTTP base (e.g. local `just dev`).
/// Idempotent; safe to call from app startup before the first snapshot read.
pub fn quorum_configure_director_api(base_url: String, bearer_token: String) {
    let config = DirectorApiConfig::new(base_url, bearer_token);
    start_director_sse_listener(config.clone());
    if let Ok(mut slot) = DIRECTOR_API.lock() {
        *slot = Some(config);
    }
}

/// Wire label for how the last snapshot was resolved (`live`, `fixture`, or
/// `fixture_fallback:<reason>` when live fetch failed).
#[must_use]
pub fn quorum_director_snapshot_source() -> String {
    LAST_SNAPSHOT_SOURCE
        .lock()
        .map(|label| label.clone())
        .unwrap_or_else(|_| "unknown".to_owned())
}

/// Returns the current director snapshot: live HTTP when configured, otherwise
/// the committed fixture. Live failures fall back to the fixture until Plan 2
/// exposes `/api/director/snapshot` on Quorum.
#[must_use]
pub fn quorum_current_director_snapshot() -> FfiDirectorSnapshot {
    let config = DIRECTOR_API.lock().ok().and_then(|guard| guard.clone());

    let (snapshot, source) = resolve_director_snapshot(config.as_ref()).unwrap_or_else(|_| {
        let fixture = quorum_director_fixture_snapshot().expect("director fixture must parse");
        (fixture, DirectorSnapshotSource::FixtureOnly)
    });

    if let Ok(mut label) = LAST_SNAPSHOT_SOURCE.lock() {
        *label = source.wire_label();
    }

    to_ffi_snapshot(&snapshot)
}

/// Block until the director snapshot version exceeds `since_version`, or
/// `timeout_ms` elapses. Returns `true` when Client Helm processed a newer SSE event.
#[must_use]
pub fn quorum_wait_director_update(since_version: u64, timeout_ms: u32) -> bool {
    let timeout = Duration::from_millis(u64::from(timeout_ms));
    wait_director_snapshot_version(since_version, timeout)
}

/// Accepts a typed director intent from Swift/Kotlin.
pub fn quorum_submit_director_intent(intent: FfiDirectorIntent) {
    let Ok(domain) = from_ffi_intent(intent) else {
        return;
    };
    if let Ok(mut slot) = LAST_DIRECTOR_INTENT.lock() {
        *slot = Some(domain.clone());
    }
    apply_local_director_intent(&domain);
    if let Ok(guard) = DIRECTOR_API.lock() {
        if let Some(config) = guard.as_ref() {
            let _ = submit_director_intent(config, &domain);
        }
    }
}

#[cfg(test)]
pub fn take_last_director_intent() -> Option<DomainDirectorIntent> {
    LAST_DIRECTOR_INTENT.lock().ok()?.take()
}

fn to_ffi_snapshot(snapshot: &DomainDirectorSnapshot) -> FfiDirectorSnapshot {
    FfiDirectorSnapshot {
        version: snapshot.version,
        frame: to_ffi_frame(&snapshot.frame),
    }
}

fn to_ffi_frame(frame: &DomainDirectorFrame) -> FfiDirectorFrame {
    FfiDirectorFrame {
        frame_id: frame.frame_id.clone(),
        title: frame.title.clone(),
        subtitle: frame.subtitle.clone(),
        now: frame.now.as_ref().map(to_ffi_now),
        waiting_for: to_ffi_waiting(&frame.waiting_for),
        primary: to_ffi_primary(&frame.primary),
        secondary: frame.secondary.iter().map(to_ffi_secondary).collect(),
        prompt: frame.prompt.as_ref().map(to_ffi_prompt),
        presence: frame.presence.iter().map(to_ffi_presence).collect(),
        context_trail: frame
            .context_trail
            .iter()
            .copied()
            .map(Into::into)
            .collect(),
        blocking: frame.blocking.into(),
    }
}

fn to_ffi_now(now: &DomainNowTask) -> FfiNowTask {
    FfiNowTask {
        objective: now.objective.clone(),
        needed_from_user: now.needed_from_user.clone(),
        estimated_minutes: now.estimated_minutes,
    }
}

fn to_ffi_waiting(waiting: &DomainWaitingFor) -> FfiWaitingFor {
    match waiting {
        DomainWaitingFor::Nobody => FfiWaitingFor {
            kind: WaitingForKind::Nobody,
            actor_labels: None,
        },
        DomainWaitingFor::Participants { actor_labels } => FfiWaitingFor {
            kind: WaitingForKind::Participants,
            actor_labels: Some(actor_labels.clone()),
        },
        DomainWaitingFor::Server => FfiWaitingFor {
            kind: WaitingForKind::Server,
            actor_labels: None,
        },
    }
}

fn to_ffi_primary(action: &DomainPrimaryAction) -> FfiPrimaryAction {
    FfiPrimaryAction {
        label: action.label.clone(),
        intent: to_ffi_intent(&action.intent),
    }
}

fn to_ffi_secondary(action: &DomainSecondaryAction) -> FfiSecondaryAction {
    FfiSecondaryAction {
        label: action.label.clone(),
        intent: to_ffi_intent(&action.intent),
    }
}

fn to_ffi_prompt(prompt: &DomainDirectorPrompt) -> FfiDirectorPrompt {
    match prompt {
        DomainDirectorPrompt::Judgment(j) => FfiDirectorPrompt {
            kind: DirectorPromptKind::Judgment,
            judgment: Some(FfiJudgmentPrompt {
                question: j.question.clone(),
                body: j.body.clone(),
                choices: j.choices.iter().map(to_ffi_choice).collect(),
            }),
            gate: None,
            review: None,
        },
        DomainDirectorPrompt::Gate(g) => FfiDirectorPrompt {
            kind: DirectorPromptKind::Gate,
            judgment: None,
            gate: Some(FfiGatePrompt {
                gate_id: g.gate_id.as_str().to_owned(),
                reason: g.reason.clone(),
                consequence: g.consequence.clone(),
                deadline_ms: g.deadline_ms,
                condition_kind: gate_condition_wire_label(&g.condition).to_owned(),
            }),
            review: None,
        },
        DomainDirectorPrompt::Review(r) => FfiDirectorPrompt {
            kind: DirectorPromptKind::Review,
            judgment: None,
            gate: None,
            review: Some(FfiReviewPrompt {
                title: r.title.clone(),
                primary_evidence: r.primary_evidence.clone(),
            }),
        },
    }
}

fn to_ffi_choice(choice: &DomainChoice) -> FfiChoice {
    FfiChoice {
        choice_id: choice.choice_id.clone(),
        label: choice.label.clone(),
    }
}

fn to_ffi_presence(hint: &DomainPresenceHint) -> FfiPresenceHint {
    FfiPresenceHint {
        actor_label: hint.actor_label.clone(),
        status: hint.status.clone(),
    }
}

fn to_ffi_intent(intent: &DomainDirectorIntent) -> FfiDirectorIntent {
    match intent {
        DomainDirectorIntent::OpenTask { frame_id } => FfiDirectorIntent {
            kind: DirectorIntentKind::OpenTask,
            frame_id: Some(frame_id.clone()),
            choice_id: None,
            gate_id: None,
            gate_verdict: None,
            review_stance: None,
            context_level: None,
        },
        DomainDirectorIntent::SubmitJudgment {
            frame_id,
            choice_id,
        } => FfiDirectorIntent {
            kind: DirectorIntentKind::SubmitJudgment,
            frame_id: Some(frame_id.clone()),
            choice_id: Some(choice_id.clone()),
            gate_id: None,
            gate_verdict: None,
            review_stance: None,
            context_level: None,
        },
        DomainDirectorIntent::RespondGate { gate_id, verdict } => FfiDirectorIntent {
            kind: DirectorIntentKind::RespondGate,
            frame_id: None,
            choice_id: None,
            gate_id: Some(gate_id.as_str().to_owned()),
            gate_verdict: Some((*verdict).into()),
            review_stance: None,
            context_level: None,
        },
        DomainDirectorIntent::SubmitReview { frame_id, stance } => FfiDirectorIntent {
            kind: DirectorIntentKind::SubmitReview,
            frame_id: Some(frame_id.clone()),
            choice_id: None,
            gate_id: None,
            gate_verdict: None,
            review_stance: Some((*stance).into()),
            context_level: None,
        },
        DomainDirectorIntent::RequestContext { level } => FfiDirectorIntent {
            kind: DirectorIntentKind::RequestContext,
            frame_id: None,
            choice_id: None,
            gate_id: None,
            gate_verdict: None,
            review_stance: None,
            context_level: Some((*level).into()),
        },
    }
}

fn from_ffi_intent(intent: FfiDirectorIntent) -> Result<DomainDirectorIntent, ()> {
    match intent.kind {
        DirectorIntentKind::OpenTask => {
            let frame_id = intent.frame_id.ok_or(())?;
            Ok(DomainDirectorIntent::OpenTask { frame_id })
        }
        DirectorIntentKind::SubmitJudgment => {
            let frame_id = intent.frame_id.ok_or(())?;
            let choice_id = intent.choice_id.ok_or(())?;
            Ok(DomainDirectorIntent::SubmitJudgment {
                frame_id,
                choice_id,
            })
        }
        DirectorIntentKind::RespondGate => {
            let gate_id = intent.gate_id.ok_or(())?;
            let verdict = intent.gate_verdict.ok_or(())?;
            Ok(DomainDirectorIntent::RespondGate {
                gate_id: GateId::from_string(gate_id),
                verdict: verdict.into(),
            })
        }
        DirectorIntentKind::SubmitReview => {
            let frame_id = intent.frame_id.ok_or(())?;
            let stance = intent.review_stance.ok_or(())?;
            Ok(DomainDirectorIntent::SubmitReview {
                frame_id,
                stance: stance.into(),
            })
        }
        DirectorIntentKind::RequestContext => {
            let level = intent.context_level.ok_or(())?;
            Ok(DomainDirectorIntent::RequestContext {
                level: level.into(),
            })
        }
    }
}

impl From<DomainGateVerdict> for GateVerdict {
    fn from(value: DomainGateVerdict) -> Self {
        match value {
            DomainGateVerdict::Approve => Self::Approve,
            DomainGateVerdict::Reject => Self::Reject,
        }
    }
}

impl From<GateVerdict> for DomainGateVerdict {
    fn from(value: GateVerdict) -> Self {
        match value {
            GateVerdict::Approve => Self::Approve,
            GateVerdict::Reject => Self::Reject,
        }
    }
}

impl From<DomainReviewStance> for ReviewStance {
    fn from(value: DomainReviewStance) -> Self {
        match value {
            DomainReviewStance::Agree => Self::Agree,
            DomainReviewStance::Disagree => Self::Disagree,
            DomainReviewStance::NeedMoreContext => Self::NeedMoreContext,
        }
    }
}

impl From<ReviewStance> for DomainReviewStance {
    fn from(value: ReviewStance) -> Self {
        match value {
            ReviewStance::Agree => Self::Agree,
            ReviewStance::Disagree => Self::Disagree,
            ReviewStance::NeedMoreContext => Self::NeedMoreContext,
        }
    }
}

impl From<DomainContextLevel> for ContextLevel {
    fn from(value: DomainContextLevel) -> Self {
        match value {
            DomainContextLevel::Task => Self::Task,
            DomainContextLevel::LocalContext => Self::LocalContext,
            DomainContextLevel::Session => Self::Session,
            DomainContextLevel::Formation => Self::Formation,
            DomainContextLevel::Organization => Self::Organization,
            DomainContextLevel::Everything => Self::Everything,
        }
    }
}

impl From<ContextLevel> for DomainContextLevel {
    fn from(value: ContextLevel) -> Self {
        match value {
            ContextLevel::Task => Self::Task,
            ContextLevel::LocalContext => Self::LocalContext,
            ContextLevel::Session => Self::Session,
            ContextLevel::Formation => Self::Formation,
            ContextLevel::Organization => Self::Organization,
            ContextLevel::Everything => Self::Everything,
        }
    }
}

impl From<DomainBlockingState> for BlockingState {
    fn from(value: DomainBlockingState) -> Self {
        match value {
            DomainBlockingState::NotBlocking => Self::NotBlocking,
            DomainBlockingState::BlocksFormation => Self::BlocksFormation,
            DomainBlockingState::BlocksSession => Self::BlocksSession,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(test)]
    fn reset_director_api_for_tests() {
        stop_director_sse_listener();
        if let Ok(mut slot) = DIRECTOR_API.lock() {
            *slot = None;
        }
        if let Ok(mut label) = LAST_SNAPSHOT_SOURCE.lock() {
            label.clear();
        }
    }

    #[test]
    fn current_director_snapshot_matches_fixture_version() {
        reset_director_api_for_tests();
        let snapshot = quorum_current_director_snapshot();
        assert_eq!(snapshot.version, 1844);
        assert_eq!(
            snapshot
                .frame
                .now
                .as_ref()
                .map(|now| now.objective.as_str()),
            Some("Evaluate Vendor X's security claims")
        );
        assert_eq!(quorum_director_snapshot_source(), "fixture");
    }

    #[test]
    #[ignore = "requires local quorum-server on :5161 (just run-local)"]
    fn debug_defaults_resolve_live_snapshot() {
        reset_director_api_for_tests();
        quorum_configure_director_api("http://127.0.0.1:5161/quorum-sense".into(), "dev".into());
        let snapshot = quorum_current_director_snapshot();
        assert_eq!(snapshot.version, 1844);
        assert_eq!(quorum_director_snapshot_source(), "live");
        reset_director_api_for_tests();
    }

    #[test]
    fn configure_director_api_then_snapshot_without_panic() {
        reset_director_api_for_tests();
        quorum_configure_director_api("http://127.0.0.1:5161/quorum-sense".into(), "dev".into());
        let snapshot = quorum_current_director_snapshot();
        let source = quorum_director_snapshot_source();
        assert!(
            source.starts_with("fixture_fallback:")
                || source == "live"
                || source == "live_sse",
            "unexpected source: {source}"
        );
        if source == "live" || source == "live_sse" {
            assert!(snapshot.version >= 1);
        } else {
            assert_eq!(snapshot.version, 1844);
        }
        reset_director_api_for_tests();
    }

    #[test]
    fn submit_director_intent_round_trips_respond_gate() {
        quorum_submit_director_intent(FfiDirectorIntent {
            kind: DirectorIntentKind::RespondGate,
            frame_id: None,
            choice_id: None,
            gate_id: Some("gate:procurement-security-approval".into()),
            gate_verdict: Some(GateVerdict::Approve),
            review_stance: None,
            context_level: None,
        });

        let stored = take_last_director_intent().expect("intent stored");
        assert!(matches!(stored, DomainDirectorIntent::RespondGate { .. }));
    }
}
