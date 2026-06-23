import SwiftUI

@main
struct QuorumMobileApp: App {
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
