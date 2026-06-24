plugins {
    id("com.android.application") version "9.2.1" apply false
    // AGP 9 has built-in Kotlin support — the standalone org.jetbrains.kotlin.android
    // plugin is removed. The Kotlin version is driven by the compose compiler plugin
    // (and the kotlin-build-tools AGP resolves), kept in lockstep at 2.4.0.
    id("org.jetbrains.kotlin.plugin.compose") version "2.4.0" apply false
    // Auto-adds the Sentry Android SDK and uploads ProGuard/R8 mappings on release builds.
    id("io.sentry.android.gradle") version "6.12.0" apply false
}

