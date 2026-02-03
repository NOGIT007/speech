import SwiftUI

struct MenuBarIcon: View {
    let isRecording: Bool
    let isTranscribing: Bool

    var body: some View {
        Image(systemName: iconName)
            .symbolRenderingMode(.hierarchical)
            .foregroundStyle(iconColor)
            .font(.body)
    }

    private var iconName: String {
        if isRecording {
            return "waveform.badge.mic"
        } else if isTranscribing {
            return "text.bubble"
        } else {
            return "waveform"
        }
    }

    private var iconColor: Color {
        if isRecording {
            return .red
        } else if isTranscribing {
            return .orange
        } else {
            return .primary
        }
    }
}
