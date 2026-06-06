import SwiftUI

struct ContentView: View {
    var body: some View {
        VStack(spacing: 16) {
            Text("Reflective")
                .font(.largeTitle)
                .fontWeight(.semibold)
            Text("Native shell template — iOS")
                .font(.subheadline)
                .foregroundStyle(.secondary)
        }
        .padding()
    }
}
