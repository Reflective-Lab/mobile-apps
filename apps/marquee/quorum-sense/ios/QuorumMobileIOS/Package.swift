// swift-tools-version: 6.0

import PackageDescription

let package = Package(
    name: "QuorumMobileIOS",
    platforms: [
        .iOS(.v18),
        .macOS(.v14)
    ],
    products: [
        .library(
            name: "QuorumMobileIOS",
            targets: ["QuorumMobileIOS"]
        )
    ],
    targets: [
        .target(
            name: "QuorumMobileIOS"
        )
    ]
)
