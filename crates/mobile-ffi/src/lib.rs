use reflective_mobile_ai::{recommended_home, AiTask, ExecutionHome};
use reflective_mobile_core::quorum::{
    append_consented_signal, draft_field_signal, SignalModality, FIELD_SIGNAL_CAPTURE_WORKFLOW_ID,
};
use reflective_mobile_core::{starter_portfolio, MobilePlatform, ProductFamily, ProductStatus};

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

pub fn ai_execution_home(platform: &str, task: &str) -> String {
    let Some(platform) = parse_platform(platform) else {
        return "unsupported-platform".to_owned();
    };
    let Some(task) = parse_task(task) else {
        return "unsupported-task".to_owned();
    };

    match recommended_home(platform, task) {
        ExecutionHome::RustCore => "rust-core".to_owned(),
        ExecutionHome::Platform(runtime) => format!("{runtime:?}"),
    }
}

pub fn quorum_field_signal_workflow_id() -> String {
    FIELD_SIGNAL_CAPTURE_WORKFLOW_ID.to_owned()
}

pub fn quorum_draft_field_signal(
    inquiry_thread_id: &str,
    modality: &str,
    raw_capture: &str,
) -> FfiQuorumSignalDraft {
    let modality = SignalModality::parse(modality).unwrap_or(SignalModality::Text);
    let draft = draft_field_signal(inquiry_thread_id, modality, raw_capture);

    FfiQuorumSignalDraft {
        workflow_id: draft.workflow_id,
        draft_id: draft.draft_id,
        inquiry_thread_id: draft.inquiry_thread_id,
        modality: draft.modality,
        raw_capture: draft.raw_capture,
        summary: draft.summary,
        latent_need: draft.latent_need,
        contradiction: draft.contradiction,
        confidence: draft.confidence,
        consent_state: draft.consent_state,
    }
}

pub fn quorum_append_consented_signal(draft: FfiQuorumSignalDraft) -> FfiQuorumAppendEvent {
    let draft = reflective_mobile_core::quorum::QuorumSignalDraft {
        workflow_id: draft.workflow_id,
        draft_id: draft.draft_id,
        inquiry_thread_id: draft.inquiry_thread_id,
        modality: draft.modality,
        raw_capture: draft.raw_capture,
        summary: draft.summary,
        latent_need: draft.latent_need,
        contradiction: draft.contradiction,
        confidence: draft.confidence,
        consent_state: draft.consent_state,
    };

    let event = append_consented_signal(&draft);

    FfiQuorumAppendEvent {
        workflow_id: event.workflow_id,
        event_type: event.event_type,
        draft_id: event.draft_id,
        inquiry_thread_id: event.inquiry_thread_id,
        sync_state: event.sync_state,
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
            ai_execution_home("ios", "structured-extraction"),
            "IosFoundationModels"
        );
        assert_eq!(ai_execution_home("android", "vector-search"), "rust-core");
    }

    #[test]
    fn ffi_facade_exposes_quorum_workflow_contract() {
        let draft = quorum_draft_field_signal(
            "inq_mobile_launch_risks",
            "voice_transcript",
            "The sales team says rollout is fine, but support is seeing confusion in every pilot.",
        );

        assert_eq!(draft.workflow_id, "quorum.field_signal_capture.v1");
        assert_eq!(draft.consent_state, "pending");

        let event = quorum_append_consented_signal(draft);
        assert_eq!(event.event_type, "SignalDraftConsented");
        assert_eq!(event.sync_state, "queued_for_sync");
    }
}
