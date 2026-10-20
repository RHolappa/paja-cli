// SPDX-License-Identifier: Apache-2.0
import AVFoundation
import XCTest
@testable import PajaVoice

final class VoiceTests: XCTestCase {
    func testPromptFileIsPrivateAndLiteral() throws {
        let text = "A Finnish goal: kehitä. $(touch never-run)"
        let file = try Coordinator.promptFile(text)
        defer { try? FileManager.default.removeItem(at: file.deletingLastPathComponent()) }
        XCTAssertEqual(try String(contentsOf: file, encoding: .utf8), text)
        let attributes = try FileManager.default.attributesOfItem(atPath: file.deletingLastPathComponent().path)
        XCTAssertEqual(attributes[.posixPermissions] as? Int, 0o700)
        XCTAssertThrowsError(try Coordinator.promptFile("  "))
        XCTAssertThrowsError(try Coordinator.promptFile(String(repeating: "ä", count: 4096)))
    }

    func testSilenceAndOverlongAudioNeverLoadAModel() async throws {
        let root = FileManager.default.temporaryDirectory.appendingPathComponent(UUID().uuidString)
        try FileManager.default.createDirectory(at: root, withIntermediateDirectories: true)
        defer { try? FileManager.default.removeItem(at: root) }
        let engine = SpeechEngine()
        for duration in [1, 31] {
            let file = root.appendingPathComponent("silence-\(duration).wav")
            let format = AVAudioFormat(standardFormatWithSampleRate: 16000, channels: 1)!
            let buffer = AVAudioPCMBuffer(pcmFormat: format, frameCapacity: AVAudioFrameCount(duration * 16000))!
            buffer.frameLength = buffer.frameCapacity
            buffer.floatChannelData![0].initialize(repeating: 0, count: Int(buffer.frameLength))
            do { let writer = try AVAudioFile(forWriting: file, settings: format.settings); try writer.write(from: buffer) }
            do {
                _ = try await engine.transcribe(file: file, language: "fi", root: root.appendingPathComponent("missing"))
                XCTFail("Invalid audio should not be transcribed")
            } catch {
                XCTAssertTrue(error.localizedDescription.contains(duration == 1 ? "No speech" : "30 seconds"))
            }
        }
    }
}
