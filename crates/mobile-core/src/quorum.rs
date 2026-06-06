pub const FIELD_SIGNAL_CAPTURE_WORKFLOW_ID: &str = "quorum.field_signal_capture.v1";
pub const FIELD_SIGNAL_CAPTURE_FIXTURE_JSON: &str =
    include_str!("../../../apps/marquee/quorum-sense/fixtures/field-signal-capture.v1.json");

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SignalModality {
    Text,
    VoiceTranscript,
    ImageOcrText,
}

impl SignalModality {
    pub fn as_str(self) -> &'static str {
        match self {
            SignalModality::Text => "text",
            SignalModality::VoiceTranscript => "voice_transcript",
            SignalModality::ImageOcrText => "image_ocr_text",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "text" => Some(Self::Text),
            "voice_transcript" => Some(Self::VoiceTranscript),
            "image_ocr_text" => Some(Self::ImageOcrText),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct QuorumSignalDraft {
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
pub struct QuorumAppendEvent {
    pub workflow_id: String,
    pub event_type: String,
    pub draft_id: String,
    pub inquiry_thread_id: String,
    pub sync_state: String,
}

pub fn draft_field_signal(
    inquiry_thread_id: &str,
    modality: SignalModality,
    raw_capture: &str,
) -> QuorumSignalDraft {
    let summary = summarize_for_fixture(raw_capture);

    QuorumSignalDraft {
        workflow_id: FIELD_SIGNAL_CAPTURE_WORKFLOW_ID.to_owned(),
        draft_id: format!("draft:{inquiry_thread_id}:field-signal-v1"),
        inquiry_thread_id: inquiry_thread_id.to_owned(),
        modality: modality.as_str().to_owned(),
        raw_capture: raw_capture.to_owned(),
        summary,
        latent_need: "needs earlier visibility into organizational ambiguity".to_owned(),
        contradiction: "participants report alignment while surfacing unresolved tension"
            .to_owned(),
        confidence: 0.67,
        consent_state: "pending".to_owned(),
    }
}

pub fn append_consented_signal(draft: &QuorumSignalDraft) -> QuorumAppendEvent {
    QuorumAppendEvent {
        workflow_id: draft.workflow_id.clone(),
        event_type: "SignalDraftConsented".to_owned(),
        draft_id: draft.draft_id.clone(),
        inquiry_thread_id: draft.inquiry_thread_id.clone(),
        sync_state: "queued_for_sync".to_owned(),
    }
}

fn summarize_for_fixture(raw_capture: &str) -> String {
    let trimmed = raw_capture.trim();
    if trimmed.is_empty() {
        return "Empty capture needs participant clarification".to_owned();
    }

    trimmed.chars().take(96).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixture_declares_first_quorum_workflow() {
        assert!(FIELD_SIGNAL_CAPTURE_FIXTURE_JSON.contains(FIELD_SIGNAL_CAPTURE_WORKFLOW_ID));
        assert!(FIELD_SIGNAL_CAPTURE_FIXTURE_JSON.contains("\"consent_state\": \"pending\""));
        assert!(FIELD_SIGNAL_CAPTURE_FIXTURE_JSON.contains("\"sync_state\": \"queued_for_sync\""));
    }

    #[test]
    fn draft_requires_consent_before_append() {
        let draft = draft_field_signal(
            "inq_mobile_launch_risks",
            SignalModality::VoiceTranscript,
            "The sales team says rollout is fine, but support is seeing confusion in every pilot.",
        );

        assert_eq!(draft.workflow_id, FIELD_SIGNAL_CAPTURE_WORKFLOW_ID);
        assert_eq!(draft.modality, "voice_transcript");
        assert_eq!(draft.consent_state, "pending");

        let event = append_consented_signal(&draft);
        assert_eq!(event.event_type, "SignalDraftConsented");
        assert_eq!(event.sync_state, "queued_for_sync");
    }
}
