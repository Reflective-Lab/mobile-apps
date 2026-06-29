//! HTTP fetch for canonical `DirectorSnapshot` from Quorum (client-only).
//!
//! Session push (SSE) stays stubbed until Plan 2; this module only performs
//! snapshot GET against `{QUORUM_BASE_URL}/api/director/snapshot`.

use super::{DirectorSnapshot, quorum_director_fixture_snapshot};
use std::time::Duration;

/// Default local Quorum base URL (`just dev` from quorum-sense).
pub const DEFAULT_LOCAL_QUORUM_BASE_URL: &str = "http://127.0.0.1:5161/quorum-sense";

/// Bearer token accepted when `LOCAL_DEV=true` on the Quorum server.
pub const DEFAULT_LOCAL_DEV_BEARER: &str = "dev";

/// Canonical director snapshot route (server-side Plan 2).
pub const DIRECTOR_SNAPSHOT_PATH: &str = "/api/director/snapshot";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DirectorApiConfig {
    pub base_url: String,
    pub bearer_token: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DirectorSnapshotSource {
    Live,
    FixtureOnly,
    FixtureFallback { reason: String },
}

#[derive(Debug, thiserror::Error)]
pub enum LiveDirectorError {
    #[error("HTTP {status}: {body}")]
    HttpError { status: u16, body: String },
    #[error("network error: {0}")]
    Transport(#[from] ureq::Error),
    #[error("failed to read response body: {0}")]
    BodyRead(#[from] std::io::Error),
    #[error("invalid snapshot JSON: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum ResolveDirectorError {
    #[error("director fixture unavailable: {0}")]
    FixtureUnavailable(#[from] serde_json::Error),
}

impl DirectorSnapshotSource {
    #[must_use]
    pub fn wire_label(&self) -> String {
        match self {
            Self::Live => "live".to_owned(),
            Self::FixtureOnly => "fixture".to_owned(),
            Self::FixtureFallback { reason } => format!("fixture_fallback:{reason}"),
        }
    }
}

/// Build the snapshot GET URL from a configured base (trailing slash tolerated).
#[must_use]
pub fn director_snapshot_url(config: &DirectorApiConfig) -> String {
    let base = config.base_url.trim_end_matches('/');
    format!("{base}{DIRECTOR_SNAPSHOT_PATH}")
}

/// Fetch a live `DirectorSnapshot` from Quorum. Requires network access.
pub fn fetch_live_director_snapshot(
    config: &DirectorApiConfig,
) -> Result<DirectorSnapshot, LiveDirectorError> {
    let url = director_snapshot_url(config);
    let agent = ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_secs(5))
        .timeout_read(Duration::from_secs(10))
        .build();

    let response = agent
        .get(&url)
        .set("Authorization", &format!("Bearer {}", config.bearer_token))
        .set("Accept", "application/json")
        .call()?;

    let status = response.status();
    let body = response
        .into_string()
        .map_err(LiveDirectorError::BodyRead)?;
    if status != 200 {
        return Err(LiveDirectorError::HttpError { status, body });
    }
    serde_json::from_str(&body).map_err(Into::into)
}

/// Resolve the snapshot the mobile UI should render.
///
/// When API config is present, tries live fetch first and falls back to the
/// committed fixture on any transport/HTTP/parse failure. Without config,
/// returns the fixture only (preview/tests).
pub fn resolve_director_snapshot(
    config: Option<&DirectorApiConfig>,
) -> Result<(DirectorSnapshot, DirectorSnapshotSource), ResolveDirectorError> {
    match config {
        Some(cfg) => match fetch_live_director_snapshot(cfg) {
            Ok(snapshot) => Ok((snapshot, DirectorSnapshotSource::Live)),
            Err(err) => {
                let reason = err.to_string();
                let fixture = quorum_director_fixture_snapshot()?;
                Ok((fixture, DirectorSnapshotSource::FixtureFallback { reason }))
            }
        },
        None => {
            let fixture = quorum_director_fixture_snapshot()?;
            Ok((fixture, DirectorSnapshotSource::FixtureOnly))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_url_joins_base_and_path() {
        let config = DirectorApiConfig {
            base_url: "http://127.0.0.1:5161/quorum-sense/".into(),
            bearer_token: "dev".into(),
        };
        assert_eq!(
            director_snapshot_url(&config),
            "http://127.0.0.1:5161/quorum-sense/api/director/snapshot"
        );
    }

    #[test]
    fn resolve_without_config_uses_fixture() {
        let (snapshot, source) = resolve_director_snapshot(None).expect("fixture must parse");
        assert_eq!(snapshot.version, 1844);
        assert_eq!(source, DirectorSnapshotSource::FixtureOnly);
    }

    #[test]
    fn resolve_with_unreachable_host_falls_back_to_fixture() {
        let config = DirectorApiConfig {
            base_url: "http://127.0.0.1:1/unreachable".into(),
            bearer_token: "dev".into(),
        };
        let (snapshot, source) =
            resolve_director_snapshot(Some(&config)).expect("fixture fallback");
        assert_eq!(snapshot.version, 1844);
        assert!(matches!(
            source,
            DirectorSnapshotSource::FixtureFallback { .. }
        ));
    }

    #[test]
    fn wire_label_covers_all_sources() {
        assert_eq!(DirectorSnapshotSource::Live.wire_label(), "live");
        assert_eq!(
            DirectorSnapshotSource::FixtureFallback {
                reason: "offline".into()
            }
            .wire_label(),
            "fixture_fallback:offline"
        );
    }
}
