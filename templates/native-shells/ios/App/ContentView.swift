import SwiftUI

struct ContentView: View {
    private let message = greet(name: "iOS")
    private let version = coreVersion()

    var body: some View {
        VStack(spacing: 16) {
            Text("Reflective")
                .font(.largeTitle)
                .fontWeight(.semibold)
            Text("Native shell template — iOS")
                .font(.subheadline)
                .foregroundStyle(.secondary)
            Text(message)
                .font(.body)
                .padding(.top, 8)
            Text("Rust core v\(version)")
                .font(.caption2)
                .foregroundStyle(.tertiary)
        }
        .padding()
    }
}
