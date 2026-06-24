package se.reflective.quorum.app

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.runtime.remember
import se.reflective.quorum.corebridge.QuorumCoreBridgeFFI
import se.reflective.quorum.ui.QuorumMobileApp

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        // Crash/error reporting (app SDK + Rust core) is initialised once at process
        // start in QuorumApplication.onCreate — see ADR 0004.
        setContent {
            // Production injects the real Rust-backed bridge; previews/tests use
            // PreviewQuorumCoreBridge (the composable default).
            QuorumMobileApp(bridge = remember { QuorumCoreBridgeFFI() })
        }
    }
}
