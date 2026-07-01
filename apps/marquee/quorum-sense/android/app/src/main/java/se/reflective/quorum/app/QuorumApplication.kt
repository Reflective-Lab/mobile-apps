package se.reflective.quorum.app

import android.app.Application
import android.content.pm.ApplicationInfo
import io.sentry.SentryEvent
import io.sentry.SentryOptions
import io.sentry.android.core.SentryAndroid
import uniffi.quorum_ffi.initObservability
import uniffi.quorum_ffi.quorumConfigureCaptureApi
import uniffi.quorum_ffi.quorumConfigureDirectorApi

/**
 * Single place both crash clients initialise (QF-2026-06-24-05, ADR 0004):
 *
 * 1. The **app** Sentry SDK — code-based `SentryAndroid.init` so we can register a
 *    `beforeSend` scrub. A manifest auto-init cannot, so auto-init is disabled in
 *    `AndroidManifest.xml` (`io.sentry.auto-init=false`) and we init here. `init`
 *    still reads the `io.sentry.*` manifest meta-data (DSN, environment, traces
 *    sample rate, `send-default-pii=false`) — that stays the single source of config
 *    — and our lambda only adds the PII scrub on top.
 * 2. The **Rust core** reporter — `initObservability` ships Rust panics to a separate
 *    Sentry project (its own scrub lives in `ffi/src/lib.rs`). It is idempotent; we
 *    call it once here at process start rather than per-activity-create.
 *
 * Both run in `Application.onCreate`, before any activity, mirroring the iOS app.
 */
class QuorumApplication : Application() {
    override fun onCreate() {
        super.onCreate()

        SentryAndroid.init(this) { options ->
            options.beforeSend = SentryOptions.BeforeSendCallback { event, _ -> scrubPii(event) }
        }

        // Rust core crash/error reporting → the separate Rust Sentry project
        // (id 4511614643142736). The DSN is a public client key (safe to commit).
        val debuggable = (applicationInfo.flags and ApplicationInfo.FLAG_DEBUGGABLE) != 0
        initObservability(
            dsn = "https://096bf7f5a5e69d38023975659d020217@o4511614588223488.ingest.de.sentry.io/4511614643142736",
            environment = if (debuggable) "debug" else "production",
            release = "quorum@0.1.2",
            debug = debuggable,
        )

        if (debuggable) {
            val quorumBase = System.getenv("QUORUM_BASE_URL")
                ?: "http://127.0.0.1:5161/quorum-sense"
            val quorumBearer = System.getenv("QUORUM_BEARER_TOKEN") ?: "dev"
            quorumConfigureDirectorApi(quorumBase, quorumBearer)
            quorumConfigureCaptureApi(quorumBase, quorumBearer)
        }
    }
}

/**
 * Strip device/user identifiers before a crash event ships (ADR 0004). The panic
 * message + stack are retained (needed to debug a crash, on the contract that they
 * must not embed user capture); only the ambient identity Sentry can attach is
 * removed. Extracted from the init lambda so it is unit-testable without an Android
 * context.
 */
internal fun scrubPii(event: SentryEvent): SentryEvent {
    event.serverName = null // device hostname
    event.user = null // id / username / ip / geo
    return event
}
