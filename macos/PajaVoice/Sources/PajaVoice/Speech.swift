// SPDX-License-Identifier: Apache-2.0
import AVFoundation
import ArgmaxCore
import Foundation
import WhisperKit

enum VoiceError: LocalizedError {
    case message(String)
    var errorDescription: String? { switch self { case .message(let value): return value } }
}

// Validate the complete local tokenizer before entering WhisperKit's loader,
// which otherwise falls back to a download when the local files are missing.
private final class LocalWhisper: WhisperKit {
    override func loadTokenizerIfNeeded() async throws {
        guard let folder = tokenizerFolder else {
            throw VoiceError.message("Download the speech model first.")
        }
        _ = try await AutoTokenizerWrapper.from(modelFolder: folder)
        try await super.loadTokenizerIfNeeded()
    }
}

actor SpeechEngine {
    static let model = "large-v3-v20240930_626MB"
    private var pipeline: WhisperKit?
    private var loadedRoot: URL?

    func prepare(root: URL, model: String = SpeechEngine.model) async throws {
        guard [Self.model, "tiny"].contains(model) else {
            throw VoiceError.message("Unsupported speech model.")
        }
        try FileManager.default.createDirectory(at: root, withIntermediateDirectories: true,
                                               attributes: [.posixPermissions: 0o700])
        let folder = try await WhisperKit.download(variant: model, downloadBase: root)
        let tokenizerRoot = root.appendingPathComponent("tokenizer")
        try FileManager.default.createDirectory(at: tokenizerRoot, withIntermediateDirectories: true)
        // Explicit setup is the only network path. No audio or prompts are sent.
        let tokenizerName = model == "tiny" ? "whisper-tiny" : "whisper-large-v3"
        for name in ["tokenizer.json", "tokenizer_config.json"] {
            let url = URL(string: "https://huggingface.co/openai/\(tokenizerName)/resolve/main/\(name)")!
            let (data, response) = try await URLSession.shared.data(from: url)
            guard (response as? HTTPURLResponse)?.statusCode == 200, data.count < 20_000_000 else {
                throw VoiceError.message("Could not download tokenizer. Retry setup when connected.")
            }
            try data.write(to: tokenizerRoot.appendingPathComponent(name), options: .atomic)
        }
        let relative = String(folder.path.dropFirst(root.path.count + 1))
        try Data(relative.utf8).write(to: root.appendingPathComponent("model-path.txt"), options: .atomic)
        pipeline = nil
        try await load(root: root)
    }

    private func load(root: URL) async throws {
        if pipeline != nil, loadedRoot == root { return }
        let marker = root.appendingPathComponent("model-path.txt")
        guard let relative = try? String(contentsOf: marker, encoding: .utf8),
              !relative.hasPrefix("/"), !relative.split(separator: "/").contains("..") else {
            throw VoiceError.message("Download the local speech model in Settings first.")
        }
        pipeline = try await LocalWhisper(WhisperKitConfig(
            modelFolder: root.appendingPathComponent(relative).path,
            tokenizerFolder: root.appendingPathComponent("tokenizer"),
            verbose: false, prewarm: false, load: true, download: false
        ))
        loadedRoot = root
    }

    func transcribe(file: URL, language: String, root: URL) async throws -> String {
        guard ["en", "fi"].contains(language) else { throw VoiceError.message("Choose English or Finnish.") }
        let bytes = try file.resourceValues(forKeys: [.fileSizeKey]).fileSize ?? 0
        guard bytes > 0, bytes <= 20_000_000 else { throw VoiceError.message("Audio must be under 20 MB.") }
        let audio = try AVAudioFile(forReading: file)
        let duration = Double(audio.length) / audio.processingFormat.sampleRate
        guard duration.isFinite, duration > 0, duration <= 30.1 else {
            throw VoiceError.message("Record a prompt of at most 30 seconds.")
        }
        let samples = try AudioProcessor.loadAudioAsFloatArray(fromPath: file.path)
        let energy = samples.reduce(0.0) { $0 + Double($1 * $1) } / Double(max(1, samples.count))
        guard energy > 0.000001 else { throw VoiceError.message("No speech was detected. Try again closer to the microphone.") }
        try await load(root: root)
        guard let pipeline else { throw VoiceError.message("Speech model is unavailable.") }
        let options = DecodingOptions(task: .transcribe, language: language,
                                      temperatureFallbackCount: 0, skipSpecialTokens: true,
                                      withoutTimestamps: true)
        let results = try await pipeline.transcribe(audioArray: samples, decodeOptions: options)
        let text = results.map(\.text).joined(separator: " ").trimmingCharacters(in: .whitespacesAndNewlines)
        guard !text.isEmpty, text.utf8.count <= 4096 else { throw VoiceError.message("No usable transcript. Try a shorter prompt.") }
        return text
    }
}
