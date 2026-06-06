import SwiftUI

public struct QuorumMobileAppRoot: View {
    public init() {}

    public var body: some View {
        NavigationStack {
            SignalCaptureView()
                .navigationTitle("Quorum")
        }
    }
}

