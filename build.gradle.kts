buildscript {
    dependencies {
        classpath("org.mozilla.rust-android-gradle:plugin:0.9.3")
    }
}

plugins {
    id("com.android.application") version "8.7.3" apply false
    id("org.jetbrains.kotlin.android") version "2.0.21" apply false
}
