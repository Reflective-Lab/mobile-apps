//! Live director resolution — HTTP snapshot GET + Client Helm SSE (Plan 2 / Track C).
//!
//! The native layer owns the SSE connection; this module parses session-host
//! envelopes, feeds `helm_client::ClientHelm`, and projects `DirectorSnapshot`
//! locally. Falls back to `GET /api/director/snapshot` until the first SSE
//! event arrives.

use super::{DirectorSnapshot, quorum_director_fixture_snapshot};
use crate::quorum::director_presenter::QuorumDomainPresenter;
use director_contracts::{DirectorIntent, GateVerdict};
use helm_client::ClientHelm;
use helm_session_contracts::gate::GatedDecision;
use helm_session_contracts::push::SessionPush;
use serde::Deserialize;
use std::io::{BufRead, BufReader};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex, OnceLock};
use std::thread::{self, JoinHandle};
use std::time::Duration;

/// Default local Quorum base URL (`just dev` from quorum-sense).
pub const DEFAULT_LOCAL_QUORUM_BASE_URL: &str = "http://127.0.0.1:5161/quorum-sense";

/// Bearer token accepted when `LOCAL_DEV=true` on the Quorum server.
pub const DEFAULT_LOCAL_DEV_BEARER: &str = "dev";

/// Default decision session for local Track A dev push / SSE stream.
pub const DEFAULT_DIRECTOR_SESSION_ID: &str = "procurement-security-review";

/// Canonical director snapshot route (server-side Plan 2).
pub const DIRECTOR_SNAPSHOT_PATH: &str = "/api/director/snapshot";

/// LOCAL_DEV intent submit route (Track B).
pub const DIRECTOR_INTENT_PATH: &str = "/api/director/dev/intent";

const SESSION_PUSH: &str = "session.push";
const SESSION_GATE_OPENED: &str = "session.gate.opened";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DirectorApiConfig {
    pub base_url: String,
    pub bearer_token: String,
    pub session_id: String,
}

impl DirectorApiConfig {
    #[must_use]
    pub fn new(base_url: String, bearer_token: String) -> Self {
        Self {
            base_url,
            bearer_token,
            session_id: DEFAULT_DIRECTOR_SESSION_ID.to_owned(),
        }
    }

    #[must_use]
    pub fn with_session_id(mut self, session_id: String) -> Self {
        self.session_id = session_id;
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DirectorSnapshotSource {
    /// Projected locally from Client Helm after an SSE session-host event.
    LiveSse,
    /// Resolved from Quorum `GET /api/director/snapshot`.
    Live,
    FixtureOnly,
    FixtureFallback {
        reason: String,
    },
}

#[derive(Debug, thiserror::Error)]
pub enum LiveDirectorError {
    #[error("HTTP {status}: {body}")]
    HttpError { status: u16, body: String },
    #[error("network error: {0}")]
    Transport(Box<ureq::Error>),
    #[error("failed to read response body: {0}")]
    BodyRead(#[from] std::io::Error),
    #[error("invalid snapshot JSON: {0}")]
    Json(#[from] serde_json::Error),
}

impl From<ureq::Error> for LiveDirectorError {
    fn from(err: ureq::Error) -> Self {
        Self::Transport(Box::new(err))
    }
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
            Self::LiveSse => "live_sse".to_owned(),
            Self::Live => "live".to_owned(),
            Self::FixtureOnly => "fixture".to_owned(),
            Self::FixtureFallback { reason } => format!("fixture_fallback:{reason}"),
        }
    }
}

#[derive(Debug, Deserialize)]
struct StreamEnvelope {
    sequence: u64,
    #[serde(rename = "type")]
    event_type: String,
    payload: serde_json::Value,
}

struct LiveSessionInner {
    helm: ClientHelm,
    version: u64,
    snapshot: Option<DirectorSnapshot>,
    has_sse_events: bool,
}

struct LiveSessionState {
    inner: Mutex<LiveSessionInner>,
    updated: Condvar,
}

struct SseListenerHandle {
    stop: Arc<AtomicBool>,
    join: Option<JoinHandle<()>>,
}

static LIVE_SESSION: OnceLock<LiveSessionState> = OnceLock::new();
static SSE_LISTENER: Mutex<Option<SseListenerHandle>> = Mutex::new(None);

fn live_session() -> Option<&'static LiveSessionState> {
    LIVE_SESSION.get()
}

fn ensure_live_session() -> &'static LiveSessionState {
    LIVE_SESSION.get_or_init(|| LiveSessionState {
        inner: Mutex::new(LiveSessionInner {
            helm: ClientHelm::new(),
            version: 0,
            snapshot: None,
            has_sse_events: false,
        }),
        updated: Condvar::new(),
    })
}

/// Build the snapshot GET URL from a configured base (trailing slash tolerated).
#[must_use]
pub fn director_snapshot_url(config: &DirectorApiConfig) -> String {
    let base = config.base_url.trim_end_matches('/');
    format!("{base}{DIRECTOR_SNAPSHOT_PATH}")
}

/// Build the session-host SSE URL for Client Helm event intake.
#[must_use]
pub fn director_session_stream_url(config: &DirectorApiConfig) -> String {
    let base = config.base_url.trim_end_matches('/');
    format!(
        "{base}/v1/sessions/{}/stream",
        config.session_id.trim_matches('/')
    )
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

/// Build the LOCAL_DEV intent POST URL from a configured base.
#[must_use]
pub fn director_intent_url(config: &DirectorApiConfig) -> String {
    let base = config.base_url.trim_end_matches('/');
    format!("{base}{DIRECTOR_INTENT_PATH}")
}

/// POST a typed director intent to Quorum (`LOCAL_DEV` route). Requires network access.
pub fn submit_director_intent(
    config: &DirectorApiConfig,
    intent: &DirectorIntent,
) -> Result<(), LiveDirectorError> {
    let url = director_intent_url(config);
    let mut body = serde_json::to_value(intent).map_err(LiveDirectorError::Json)?;
    if let Some(obj) = body.as_object_mut() {
        obj.insert(
            "session_id".to_owned(),
            serde_json::Value::String(config.session_id.clone()),
        );
    }
    let agent = ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_secs(5))
        .timeout_read(Duration::from_secs(10))
        .build();

    let response = agent
        .post(&url)
        .set("Authorization", &format!("Bearer {}", config.bearer_token))
        .set("Content-Type", "application/json")
        .send_json(body)?;

    let status = response.status();
    if status != 200 {
        let body = response
            .into_string()
            .map_err(LiveDirectorError::BodyRead)?;
        return Err(LiveDirectorError::HttpError { status, body });
    }
    Ok(())
}

/// Apply a director intent to the in-memory Client Helm mirror (immediate UI feedback).
pub fn apply_local_director_intent(intent: &DirectorIntent) {
    let state = ensure_live_session();
    let Ok(mut guard) = state.inner.lock() else {
        return;
    };

    if let DirectorIntent::RespondGate { gate_id, verdict } = intent {
        let response = serde_json::json!({
            "verdict": match verdict {
                GateVerdict::Approve => "approve",
                GateVerdict::Reject => "reject",
            }
        });
        guard.helm.respond_to_gate(gate_id, response);
    }

    let presenter = QuorumDomainPresenter;
    let version = guard.version.saturating_add(1);
    let snapshot = guard.helm.director_snapshot(version, &presenter);
    guard.version = version;
    guard.snapshot = Some(snapshot);
    drop(guard);
    state.updated.notify_all();
}

/// Start (or restart) the background Client Helm SSE listener for the given config.
pub fn start_director_sse_listener(config: DirectorApiConfig) {
    stop_director_sse_listener();

    let stop = Arc::new(AtomicBool::new(false));
    let stop_flag = Arc::clone(&stop);
    let state = ensure_live_session();
    let join = thread::spawn(move || sse_listener_loop(config, state, stop_flag));

    if let Ok(mut slot) = SSE_LISTENER.lock() {
        *slot = Some(SseListenerHandle {
            stop,
            join: Some(join),
        });
    }
}

/// Stop the background SSE listener, if running.
pub fn stop_director_sse_listener() {
    let handle = SSE_LISTENER.lock().ok().and_then(|mut slot| slot.take());
    if let Some(handle) = handle {
        handle.stop.store(true, Ordering::Relaxed);
        if let Some(join) = handle.join {
            let _ = join.join();
        }
    }
}

/// Reset in-memory Client Helm SSE state (tests only).
#[cfg(test)]
pub fn reset_live_session_for_tests() {
    stop_director_sse_listener();
    if let Some(state) = live_session()
        && let Ok(mut guard) = state.inner.lock()
    {
        *guard = LiveSessionInner {
            helm: ClientHelm::new(),
            version: 0,
            snapshot: None,
            has_sse_events: false,
        };
    }
}

/// Block until the Client Helm snapshot version exceeds `since`, or `timeout` elapses.
#[must_use]
pub fn wait_director_snapshot_version(since: u64, timeout: Duration) -> bool {
    let Some(state) = live_session() else {
        return false;
    };
    let Ok(mut guard) = state.inner.lock() else {
        return false;
    };
    let deadline = std::time::Instant::now().checked_add(timeout);
    while guard.version <= since {
        let Some(deadline) = deadline else {
            return false;
        };
        let remaining = deadline.saturating_duration_since(std::time::Instant::now());
        if remaining.is_zero() {
            return false;
        }
        let wait = state.updated.wait_timeout(guard, remaining);
        guard = match wait {
            Ok((g, _)) => g,
            Err(_) => return false,
        };
    }
    guard.version > since
}

/// Resolve the snapshot the mobile UI should render.
///
/// Prefers Client Helm SSE projection when events have been received; otherwise
/// tries live HTTP GET; then fixture fallback / fixture-only.
pub fn resolve_director_snapshot(
    config: Option<&DirectorApiConfig>,
) -> Result<(DirectorSnapshot, DirectorSnapshotSource), ResolveDirectorError> {
    if config.is_some()
        && let Some(state) = live_session()
        && let Ok(guard) = state.inner.lock()
        && guard.has_sse_events
        && let Some(snapshot) = guard.snapshot.clone()
    {
        return Ok((snapshot, DirectorSnapshotSource::LiveSse));
    }

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

fn sse_listener_loop(
    config: DirectorApiConfig,
    state: &'static LiveSessionState,
    stop: Arc<AtomicBool>,
) {
    while !stop.load(Ordering::Relaxed) {
        let _ = read_sse_stream(&config, state, &stop);
        if stop.load(Ordering::Relaxed) {
            break;
        }
        thread::sleep(Duration::from_secs(2));
    }
}

fn read_sse_stream(
    config: &DirectorApiConfig,
    state: &LiveSessionState,
    stop: &AtomicBool,
) -> Result<(), LiveDirectorError> {
    let url = director_session_stream_url(config);
    let agent = ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_secs(5))
        .timeout_read(Duration::from_secs(300))
        .build();

    let response = agent
        .get(&url)
        .set("Authorization", &format!("Bearer {}", config.bearer_token))
        .set("Accept", "text/event-stream")
        .call()?;

    if response.status() != 200 {
        let status = response.status();
        let body = response
            .into_string()
            .map_err(LiveDirectorError::BodyRead)?;
        return Err(LiveDirectorError::HttpError { status, body });
    }

    let reader = response.into_reader();
    let mut lines = BufReader::new(reader).lines();
    let mut pending = SseEventBuilder::default();

    while !stop.load(Ordering::Relaxed) {
        let line = match lines.next() {
            Some(Ok(line)) => line,
            Some(Err(err)) => return Err(LiveDirectorError::BodyRead(err)),
            None => break,
        };

        if line.is_empty() {
            if let Some(envelope) = pending.take_envelope() {
                apply_stream_envelope(state, &envelope);
            }
            continue;
        }

        pending.ingest_line(&line);
    }

    Ok(())
}

#[derive(Default)]
struct SseEventBuilder {
    data_lines: Vec<String>,
}

impl SseEventBuilder {
    fn ingest_line(&mut self, line: &str) {
        if let Some(rest) = line.strip_prefix("data:") {
            self.data_lines.push(rest.trim_start().to_owned());
        }
    }

    fn take_envelope(&mut self) -> Option<StreamEnvelope> {
        if self.data_lines.is_empty() {
            return None;
        }
        let data = self.data_lines.join("\n");
        self.data_lines.clear();
        serde_json::from_str(&data).ok()
    }
}

fn apply_stream_envelope(state: &LiveSessionState, envelope: &StreamEnvelope) {
    let Ok(mut guard) = state.inner.lock() else {
        return;
    };

    match envelope.event_type.as_str() {
        SESSION_PUSH => {
            if let Ok(push) = serde_json::from_value::<SessionPush>(envelope.payload.clone()) {
                let _action = guard.helm.handle_push(push);
            }
        }
        SESSION_GATE_OPENED => {
            if let Some(gate_value) = envelope.payload.get("gate")
                && let Ok(gate) = serde_json::from_value::<GatedDecision>(gate_value.clone())
            {
                guard.helm.handle_gate(gate);
            }
        }
        _ => return,
    }

    let presenter = QuorumDomainPresenter;
    let snapshot = guard.helm.director_snapshot(envelope.sequence, &presenter);
    guard.version = envelope.sequence;
    guard.snapshot = Some(snapshot);
    guard.has_sse_events = true;
    drop(guard);
    state.updated.notify_all();
}

#[cfg(test)]
mod tests {
    use super::*;
    use helm_session_contracts::finding::FindingId;
    use helm_session_contracts::push::SessionContext;
    use helm_session_contracts::urgency::UrgencyIntent;
    use std::sync::Mutex;

    /// Global `LIVE_SESSION` is process-wide; serialize mutating tests.
    static LIVE_DIRECTOR_TEST_LOCK: Mutex<()> = Mutex::new(());

    fn live_session_test_lock() -> std::sync::MutexGuard<'static, ()> {
        LIVE_DIRECTOR_TEST_LOCK
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    #[test]
    fn snapshot_url_joins_base_and_path() {
        let config = DirectorApiConfig {
            base_url: "http://127.0.0.1:5161/quorum-sense/".into(),
            bearer_token: "dev".into(),
            session_id: DEFAULT_DIRECTOR_SESSION_ID.into(),
        };
        assert_eq!(
            director_snapshot_url(&config),
            "http://127.0.0.1:5161/quorum-sense/api/director/snapshot"
        );
        assert_eq!(
            director_session_stream_url(&config),
            "http://127.0.0.1:5161/quorum-sense/v1/sessions/procurement-security-review/stream"
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
        let _lock = live_session_test_lock();
        reset_live_session_for_tests();
        let config = DirectorApiConfig {
            base_url: "http://127.0.0.1:1/unreachable".into(),
            bearer_token: "dev".into(),
            session_id: DEFAULT_DIRECTOR_SESSION_ID.into(),
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
        assert_eq!(DirectorSnapshotSource::LiveSse.wire_label(), "live_sse");
        assert_eq!(
            DirectorSnapshotSource::FixtureFallback {
                reason: "offline".into()
            }
            .wire_label(),
            "fixture_fallback:offline"
        );
    }

    #[test]
    fn sse_envelope_updates_client_helm_projection() {
        let _lock = live_session_test_lock();
        reset_live_session_for_tests();
        let state = ensure_live_session();
        let push = SessionPush {
            finding_id: FindingId::from_string("find-sse-test"),
            urgency_intent: UrgencyIntent::Advisory,
            payload: serde_json::json!({"objective": "SSE probe"}),
            session_context: SessionContext {
                session_id: DEFAULT_DIRECTOR_SESSION_ID.into(),
                phase: "decision".into(),
                cycle: 1,
                timestamp_ms: 1,
            },
        };
        let payload = serde_json::to_value(&push).expect("serialize push");
        apply_stream_envelope(
            state,
            &StreamEnvelope {
                sequence: 7,
                event_type: SESSION_PUSH.to_owned(),
                payload,
            },
        );

        let guard = state.inner.lock().expect("lock");
        assert!(guard.has_sse_events);
        assert_eq!(guard.version, 7);
        assert_eq!(guard.snapshot.as_ref().map(|s| s.version), Some(7));
    }

    #[test]
    fn resolve_prefers_sse_projection_over_http() {
        let _lock = live_session_test_lock();
        reset_live_session_for_tests();
        let state = ensure_live_session();
        apply_stream_envelope(
            state,
            &StreamEnvelope {
                sequence: 3,
                event_type: SESSION_PUSH.to_owned(),
                payload: serde_json::to_value(SessionPush {
                    finding_id: FindingId::from_string("find-prefer-sse"),
                    urgency_intent: UrgencyIntent::Advisory,
                    payload: serde_json::json!({}),
                    session_context: SessionContext {
                        session_id: DEFAULT_DIRECTOR_SESSION_ID.into(),
                        phase: "decision".into(),
                        cycle: 1,
                        timestamp_ms: 1,
                    },
                })
                .expect("serialize"),
            },
        );

        let config = DirectorApiConfig {
            base_url: "http://127.0.0.1:1/unreachable".into(),
            bearer_token: "dev".into(),
            session_id: DEFAULT_DIRECTOR_SESSION_ID.into(),
        };
        let (snapshot, source) = resolve_director_snapshot(Some(&config)).expect("sse projection");
        assert_eq!(source, DirectorSnapshotSource::LiveSse);
        assert_eq!(snapshot.version, 3);
    }
}
