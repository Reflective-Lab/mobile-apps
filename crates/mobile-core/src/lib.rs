pub mod quorum;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProductFamily {
    Marquee,
    Studio,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProductStatus {
    Lead,
    Candidate,
    Archived,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MobilePlatform {
    Ios,
    Android,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlatformRuntime {
    IosFoundationModels,
    IosCoreMl,
    IosVision,
    IosSpeech,
    IosAvFoundation,
    AndroidGeminiNano,
    AndroidLiteRt,
    AndroidMlKit,
    AndroidMediaPipe,
    AndroidCameraX,
    AndroidMedia3,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SharedCapability {
    BusinessLogic,
    Sync,
    StorageModel,
    VectorSearch,
    Embeddings,
    AiOrchestration,
    DeterministicReplay,
    DataModels,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MobileAppSpec {
    pub slug: &'static str,
    pub display_name: &'static str,
    pub family: ProductFamily,
    pub status: ProductStatus,
    pub source_repo: &'static str,
    pub platforms: &'static [MobilePlatform],
    pub platform_runtimes: &'static [PlatformRuntime],
    pub shared_capabilities: &'static [SharedCapability],
}

impl MobileAppSpec {
    pub fn supports_platform(&self, platform: MobilePlatform) -> bool {
        self.platforms.contains(&platform)
    }

    pub fn needs_runtime(&self, runtime: PlatformRuntime) -> bool {
        self.platform_runtimes.contains(&runtime)
    }

    pub fn uses_shared_capability(&self, capability: SharedCapability) -> bool {
        self.shared_capabilities.contains(&capability)
    }

    pub fn requires_native_shells(&self) -> bool {
        self.platform_runtimes.iter().any(|runtime| {
            matches!(
                runtime,
                PlatformRuntime::IosFoundationModels
                    | PlatformRuntime::IosCoreMl
                    | PlatformRuntime::IosVision
                    | PlatformRuntime::IosSpeech
                    | PlatformRuntime::IosAvFoundation
                    | PlatformRuntime::AndroidGeminiNano
                    | PlatformRuntime::AndroidLiteRt
                    | PlatformRuntime::AndroidMlKit
                    | PlatformRuntime::AndroidMediaPipe
                    | PlatformRuntime::AndroidCameraX
                    | PlatformRuntime::AndroidMedia3
            )
        })
    }
}

const BOTH_PLATFORMS: &[MobilePlatform] = &[MobilePlatform::Ios, MobilePlatform::Android];

const QUORUM_RUNTIMES: &[PlatformRuntime] = &[
    PlatformRuntime::IosFoundationModels,
    PlatformRuntime::IosVision,
    PlatformRuntime::IosSpeech,
    PlatformRuntime::IosAvFoundation,
    PlatformRuntime::AndroidGeminiNano,
    PlatformRuntime::AndroidMlKit,
    PlatformRuntime::AndroidMediaPipe,
    PlatformRuntime::AndroidCameraX,
];

const WOLFGANG_RUNTIMES: &[PlatformRuntime] = &[
    PlatformRuntime::IosFoundationModels,
    PlatformRuntime::IosSpeech,
    PlatformRuntime::AndroidGeminiNano,
    PlatformRuntime::AndroidMlKit,
];

const INKLING_RUNTIMES: &[PlatformRuntime] = &[
    PlatformRuntime::IosFoundationModels,
    PlatformRuntime::IosVision,
    PlatformRuntime::IosSpeech,
    PlatformRuntime::IosAvFoundation,
    PlatformRuntime::AndroidGeminiNano,
    PlatformRuntime::AndroidMlKit,
    PlatformRuntime::AndroidCameraX,
];

const SHARED_AI_CORE: &[SharedCapability] = &[
    SharedCapability::BusinessLogic,
    SharedCapability::Sync,
    SharedCapability::StorageModel,
    SharedCapability::VectorSearch,
    SharedCapability::Embeddings,
    SharedCapability::AiOrchestration,
    SharedCapability::DeterministicReplay,
    SharedCapability::DataModels,
];

pub const STARTER_PORTFOLIO: &[MobileAppSpec] = &[
    MobileAppSpec {
        slug: "quorum-sense",
        display_name: "Quorum Mobile",
        family: ProductFamily::Marquee,
        status: ProductStatus::Lead,
        source_repo: "../marquee-apps/quorum-sense",
        platforms: BOTH_PLATFORMS,
        platform_runtimes: QUORUM_RUNTIMES,
        shared_capabilities: SHARED_AI_CORE,
    },
    MobileAppSpec {
        slug: "wolfgang-chat",
        display_name: "Wolfgang Mobile",
        family: ProductFamily::Studio,
        status: ProductStatus::Candidate,
        source_repo: "../studio-apps/wolfgang-chat",
        platforms: BOTH_PLATFORMS,
        platform_runtimes: WOLFGANG_RUNTIMES,
        shared_capabilities: SHARED_AI_CORE,
    },
    MobileAppSpec {
        slug: "inkling-notes",
        display_name: "Inkling Mobile",
        family: ProductFamily::Studio,
        status: ProductStatus::Candidate,
        source_repo: "../studio-apps/inkling-notes",
        platforms: BOTH_PLATFORMS,
        platform_runtimes: INKLING_RUNTIMES,
        shared_capabilities: SHARED_AI_CORE,
    },
];

pub fn starter_portfolio() -> &'static [MobileAppSpec] {
    STARTER_PORTFOLIO
}

pub fn lead_mobile_app() -> Option<&'static MobileAppSpec> {
    STARTER_PORTFOLIO
        .iter()
        .find(|app| app.status == ProductStatus::Lead)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quorum_is_the_lead_mobile_app() {
        let lead = lead_mobile_app().expect("expected a lead mobile app");

        assert_eq!(lead.slug, "quorum-sense");
        assert_eq!(lead.family, ProductFamily::Marquee);
        assert!(lead.supports_platform(MobilePlatform::Ios));
        assert!(lead.supports_platform(MobilePlatform::Android));
    }

    #[test]
    fn lead_app_requires_native_ai_shells_and_shared_rust() {
        let lead = lead_mobile_app().expect("expected a lead mobile app");

        assert!(lead.requires_native_shells());
        assert!(lead.needs_runtime(PlatformRuntime::IosFoundationModels));
        assert!(lead.needs_runtime(PlatformRuntime::AndroidGeminiNano));
        assert!(lead.uses_shared_capability(SharedCapability::AiOrchestration));
        assert!(lead.uses_shared_capability(SharedCapability::DeterministicReplay));
    }
}
