plugins {
    id("com.android.application")
    id("org.jetbrains.kotlin.android")
    id("org.jetbrains.kotlin.plugin.compose")
}

// Per-app identity. Override from the CLI or scripts/app-config.sh:
//   ./gradlew :app:assembleDebug -PappSlug=atlas -PappName=Atlas
// `namespace` stays constant (shared shell code package); only the installed
// applicationId + display name vary across the fleet.
val appSlug = (project.findProperty("appSlug") as String?) ?: "quorum"
val appName = (project.findProperty("appName") as String?) ?: "Quorum"

android {
    namespace = "se.reflective.shell"
    compileSdk = 35

    defaultConfig {
        applicationId = "se.reflective.$appSlug"
        minSdk = 28
        targetSdk = 35
        versionCode = 1
        versionName = "0.1.0"
        resValue("string", "app_name", appName)
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
    debugImplementation("androidx.compose.ui:ui-tooling")

    // UniFFI Kotlin bindings depend on JNA at runtime. The @aar variant
    // bundles the native JNA libs Android needs.
    implementation("net.java.dev.jna:jna:5.14.0@aar")
}
