// SPDX-License-Identifier: Apache-2.0
import Foundation

struct LocalProject: Codable, Identifiable, Hashable {
    let id: String
    let name: String
}

enum Coordinator {
    static var executable: URL {
        Bundle.main.resourceURL!.appendingPathComponent("paja")
    }

    static func call(arguments: [String], directory: URL) async throws -> [String: Any] {
        let binary = executable
        return try await Task.detached {
            guard FileManager.default.isExecutableFile(atPath: binary.path) else {
                throw VoiceError.message("The bundled Paja CLI is missing. Rebuild with scripts/build-macos.sh.")
            }
            let process = Process()
            process.executableURL = binary
            process.arguments = ["-C", directory.path, "--json"] + arguments
            let output = Pipe()
            process.standardOutput = output
            process.standardError = FileHandle.nullDevice
            process.standardInput = FileHandle.nullDevice
            try process.run()
            let data = output.fileHandleForReading.readDataToEndOfFile()
            process.waitUntilExit()
            guard let response = try JSONSerialization.jsonObject(with: data) as? [String: Any] else {
                throw VoiceError.message("The coordinator returned an invalid response.")
            }
            guard process.terminationStatus == 0, response["ok"] as? Bool == true else {
                let message = (response["error"] as? [String: Any])?["message"] as? String
                throw VoiceError.message(message ?? "The coordinator could not complete this operation.")
            }
            return response
        }.value
    }

    static func promptFile(_ text: String) throws -> URL {
        guard !text.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty, text.utf8.count <= 4096 else {
            throw VoiceError.message("Enter a prompt of 1–4096 UTF-8 bytes.")
        }
        let directory = FileManager.default.temporaryDirectory.appendingPathComponent("paja-\(UUID().uuidString)")
        try FileManager.default.createDirectory(at: directory, withIntermediateDirectories: true,
                                               attributes: [.posixPermissions: 0o700])
        let file = directory.appendingPathComponent("prompt.txt")
        try Data(text.utf8).write(to: file)
        return file
    }

    static func summary(_ response: [String: Any]) -> String {
        let data = response["data"] as? [String: Any] ?? [:]
        var parts: [String] = []
        if let message = data["message"] as? String { parts.append(message) }
        if let project = response["project"] as? [String: Any], let revision = project["candidate_revision"] as? Int {
            parts.append("Version \(revision) · \(project["state"] as? String ?? "")")
        }
        if let report = data["report"] as? [String: Any], let counts = report["counts"] as? [String: Int] {
            parts.append("\(counts["passed"] ?? 0) passed · \(counts["failed"] ?? 0) failed · \(counts["blocked"] ?? 0) blocked")
        }
        if let execution = data["execution"] as? [String: Any],
           let output = execution["output"] as? [String: Any], let total = output["total_cents"] as? Int {
            parts.append(String(format: "Selected lunches total €%.2f", Double(total) / 100))
        }
        return parts.joined(separator: "\n\n")
    }
}
