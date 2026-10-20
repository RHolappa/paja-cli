// SPDX-License-Identifier: Apache-2.0
import SwiftUI

struct PajaVoiceApp: App {
    @StateObject private var workspace = Workspace()
    var body: some Scene {
        WindowGroup("Paja Voice") {
            ContentView(workspace: workspace)
                .frame(minWidth: 700, minHeight: 700)
                .onDisappear { workspace.cancel() }
                .onReceive(NotificationCenter.default.publisher(for: NSApplication.willTerminateNotification)) { _ in
                    workspace.cancel()
                }
        }
        .defaultSize(width: 820, height: 830)
    }
}

struct ContentView: View {
    @ObservedObject var workspace: Workspace
    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 22) {
                HStack {
                    Image(systemName: "waveform.circle.fill").font(.system(size: 46)).foregroundStyle(.orange)
                    VStack(alignment: .leading, spacing: 4) {
                        Text("Paja Voice").font(.largeTitle.bold())
                        Text("Build, test, and improve your AI agent.").foregroundStyle(.secondary)
                    }
                    Spacer()
                    Text("INNOSTIUM OY").font(.caption.weight(.semibold)).foregroundStyle(.secondary)
                }
                Text("Local control preview · The meal helper is a deterministic fixture. Spoken goals are saved as a brief; generating a real agent from that brief is not implemented yet.")
                    .font(.callout).padding().background(.orange.opacity(0.09), in: RoundedRectangle(cornerRadius: 12))

                if !workspace.modelReady {
                    HStack {
                        VStack(alignment: .leading) {
                            Text("Enable private voice input").font(.headline)
                            Text("Download the multilingual model once (roughly 1 GB including support files). Typing works immediately.")
                                .font(.callout).foregroundStyle(.secondary)
                        }
                        Spacer()
                        Button("Download model") { workspace.prepare() }.disabled(workspace.busy)
                    }
                }

                VStack(alignment: .leading, spacing: 12) {
                    HStack {
                        Text("Your prompt").font(.headline)
                        Spacer()
                        Picker("Speech language", selection: $workspace.language) {
                            Text("English").tag("en")
                            Text("Suomi").tag("fi")
                        }.frame(width: 190).disabled(workspace.busy)
                    }
                    TextEditor(text: $workspace.prompt)
                        .font(.body).padding(10).frame(height: 150)
                        .scrollContentBackground(.hidden)
                        .background(.background, in: RoundedRectangle(cornerRadius: 12))
                        .overlay(RoundedRectangle(cornerRadius: 12).stroke(.secondary.opacity(0.25)))
                        .disabled(workspace.busy)
                        .accessibilityLabel("Editable prompt")
                    HStack {
                        if workspace.phase == "recording" {
                            Button { workspace.stop() } label: { Label("Stop · \(workspace.seconds)s / 30s", systemImage: "stop.fill") }
                                .tint(.red).buttonStyle(.borderedProminent)
                        } else {
                            Button { workspace.record() } label: { Label("Record a prompt", systemImage: "mic.fill") }
                                .buttonStyle(.borderedProminent).tint(.orange)
                                .disabled(workspace.busy || !workspace.modelReady)
                        }
                        if workspace.busy && workspace.phase != "coordinator" {
                            Button("Cancel") { workspace.cancel() }
                        }
                        Spacer()
                        Button("Use lunch example") { workspace.prompt = "Choose two lunches for a total of eight euros" }
                            .disabled(workspace.busy)
                    }
                    Text("Audio stays on this Mac. Review the transcript before using it.").font(.caption).foregroundStyle(.secondary)
                    if workspace.modelReady {
                        Button("Repair speech setup") { workspace.prepare() }
                            .font(.caption).buttonStyle(.link).disabled(workspace.busy)
                    }
                }

                Divider()
                HStack {
                    TextField("New project name", text: $workspace.projectName).frame(maxWidth: 220)
                    Button("Create project from prompt") { workspace.operate("init") }
                        .disabled(workspace.busy || workspace.prompt.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty)
                }
                if !workspace.projects.isEmpty {
                    Picker("Project", selection: $workspace.selected) {
                        ForEach(workspace.projects) { project in Text(project.name).tag(Optional(project)) }
                    }.disabled(workspace.busy)
                    HStack {
                        ForEach(["build", "test", "improve", "results", "status"], id: \.self) { action in
                            Button(action.capitalized) { workspace.operate(action) }
                        }
                        Spacer()
                        Button("Run prompt") { workspace.operate("run") }.buttonStyle(.borderedProminent)
                    }.disabled(workspace.busy)
                }
                VStack(alignment: .leading, spacing: 12) {
                    HStack {
                        if workspace.busy { ProgressView().controlSize(.small) }
                        Text(workspace.message).textSelection(.enabled)
                    }.accessibilityElement(children: .combine)
                    if !workspace.details.isEmpty {
                        DisclosureGroup("Execution evidence") {
                            Text(workspace.details).font(.system(.caption, design: .monospaced))
                                .textSelection(.enabled).frame(maxWidth: .infinity, alignment: .leading)
                        }
                    }
                }.padding().frame(maxWidth: .infinity, alignment: .leading)
                    .background(.secondary.opacity(0.07), in: RoundedRectangle(cornerRadius: 12))
                Text("Rakenna, testaa ja kehitä tekoälyagenttiasi.").font(.caption).foregroundStyle(.secondary)
            }.padding(30)
        }
    }
}

@main enum Entry {
    @MainActor static func main() async {
        let arguments = CommandLine.arguments
        // Opt-in, file-only diagnostics use the same speech engine without opening
        // a window or requesting microphone permission.
        if arguments.count > 1 {
            do {
                guard arguments.count >= 4 else { throw VoiceError.message("Usage: PajaVoice --prepare-model ASSETS tiny|large-v3-v20240930_626MB or --transcribe-file ASSETS AUDIO en|fi") }
                let root = URL(fileURLWithPath: arguments[2], isDirectory: true)
                let engine = SpeechEngine()
                if arguments[1] == "--prepare-model", arguments.count == 4 {
                    try await engine.prepare(root: root, model: arguments[3])
                    print("Model ready.")
                } else if arguments[1] == "--transcribe-file", arguments.count == 5 {
                    let text = try await engine.transcribe(file: URL(fileURLWithPath: arguments[3]), language: arguments[4], root: root)
                    let data = try JSONSerialization.data(withJSONObject: ["schema_version": "paja.transcript.v1", "text": text, "language": arguments[4], "backend": "whisperkit-local"])
                    print(String(decoding: data, as: UTF8.self))
                } else { throw VoiceError.message("Unknown diagnostic command.") }
            } catch {
                FileHandle.standardError.write(Data("\(error.localizedDescription)\n".utf8))
                exit(1)
            }
        } else {
            PajaVoiceApp.main()
        }
    }
}
