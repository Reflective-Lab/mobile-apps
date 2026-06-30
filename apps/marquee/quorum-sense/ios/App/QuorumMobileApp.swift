import Sentry
import SwiftUI
import UIKit

@main
struct QuorumMobileApp: App {
    init() {
        Brand.registerFonts()
        Self.configureNavigationBarAppearance()

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

            // Telemetry-PII boundary (QF-2026-06-24-05, ADR 0004). Field-signal
            // capture (transcript / OCR / free text) is sensitive and must not leave
            // the device via the crash pipe. Keep default PII off and strip device/
            // user identifiers before any event ships.
            options.sendDefaultPii = false
            options.beforeSend = { event in
                event.serverName = nil // device hostname
                event.user = nil // id / username / ip / geo
                return event
            }

            // Performance + profiling. Dial these down for production volume.
            options.tracesSampleRate = 1.0
            options.configureProfiling = {
                $0.sessionSampleRate = 1.0
                $0.lifecycle = .trace
            }
        }

        // Rust core crash/error reporting → the separate Rust Sentry project
        // (id 4511614643142736). `initObservability` is the UniFFI-generated entry
        // point (CoreBridge); the DSN is a public client key (safe to commit).
        #if DEBUG
        let rustEnvironment = "debug"
        let rustDebug = true
        #else
        let rustEnvironment = "production"
        let rustDebug = false
        #endif
        let shortVersion = Bundle.main.infoDictionary?["CFBundleShortVersionString"] as? String ?? "0.0.0"
        initObservability(
            dsn: "https://096bf7f5a5e69d38023975659d020217@o4511614588223488.ingest.de.sentry.io/4511614643142736",
            environment: rustEnvironment,
            release: "quorum@\(shortVersion)",
            debug: rustDebug
        )

        Self.configureDirectorApiIfPresent()
    }

    /// When `QUORUM_BASE_URL` is set (scheme env or Xcode scheme), wire live director + SSE.
    private static func configureDirectorApiIfPresent() {
        let env = ProcessInfo.processInfo.environment
        let quorumBase = env["QUORUM_BASE_URL"]
            ?? {
                #if DEBUG
                return "http://127.0.0.1:5161/quorum-sense"
                #else
                return nil
                #endif
            }()
        guard let quorumBase else { return }
        let quorumBearer = env["QUORUM_BEARER_TOKEN"] ?? "dev"
        quorumConfigureDirectorApi(baseUrl: quorumBase, bearerToken: quorumBearer)
    }

    var body: some Scene {
        WindowGroup {
            RootView()
                .tint(Brand.accent)
        }
    }

    /// Brands the navigation bar: DM Serif Display large title on the paper canvas.
    /// Fonts must already be registered (see `Brand.registerFonts()` above).
    private static func configureNavigationBarAppearance() {
        let appearance = UINavigationBarAppearance()
        appearance.configureWithOpaqueBackground()
        appearance.backgroundColor = UIColor(Brand.paper)
        appearance.shadowColor = UIColor(Brand.line)
        if let large = UIFont(name: Brand.FontName.display, size: 34) {
            appearance.largeTitleTextAttributes = [.font: large, .foregroundColor: UIColor(Brand.ink)]
        }
        if let inline = UIFont(name: Brand.FontName.sansMedium, size: 17) {
            appearance.titleTextAttributes = [.font: inline, .foregroundColor: UIColor(Brand.ink)]
        }
        UINavigationBar.appearance().standardAppearance = appearance
        UINavigationBar.appearance().scrollEdgeAppearance = appearance
        UINavigationBar.appearance().compactAppearance = appearance
    }
}

@MainActor
struct RootView: View {
    private let bridge: QuorumCoreBridgeFFI

    init() {
        do {
            bridge = try QuorumCoreBridgeFFI()
        } catch {
            fatalError("Failed to initialise Quorum core bridge: \(error)")
        }
    }

    var body: some View {
        NavigationStack {
            DirectorNowView(bridge: bridge)
                .navigationTitle("Director")
                .navigationBarTitleDisplayMode(.inline)
                .toolbar {
                    ToolbarItem(placement: .topBarTrailing) {
                        Menu {
                            NavigationLink("Signal Capture") {
                                SignalCaptureView(bridge: bridge)
                                    .navigationTitle("Signal Capture")
                            }
                        } label: {
                            Label("More", systemImage: "ellipsis.circle")
                        }
                        .accessibilityLabel("Context escape")
                    }
                }
        }
    }
}
