import CoreText
import SwiftUI

/// Reflective brand design tokens, mirrored from the Quorum Sense web app
/// (apps.reflective.se/quorum-sense). Colors are the site's CSS custom
/// properties; fonts are the bundled DM Sans / DM Serif Display / IBM Plex Mono
/// families, registered at launch via `Brand.registerFonts()`.
enum Brand {
    // MARK: Colors (light/"paper" palette)
    static let accent = Color(hex: 0x177E66)
    static let accentDark = Color(hex: 0x0F5C4A)
    static let accentSoft = Color(hex: 0x177E66, alpha: 0.12)

    static let ink = Color(hex: 0x101417)
    static let inkSoft = Color(hex: 0x4D5768)
    static let inkMuted = Color(hex: 0x65747A)

    static let paper = Color(hex: 0xF7F8F3)
    static let surface = Color(hex: 0xFFFFFF)
    static let surfaceMuted = Color(hex: 0xEEF7F4)
    static let line = Color(hex: 0x101417, alpha: 0.10)

    static let blue = Color(hex: 0x0B72B9)
    static let coral = Color(hex: 0xD94F3D)
    static let gold = Color(hex: 0x9D6B00)
    static let ok = Color(hex: 0x137333)
    static let warn = Color(hex: 0xAD5F00)
    static let danger = Color(hex: 0xB42318)

    // MARK: Fonts (PostScript names of the bundled TTFs)
    enum FontName {
        static let display = "DMSerifDisplay-Regular"
        static let displayItalic = "DMSerifDisplay-Italic"
        static let sans = "DMSans-Regular"
        static let sansMedium = "DMSans-Medium"
        static let sansBold = "DMSans-Bold"
        static let mono = "IBMPlexMono-Regular"
        static let monoMedium = "IBMPlexMono-Medium"
        static let monoSemibold = "IBMPlexMono-SemiBold"

        static let all = [
            display, displayItalic, sans, sansMedium, sansBold,
            mono, monoMedium, monoSemibold,
        ]
    }

    static func display(_ size: CGFloat) -> Font { .custom(FontName.display, size: size) }
    static func sans(_ size: CGFloat) -> Font { .custom(FontName.sans, size: size) }
    static func sansMedium(_ size: CGFloat) -> Font { .custom(FontName.sansMedium, size: size) }
    static func sansBold(_ size: CGFloat) -> Font { .custom(FontName.sansBold, size: size) }
    static func mono(_ size: CGFloat) -> Font { .custom(FontName.mono, size: size) }
    static func monoMedium(_ size: CGFloat) -> Font { .custom(FontName.monoMedium, size: size) }

    /// Registers the bundled font files so `Font.custom(...)` can resolve them.
    /// Idempotent: re-registering an already-registered font is a no-op.
    static func registerFonts() {
        for name in FontName.all {
            guard let url = Bundle.main.url(forResource: name, withExtension: "ttf") else { continue }
            CTFontManagerRegisterFontsForURL(url as CFURL, .process, nil)
        }
    }
}

extension Color {
    /// Hex literal initialiser, e.g. `Color(hex: 0x177E66)`.
    init(hex: UInt32, alpha: Double = 1.0) {
        self.init(
            .sRGB,
            red: Double((hex >> 16) & 0xFF) / 255.0,
            green: Double((hex >> 8) & 0xFF) / 255.0,
            blue: Double(hex & 0xFF) / 255.0,
            opacity: alpha
        )
    }
}
