plugins {
    id("com.android.application")
    id("org.jetbrains.kotlin.android")
    id("org.jetbrains.kotlin.plugin.compose")
    id("io.sentry.android.gradle")
}

android {
    namespace = "se.reflective.quorum"
    compileSdk = 35

    defaultConfig {
        applicationId = "se.reflective.quorum"
        minSdk = 28
        targetSdk = 35
        versionCode = 1
        versionName = "0.1.0"
        testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"
    }

    testOptions {
        // Kotest + JUnit5 run the local (JVM) unit tests.
        unitTests.all { it.useJUnitPlatform() }
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    kotlinOptions {
        jvmTarget = "17"
    }

    buildFeatures {
        compose = true
    }
}

dependencies {
    implementation(platform("androidx.compose:compose-bom:2025.01.00"))
    implementation("androidx.activity:activity-compose:1.9.3")
    implementation("androidx.compose.foundation:foundation")
    implementation("androidx.compose.material3:material3")
    implementation("androidx.compose.ui:ui")
    implementation("androidx.compose.ui:ui-tooling-preview")
    implementation("androidx.lifecycle:lifecycle-runtime-ktx:2.8.7")
    implementation("org.jetbrains.kotlinx:kotlinx-coroutines-android:1.9.0")
    debugImplementation("androidx.compose.ui:ui-tooling")

    // UniFFI Kotlin bindings depend on JNA at runtime. The @aar variant bundles
    // the native JNA libs Android needs.
    implementation("net.java.dev.jna:jna:5.14.0@aar")

    // --- Local (JVM) unit tests: Kotest runner + assertions + property,
    // MockK, Turbine, coroutines-test. ---
    testImplementation("io.kotest:kotest-runner-junit5:5.9.1")
    testImplementation("io.kotest:kotest-assertions-core:5.9.1")
    testImplementation("io.kotest:kotest-property:5.9.1")
    testImplementation("io.mockk:mockk:1.13.13")
    testImplementation("app.cash.turbine:turbine:1.1.0")
    testImplementation("org.jetbrains.kotlinx:kotlinx-coroutines-test:1.9.0")

    // --- Instrumented tests: Compose UI testing. ---
    androidTestImplementation("androidx.test.ext:junit:1.2.1")
    androidTestImplementation(platform("androidx.compose:compose-bom:2025.01.00"))
    androidTestImplementation("androidx.compose.ui:ui-test-junit4")
    debugImplementation("androidx.compose.ui:ui-test-manifest")
}

// Sentry Android Gradle plugin. The SDK itself is auto-added; the DSN is declared
// in AndroidManifest.xml (io.sentry.dsn). This block governs build-time symbol
// upload. The auth token is read from the SENTRY_AUTH_TOKEN env var (CI) or
// app/sentry.properties (local, gitignored) — never hardcode it here.
//
// Uploads are gated behind -PsentryUpload so ordinary debug builds/tests never
// attempt an upload (which would fail without the token). Only the dedicated
// release CI job passes the flag: ./gradlew :app:assembleRelease -PsentryUpload.
val sentryUploads = hasProperty("sentryUpload")
sentry {
    // Org is EU-region; uploads must target the .de endpoint.
    url.set("https://de.sentry.io/")
    org.set("reflective-labs-xa")
    projectName.set("android")

    // R8/ProGuard mappings — only meaningful once release minification is enabled
    // (no buildTypes { release { isMinifyEnabled = true } } yet, so currently a no-op).
    includeProguardMapping.set(sentryUploads)
    autoUploadProguardMapping.set(sentryUploads)

    // Native (Rust/UniFFI .so) debug symbols so native crashes symbolicate.
    uploadNativeSymbols.set(sentryUploads)
    includeNativeSources.set(sentryUploads)

    // Source-context: shows the offending source lines in Sentry.
    includeSourceContext.set(sentryUploads)
    autoUploadSourceContext.set(sentryUploads)
}

