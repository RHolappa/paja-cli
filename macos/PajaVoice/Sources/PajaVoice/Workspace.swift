// SPDX-License-Identifier: Apache-2.0
import AppKit
import AVFoundation
import Foundation
import SwiftUI

@MainActor final class Workspace: ObservableObject {
    @Published var prompt = ""
    @Published var language = "en"
    @Published var projectName = "meal-helper"
    @Published var projects: [LocalProject] = []
    @Published var selected: LocalProject?
    @Published var phase = "idle"
    @Published var message = "Describe your agent’s job, or try the lunch-helper control example."
    @Published var details = ""
    @Published var seconds = 0
    @Published var modelReady = false
    private var recorder: AVAudioRecorder?
    private var recordingURL: URL?
    private var task: Task<Void, Never>?
    private var epoch = UUID()
    private let speech = SpeechEngine()
    let root: URL
    var assets: URL { root.appendingPathComponent("speech") }
    var projectRoot: URL { root.appendingPathComponent("projects") }
    var busy: Bool { phase != "idle" }

    init() {
        root = FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask)[0]
            .appendingPathComponent("PajaVoice")
        try? FileManager.default.createDirectory(at: projectRoot, withIntermediateDirectories: true,
                                                attributes: [.posixPermissions: 0o700])
        if let data = try? Data(contentsOf: root.appendingPathComponent("projects.json")),
           let saved = try? JSONDecoder().decode([LocalProject].self, from: data) {
            projects = saved
            selected = saved.first
        }
        modelReady = FileManager.default.fileExists(atPath: assets.appendingPathComponent("model-path.txt").path)
    }

    func prepare() {
        guard !busy else { return }
        phase = "setup"
        message = "Downloading the multilingual speech model. This happens once; audio stays on your Mac."
        let ticket = UUID(); epoch = ticket
        task = Task {
            do {
                try await speech.prepare(root: assets)
                guard epoch == ticket else { return }
                modelReady = true; phase = "idle"
                message = "Speech is ready. Press the microphone and speak in the selected language."
            } catch { fail(error, ticket: ticket) }
        }
    }

    func record() {
        guard !busy, modelReady else { return }
        phase = "permission"
        message = "Waiting for microphone permission…"
        let ticket = UUID(); epoch = ticket
        task = Task {
            let allowed = await AVCaptureDevice.requestAccess(for: .audio)
            guard epoch == ticket else { return }
            guard allowed else {
                fail(VoiceError.message("Allow Paja Voice in System Settings → Privacy & Security → Microphone."), ticket: ticket)
                return
            }
            do {
                let directory = FileManager.default.temporaryDirectory.appendingPathComponent("paja-audio-\(UUID().uuidString)")
                try FileManager.default.createDirectory(at: directory, withIntermediateDirectories: true,
                                                       attributes: [.posixPermissions: 0o700])
                let url = directory.appendingPathComponent("prompt.wav")
                recordingURL = url
                let audio = try AVAudioRecorder(url: url, settings: [
                    AVFormatIDKey: kAudioFormatLinearPCM, AVSampleRateKey: 16000,
                    AVNumberOfChannelsKey: 1, AVLinearPCMBitDepthKey: 16,
                    AVLinearPCMIsFloatKey: false, AVLinearPCMIsBigEndianKey: false
                ])
                recorder = audio
                guard audio.record(forDuration: 30) else { throw VoiceError.message("Could not start the microphone.") }
                seconds = 0; phase = "recording"
                message = "Listening… Stop when you’re done. Your existing draft is preserved until transcription succeeds."
                for tick in 1...30 {
                    try await Task.sleep(nanoseconds: 1_000_000_000)
                    guard epoch == ticket, phase == "recording" else { return }
                    seconds = tick
                }
                stop()
            } catch is CancellationError { } catch { fail(error, ticket: ticket) }
        }
    }

    func stop() {
        guard phase == "recording", let file = recordingURL else { return }
        recorder?.stop(); recorder = nil; recordingURL = nil
        task?.cancel()
        phase = "transcribing"; message = "Transcribing on your Mac…"
        let ticket = epoch
        let recordedLanguage = language
        task = Task {
            defer { try? FileManager.default.removeItem(at: file.deletingLastPathComponent()) }
            do {
                let text = try await speech.transcribe(file: file, language: recordedLanguage, root: assets)
                guard epoch == ticket else { return }
                prompt = text; phase = "idle"
                message = "Review your transcript, then choose what to do with it."
            } catch { fail(error, ticket: ticket) }
        }
    }

    func cancel() {
        guard phase != "coordinator" else { return }
        epoch = UUID(); task?.cancel(); recorder?.stop(); recorder = nil
        if let file = recordingURL { try? FileManager.default.removeItem(at: file.deletingLastPathComponent()) }
        recordingURL = nil; phase = "idle"
        message = "Cancelled. Your draft is unchanged."
    }

    func fail(_ error: Error, ticket: UUID) {
        guard epoch == ticket else { return }
        recorder?.stop(); recorder = nil
        if let file = recordingURL { try? FileManager.default.removeItem(at: file.deletingLastPathComponent()) }
        recordingURL = nil; phase = "idle"; message = error.localizedDescription
    }

    func operate(_ operation: String) {
        guard !busy else { return }
        let text = prompt, name = projectName, project = selected
        phase = "coordinator"; message = "\(operation.capitalized)…"
        let ticket = UUID(); epoch = ticket
        task = Task {
            var file: URL?
            defer { if let file { try? FileManager.default.removeItem(at: file.deletingLastPathComponent()) } }
            do {
                var response: [String: Any]
                let request = UUID().uuidString
                if operation == "init" {
                    file = try Coordinator.promptFile(text)
                    response = try await Coordinator.call(arguments: ["init", name, "--goal-file", file!.path, "--request-id", request], directory: projectRoot)
                    guard let metadata = response["project"] as? [String: Any], let id = metadata["id"] as? String else {
                        throw VoiceError.message("Project identity missing from response.")
                    }
                    let new = LocalProject(id: id, name: name)
                    projects.append(new); selected = new
                    try JSONEncoder().encode(projects).write(to: root.appendingPathComponent("projects.json"), options: .atomic)
                } else {
                    guard let project else { throw VoiceError.message("Create a project first.") }
                    let directory = projectRoot.appendingPathComponent(project.name)
                    let status = try await Coordinator.call(arguments: ["status", "--project", project.id], directory: directory)
                    let metadata = status["project"] as? [String: Any] ?? [:]
                    let revision = metadata["candidate_revision"] as? Int ?? 0
                    var args = [operation, "--project", project.id]
                    if ["build", "test", "improve", "run"].contains(operation) {
                        args += ["--revision", String(revision), "--request-id", request]
                    }
                    if operation == "run" {
                        file = try Coordinator.promptFile(text); args += ["--input-file", file!.path]
                    }
                    if ["improve", "results"].contains(operation) {
                        let data = status["data"] as? [String: Any] ?? [:]
                        guard let run = data["latest_test_run"] as? String else { throw VoiceError.message("Test this version first.") }
                        args += [operation == "improve" ? "--from" : "--run", run]
                    }
                    response = operation == "status" ? status : try await Coordinator.call(arguments: args, directory: directory)
                }
                message = Coordinator.summary(response)
                details = String(data: try JSONSerialization.data(withJSONObject: response, options: [.prettyPrinted, .sortedKeys]), encoding: .utf8) ?? ""
                phase = "idle"
            } catch { fail(error, ticket: ticket) }
        }
    }
}
