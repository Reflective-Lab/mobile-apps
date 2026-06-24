plugins {
    id("com.android.application") version "9.2.1" apply false
    id("org.jetbrains.kotlin.android") version "2.0.21" apply false
    id("org.jetbrains.kotlin.plugin.compose") version "2.0.21" apply false
    // Auto-adds the Sentry Android SDK and uploads ProGuard/R8 mappings on release builds.
    id("io.sentry.android.gradle") version "6.12.0" apply false
}

