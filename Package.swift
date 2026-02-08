// swift-tools-version:5.9
import PackageDescription

let package = Package(
    name: "Speech",
    platforms: [
        .macOS(.v14)
    ],
    products: [
        .executable(name: "Speech", targets: ["Speech"])
    ],
    dependencies: [
        .package(url: "https://github.com/soffes/HotKey.git", from: "0.2.0"),
        .package(url: "https://github.com/argmaxinc/WhisperKit.git", from: "0.15.0")
    ],
    targets: [
        .executableTarget(
            name: "Speech",
            dependencies: [
                "HotKey",
                "WhisperKit"
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
