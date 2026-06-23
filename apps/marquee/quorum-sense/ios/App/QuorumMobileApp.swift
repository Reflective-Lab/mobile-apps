import Sentry
import SwiftUI

@main
struct QuorumMobileApp: App {
    init() {
        SentrySDK.start { options in
            // DSN is a public client key (safe to commit). From Sentry project
            // apple-ios (id 4511614599888976) → Client Keys (DSN). Org is EU-region.
            options.dsn = "https://e2540a9ee8224d64f10d0f46ac41be03@o4511614588223488.ingest.de.sentry.io/4511614599888976"

            #if DEBUG
            options.debug = true
            options.environment = "debug"
            #else
            options.environment = "production"
            #endif

            // Performance + profiling. Dial these down for production volume.
            options.tracesSampleRate = 1.0
            options.configureProfiling = {
                $0.sessionSampleRate = 1.0
                $0.lifecycle = .trace
            }
        }
    }

    var body: some Scene {
        WindowGroup {
            RootView()
        }
    }
}

struct RootView: View {
    var body: some View {
        NavigationStack {
            SignalCaptureView(bridge: QuorumCoreBridgeFFI())
                .navigationTitle("Quorum")
        }
    }
}
