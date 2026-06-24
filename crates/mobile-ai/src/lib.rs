// Boundary hardening (QF-2026-06-24-02) — see reflective-mobile-core. AI-routing
// logic runs under the UniFFI seam; forbid hand-written `unsafe` and deny
// implicit-abort paths so a routing decision can never panic into the native shell.
#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use reflective_mobile_core::{MobilePlatform, PlatformRuntime};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AiTask {
    GenerativeText,
    StructuredExtraction,
    Embeddings,
    VectorSearch,
    VisionUnderstanding,
    SpeechTranscription,
    CameraRealtimeInference,
    AudioPreprocessing,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExecutionHome {
    RustCore,
    Platform(PlatformRuntime),
}

pub fn recommended_home(platform: MobilePlatform, task: AiTask) -> ExecutionHome {
    match (platform, task) {
        (MobilePlatform::Ios, AiTask::GenerativeText)
        | (MobilePlatform::Ios, AiTask::StructuredExtraction) => {
            ExecutionHome::Platform(PlatformRuntime::IosFoundationModels)
        }
        (MobilePlatform::Ios, AiTask::VisionUnderstanding)
        | (MobilePlatform::Ios, AiTask::CameraRealtimeInference) => {
            ExecutionHome::Platform(PlatformRuntime::IosVision)
        }
        (MobilePlatform::Ios, AiTask::SpeechTranscription) => {
            ExecutionHome::Platform(PlatformRuntime::IosSpeech)
        }
        (MobilePlatform::Android, AiTask::GenerativeText)
        | (MobilePlatform::Android, AiTask::StructuredExtraction) => {
            ExecutionHome::Platform(PlatformRuntime::AndroidGeminiNano)
        }
        (MobilePlatform::Android, AiTask::VisionUnderstanding) => {
            ExecutionHome::Platform(PlatformRuntime::AndroidMlKit)
        }
        (MobilePlatform::Android, AiTask::CameraRealtimeInference) => {
            ExecutionHome::Platform(PlatformRuntime::AndroidMediaPipe)
        }
        (MobilePlatform::Android, AiTask::SpeechTranscription) => {
            ExecutionHome::Platform(PlatformRuntime::AndroidMlKit)
        }
        (_, AiTask::Embeddings) | (_, AiTask::VectorSearch) | (_, AiTask::AudioPreprocessing) => {
            ExecutionHome::RustCore
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ios_genai_routes_to_foundation_models() {
        assert_eq!(
            recommended_home(MobilePlatform::Ios, AiTask::StructuredExtraction),
            ExecutionHome::Platform(PlatformRuntime::IosFoundationModels)
        );
    }

    #[test]
    fn android_realtime_camera_routes_to_mediapipe() {
        assert_eq!(
            recommended_home(MobilePlatform::Android, AiTask::CameraRealtimeInference),
            ExecutionHome::Platform(PlatformRuntime::AndroidMediaPipe)
        );
    }

    #[test]
    fn shared_memory_tasks_route_to_rust() {
        assert_eq!(
            recommended_home(MobilePlatform::Ios, AiTask::VectorSearch),
            ExecutionHome::RustCore
        );
        assert_eq!(
            recommended_home(MobilePlatform::Android, AiTask::Embeddings),
            ExecutionHome::RustCore
        );
    }
}
