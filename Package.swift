// swift-tools-version:5.9
import PackageDescription

let package = Package(
    name: "Speech",
    platforms: [
        .macOS(.v13)
    ],
    products: [
        .executable(name: "Speech", targets: ["Speech"])
    ],
    dependencies: [
        .package(url: "https://github.com/soffes/HotKey.git", from: "0.2.0"),
        .package(url: "https://github.com/exPHAT/SwiftWhisper.git", branch: "master")
    ],
    targets: [
        .executableTarget(
            name: "Speech",
            dependencies: [
                "HotKey",
                "SwiftWhisper"
            ],
            path: "Sources",
            resources: [
                .process("Resources/Assets.xcassets"),
                .copy("Resources/AppIcon.icns")
            ],
            swiftSettings: [
                .unsafeFlags(["-parse-as-library"])
            ]
        )
    ]
)
