// swift-tools-version: 5.10
// SPDX-License-Identifier: Apache-2.0
import PackageDescription

let package = Package(
    name: "PajaVoice",
    platforms: [.macOS("15.0")],
    products: [.executable(name: "PajaVoice", targets: ["PajaVoice"])],
    dependencies: [
        .package(url: "https://github.com/argmaxinc/argmax-oss-swift.git", exact: "1.1.1")
    ],
    targets: [
        .executableTarget(name: "PajaVoice", dependencies: [
            .product(name: "WhisperKit", package: "argmax-oss-swift")
        ]),
        .testTarget(name: "PajaVoiceTests", dependencies: ["PajaVoice"])
    ]
)
