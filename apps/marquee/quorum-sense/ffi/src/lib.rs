#![allow(clippy::empty_line_after_doc_comments)]

// UniFFI scaffolding for src/quorum_mobile.udl (compiled copy of the
// canonical contract in schemas/quorum-mobile.udl).
uniffi::include_scaffolding!("quorum_mobile");

use reflective_mobile_ai::{AiTask, ExecutionHome, recommended_home};
use reflective_mobile_core::quorum::{
    Confidence, ConsentState, DraftId, FIELD_SIGNAL_CAPTURE_WORKFLOW_ID, InquiryThreadId,
    QuorumAppendEvent, QuorumSignalDraft, SignalModality, WorkflowId, append_consented_signal,
    draft_field_signal,
};
use reflective_mobile_core::{
    MobilePlatform, PlatformRuntime, ProductFamily, ProductStatus, starter_portfolio,
};

// ---------------------------------------------------------------------------
// Observability. Rust-side crash + error reporting via Sentry, sent directly to
// the separate Rust project (ureq + rustls transport — no OpenSSL). The host app
// initialises it at startup; Rust panics are captured automatically and
// unexpected boundary errors are reported explicitly. Routine validation is not.
// ---------------------------------------------------------------------------
#[cfg(feature = "observability")]
static SENTRY_GUARD: std::sync::OnceLock<sentry::ClientInitGuard> = std::sync::OnceLock::new();

/// Wire up Sentry. Idempotent: only the first call initialises the client, so it
/// is safe to invoke from `Application.onCreate` / `App.init` on every launch.
pub fn init_observability(dsn: String, environment: String, release: String, debug: bool) {
    #[cfg(feature = "observability")]
    {
        if SENTRY_GUARD.get().is_some() {
            return;
        }
        let guard = sentry::init((
            dsn,
            sentry::ClientOptions {
                release: Some(release.into()),
                environment: Some(environment.into()),
                debug,
                // The panic integration (default with the `panic` feature) reports
                // Rust panics — with backtraces — that UniFFI would otherwise hide.
                ..Default::default()
            },
        ));
        let _ = SENTRY_GUARD.set(guard);
    }
    #[cfg(not(feature = "observability"))]
    let _ = (dsn, environment, release, debug);
}

/// Report an error only when it is an unexpected/internal failure; routine
/// boundary-validation errors are expected and never reported.
fn report_if_unexpected(error: &QuorumError) {
    #[cfg(feature = "observability")]
    if error.is_reportable() {
        sentry::capture_error(error);
    }
    #[cfg(not(feature = "observability"))]
    let _ = error;
}

/// Tap a fallible boundary result for reporting, then pass it through unchanged.
fn observed<T>(result: Result<T, QuorumError>) -> Result<T, QuorumError> {
    if let Err(error) = &result {
        report_if_unexpected(error);
    }
    result
}

// ---------------------------------------------------------------------------
// Boundary error. Raw foreign input (strings, floats) is mapped to typed domain
// values here; anything that doesn't map becomes one of these errors rather than
// a silent default. The offending value rides along for diagnostics.
// ---------------------------------------------------------------------------
#[derive(Debug, thiserror::Error)]
pub enum QuorumError {
    #[error("unknown signal modality: {value}")]
    InvalidModality { value: String },
    #[error("unknown consent state: {value}")]
    InvalidConsentState { value: String },
    #[error("confidence {value} is outside 0.0..=1.0")]
    ConfidenceOutOfRange { value: f32 },
    #[error("unsupported platform: {value}")]
    UnsupportedPlatform { value: String },
    #[error("unsupported task: {value}")]
    UnsupportedTask { value: String },
}

impl QuorumError {
    /// Whether this is an unexpected/internal failure worth a Sentry event.
    /// Every current variant is a routine boundary-validation error (the caller
    /// sent something invalid), so this is `false` today; future *internal*
    /// failure variants (storage, sync, …) should return `true`.
    fn is_reportable(&self) -> bool {
        match self {
            QuorumError::InvalidModality { .. }
            | QuorumError::InvalidConsentState { .. }
            | QuorumError::ConfidenceOutOfRange { .. }
            | QuorumError::UnsupportedPlatform { .. }
            | QuorumError::UnsupportedTask { .. } => false,
        }
    }
}

// ---------------------------------------------------------------------------
// Wire DTOs. These are intentionally string/float-typed: that's the boundary
// representation crossing into Swift/Kotlin. Semantics live in the domain types
// in mobile-core; the conversions below are the only place strings become meaning.
// ---------------------------------------------------------------------------
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FfiAppSummary {
    pub slug: String,
    pub display_name: String,
    pub family: String,
    pub status: String,
    pub source_repo: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FfiQuorumSignalDraft {
    pub workflow_id: String,
    pub draft_id: String,
    pub inquiry_thread_id: String,
    pub modality: String,
    pub raw_capture: String,
    pub summary: String,
    pub latent_need: String,
    pub contradiction: String,
    pub confidence: f32,
    pub consent_state: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FfiQuorumAppendEvent {
    pub workflow_id: String,
    pub event_type: String,
    pub draft_id: String,
    pub inquiry_thread_id: String,
    pub sync_state: String,
}

pub fn mobile_portfolio() -> Vec<FfiAppSummary> {
    starter_portfolio()
        .iter()
        .map(|app| FfiAppSummary {
            slug: app.slug.to_owned(),
            display_name: app.display_name.to_owned(),
            family: family_label(app.family).to_owned(),
            status: status_label(app.status).to_owned(),
            source_repo: app.source_repo.to_owned(),
        })
        .collect()
}

pub fn ai_execution_home(platform: String, task: String) -> Result<String, QuorumError> {
    observed((|| {
        let platform_value =
            parse_platform(&platform).ok_or_else(|| QuorumError::UnsupportedPlatform {
                value: platform.clone(),
            })?;
        let task_value = parse_task(&task).ok_or_else(|| QuorumError::UnsupportedTask {
            value: task.clone(),
        })?;
        Ok(execution_home_label(recommended_home(platform_value, task_value)).to_owned())
    })())
}

pub fn quorum_field_signal_workflow_id() -> String {
    FIELD_SIGNAL_CAPTURE_WORKFLOW_ID.to_owned()
}

pub fn quorum_draft_field_signal(
    inquiry_thread_id: String,
    modality: String,
    raw_capture: String,
) -> Result<FfiQuorumSignalDraft, QuorumError> {
    observed((|| {
        let modality_value =
            SignalModality::parse(&modality).ok_or_else(|| QuorumError::InvalidModality {
                value: modality.clone(),
            })?;
        let draft = draft_field_signal(&inquiry_thread_id, modality_value, &raw_capture);
        Ok(to_ffi_draft(&draft))
    })())
}

pub fn quorum_append_consented_signal(
    draft: FfiQuorumSignalDraft,
) -> Result<FfiQuorumAppendEvent, QuorumError> {
    observed((|| {
        let domain = from_ffi_draft(draft)?;
        Ok(to_ffi_event(&append_consented_signal(&domain)))
    })())
}

// ---------------------------------------------------------------------------
// Boundary mappings — the single home for String/float <-> domain conversion.
// ---------------------------------------------------------------------------
fn to_ffi_draft(draft: &QuorumSignalDraft) -> FfiQuorumSignalDraft {
    FfiQuorumSignalDraft {
        workflow_id: draft.workflow_id.as_str().to_owned(),
        draft_id: draft.draft_id.as_str().to_owned(),
        inquiry_thread_id: draft.inquiry_thread_id.as_str().to_owned(),
        modality: draft.modality.as_str().to_owned(),
        raw_capture: draft.raw_capture.clone(),
        summary: draft.summary.clone(),
        latent_need: draft.latent_need.clone(),
        contradiction: draft.contradiction.clone(),
        confidence: draft.confidence.value(),
        consent_state: draft.consent_state.as_str().to_owned(),
    }
}

fn from_ffi_draft(draft: FfiQuorumSignalDraft) -> Result<QuorumSignalDraft, QuorumError> {
    let modality =
        SignalModality::parse(&draft.modality).ok_or_else(|| QuorumError::InvalidModality {
            value: draft.modality.clone(),
        })?;
    let consent_state = ConsentState::parse(&draft.consent_state).ok_or_else(|| {
        QuorumError::InvalidConsentState {
            value: draft.consent_state.clone(),
        }
    })?;
    let confidence =
        Confidence::new(draft.confidence).ok_or(QuorumError::ConfidenceOutOfRange {
            value: draft.confidence,
        })?;

    Ok(QuorumSignalDraft {
        workflow_id: WorkflowId::new(draft.workflow_id),
        draft_id: DraftId::new(draft.draft_id),
        inquiry_thread_id: InquiryThreadId::new(draft.inquiry_thread_id),
        modality,
        raw_capture: draft.raw_capture,
        summary: draft.summary,
        latent_need: draft.latent_need,
        contradiction: draft.contradiction,
        confidence,
        consent_state,
    })
}

fn to_ffi_event(event: &QuorumAppendEvent) -> FfiQuorumAppendEvent {
    FfiQuorumAppendEvent {
        workflow_id: event.workflow_id.as_str().to_owned(),
        event_type: event.event_type.as_str().to_owned(),
        draft_id: event.draft_id.as_str().to_owned(),
        inquiry_thread_id: event.inquiry_thread_id.as_str().to_owned(),
        sync_state: event.sync_state.as_str().to_owned(),
    }
}

fn parse_platform(platform: &str) -> Option<MobilePlatform> {
    match platform {
        "ios" => Some(MobilePlatform::Ios),
        "android" => Some(MobilePlatform::Android),
        _ => None,
    }
}

fn parse_task(task: &str) -> Option<AiTask> {
    match task {
        "generative-text" => Some(AiTask::GenerativeText),
        "structured-extraction" => Some(AiTask::StructuredExtraction),
        "embeddings" => Some(AiTask::Embeddings),
        "vector-search" => Some(AiTask::VectorSearch),
        "vision-understanding" => Some(AiTask::VisionUnderstanding),
        "speech-transcription" => Some(AiTask::SpeechTranscription),
        "camera-realtime-inference" => Some(AiTask::CameraRealtimeInference),
        "audio-preprocessing" => Some(AiTask::AudioPreprocessing),
        _ => None,
    }
}

/// Stable wire label for an execution home (explicit mapping — never `{:?}`,
/// which would silently change if a variant were renamed).
fn execution_home_label(home: ExecutionHome) -> &'static str {
    match home {
        ExecutionHome::RustCore => "rust-core",
        ExecutionHome::Platform(runtime) => platform_runtime_label(runtime),
    }
}

fn platform_runtime_label(runtime: PlatformRuntime) -> &'static str {
    match runtime {
        PlatformRuntime::IosFoundationModels => "ios-foundation-models",
        PlatformRuntime::IosCoreMl => "ios-core-ml",
        PlatformRuntime::IosVision => "ios-vision",
        PlatformRuntime::IosSpeech => "ios-speech",
        PlatformRuntime::IosAvFoundation => "ios-av-foundation",
        PlatformRuntime::AndroidGeminiNano => "android-gemini-nano",
        PlatformRuntime::AndroidLiteRt => "android-lite-rt",
        PlatformRuntime::AndroidMlKit => "android-ml-kit",
        PlatformRuntime::AndroidMediaPipe => "android-media-pipe",
        PlatformRuntime::AndroidCameraX => "android-camera-x",
        PlatformRuntime::AndroidMedia3 => "android-media3",
    }
}

fn family_label(family: ProductFamily) -> &'static str {
    match family {
        ProductFamily::Marquee => "marquee",
        ProductFamily::Studio => "studio",
    }
}

fn status_label(status: ProductStatus) -> &'static str {
    match status {
        ProductStatus::Lead => "lead",
        ProductStatus::Candidate => "candidate",
        ProductStatus::Archived => "archived",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Values mirror apps/marquee/quorum-sense/fixtures/field-signal-capture.v1.json.
    const FIXTURE_INQUIRY_THREAD_ID: &str = "inq_mobile_launch_risks";
    const FIXTURE_MODALITY: &str = "voice_transcript";
    const FIXTURE_RAW_CAPTURE: &str =
        "The sales team says rollout is fine, but support is seeing confusion in every pilot.";

    fn fixture_draft() -> FfiQuorumSignalDraft {
        quorum_draft_field_signal(
            FIXTURE_INQUIRY_THREAD_ID.to_owned(),
            FIXTURE_MODALITY.to_owned(),
            FIXTURE_RAW_CAPTURE.to_owned(),
        )
        .expect("fixture modality is valid")
    }

    #[test]
    fn portfolio_exposes_quorum_first() {
        let portfolio = mobile_portfolio();

        assert_eq!(portfolio[0].slug, "quorum-sense");
        assert_eq!(portfolio[0].family, "marquee");
        assert_eq!(portfolio[0].status, "lead");
    }

    #[test]
    fn ffi_facade_exposes_ai_routing_policy() {
        assert_eq!(
            ai_execution_home("ios".to_owned(), "structured-extraction".to_owned()).unwrap(),
            "ios-foundation-models"
        );
        assert_eq!(
            ai_execution_home("android".to_owned(), "vector-search".to_owned()).unwrap(),
            "rust-core"
        );
        assert!(matches!(
            ai_execution_home("web".to_owned(), "embeddings".to_owned()),
            Err(QuorumError::UnsupportedPlatform { .. })
        ));
        assert!(matches!(
            ai_execution_home("ios".to_owned(), "telepathy".to_owned()),
            Err(QuorumError::UnsupportedTask { .. })
        ));
    }

    #[test]
    fn workflow_id_matches_fixture_contract() {
        assert_eq!(
            quorum_field_signal_workflow_id(),
            "quorum.field_signal_capture.v1"
        );
    }

    #[test]
    fn draft_matches_fixture_values() {
        let draft = fixture_draft();

        assert_eq!(draft.workflow_id, "quorum.field_signal_capture.v1");
        assert_eq!(
            draft.draft_id,
            "draft:inq_mobile_launch_risks:field-signal-v1"
        );
        assert_eq!(draft.inquiry_thread_id, FIXTURE_INQUIRY_THREAD_ID);
        assert_eq!(draft.modality, "voice_transcript");
        assert_eq!(draft.raw_capture, FIXTURE_RAW_CAPTURE);
        assert_eq!(
            draft.latent_need,
            "needs earlier visibility into organizational ambiguity"
        );
        assert_eq!(
            draft.contradiction,
            "participants report alignment while surfacing unresolved tension"
        );
        assert_eq!(draft.confidence, 0.67);
        assert_eq!(draft.consent_state, "pending");
    }

    #[test]
    fn append_after_consent_queues_fixture_event() {
        let draft = fixture_draft();
        let event = quorum_append_consented_signal(draft.clone()).unwrap();

        assert_eq!(event.workflow_id, draft.workflow_id);
        assert_eq!(event.event_type, "SignalDraftConsented");
        assert_eq!(event.draft_id, draft.draft_id);
        assert_eq!(event.inquiry_thread_id, FIXTURE_INQUIRY_THREAD_ID);
        assert_eq!(event.sync_state, "queued_for_sync");
    }

    #[test]
    fn unknown_modality_is_rejected_at_the_boundary() {
        let result = quorum_draft_field_signal(
            FIXTURE_INQUIRY_THREAD_ID.to_owned(),
            "hologram".to_owned(),
            FIXTURE_RAW_CAPTURE.to_owned(),
        );

        assert!(matches!(
            result,
            Err(QuorumError::InvalidModality { value }) if value == "hologram"
        ));
    }

    #[test]
    fn append_rejects_out_of_range_confidence() {
        let mut draft = fixture_draft();
        draft.confidence = 2.0;

        assert!(matches!(
            quorum_append_consented_signal(draft),
            Err(QuorumError::ConfidenceOutOfRange { value }) if value == 2.0
        ));
    }

    #[test]
    fn append_rejects_unknown_consent_state() {
        let mut draft = fixture_draft();
        draft.consent_state = "revoked".to_owned();

        assert!(matches!(
            quorum_append_consented_signal(draft),
            Err(QuorumError::InvalidConsentState { value }) if value == "revoked"
        ));
    }
}
