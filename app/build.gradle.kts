plugins {
    id("com.android.application")
    id("org.jetbrains.kotlin.android")
    id("org.mozilla.rust-android-gradle.rust-android")
}

android {
    namespace = "com.styk.animate"
    compileSdk = 36
    ndkVersion = "26.1.10909125"

    defaultConfig {
        applicationId = "com.styk.animate"
        minSdk = 26
        targetSdk = 36
        versionCode = 125
        versionName = "1.2.5"

        ndk {
            abiFilters += listOf("arm64", "arm", "x86", "x86_64")
        }
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

cargo {
    module = "../native/rust/styk_core"
    libname = "styk_core"
    targets = listOf("arm64", "arm", "x86", "x86_64")
    profile = "release"
    extraCargoBuildArguments = listOf("--locked")
}

// Ensure cargoBuild runs before Kotlin compilation
tasks.whenTaskAdded {
    if (name == "javaPreCompileDebug" || name == "javaPreCompileRelease") {
        dependsOn("cargoBuild")
    }
}

dependencies {
    implementation(platform("androidx.compose:compose-bom:2024.12.01"))
    implementation("androidx.compose.ui:ui")
    implementation("androidx.compose.material3:material3")
    implementation("androidx.compose.foundation:foundation")
    implementation("androidx.activity:activity-compose:1.9.3")
}
