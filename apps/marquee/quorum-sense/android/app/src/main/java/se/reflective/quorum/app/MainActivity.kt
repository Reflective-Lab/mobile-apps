package se.reflective.quorum.app

import android.content.pm.ApplicationInfo
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.runtime.remember
import se.reflective.quorum.corebridge.QuorumCoreBridgeFFI
import se.reflective.quorum.ui.QuorumMobileApp
import uniffi.quorum_ffi.initObservability

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        // Rust core crash/error reporting → the separate Rust Sentry project
        // (id 4511614643142736). Idempotent on the Rust side, so calling
        // per-activity-create is safe. The DSN is a public client key (safe to commit).
        val debuggable = (applicationInfo.flags and ApplicationInfo.FLAG_DEBUGGABLE) != 0
        initObservability(
            dsn = "https://096bf7f5a5e69d38023975659d020217@o4511614588223488.ingest.de.sentry.io/4511614643142736",
            environment = if (debuggable) "debug" else "production",
            release = "quorum@0.1.0",
            debug = debuggable,
        )

        setContent {
            // Production injects the real Rust-backed bridge; previews/tests use
            // PreviewQuorumCoreBridge (the composable default).
            QuorumMobileApp(bridge = remember { QuorumCoreBridgeFFI() })
        }
    }
}
