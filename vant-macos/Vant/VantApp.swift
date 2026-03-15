import SwiftUI

@main
struct VantApp: App {
    var body: some Scene {
        WindowGroup {
            ContentView()
        }
    }
}

struct ContentView: View {
    var body: some View {
        VStack(spacing: 16) {
            Text("Vant Settings")
                .font(.largeTitle)
            Text("AI-Powered Vietnamese Input Method")
                .foregroundColor(.secondary)
            Text("Version 0.1.0")
                .font(.caption)
                .foregroundColor(.secondary)
        }
        .padding(40)
    }
}
