// Orbiscreen — Android app Gradle build script (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen

import org.gradle.api.tasks.Copy

plugins {
    id("com.android.application")
    id("org.jetbrains.kotlin.android")
}

android {
    namespace = "com.orbiscreen.android"
    compileSdk = 34

    defaultConfig {
        applicationId = "com.orbiscreen.android"
        minSdk = 26
        targetSdk = 34
        versionCode = 1
        versionName = "0.1.0"
    }

    buildTypes {
        release {
            isMinifyEnabled = false
            proguardFiles(
                getDefaultProguardFile("proguard-android-optimize.txt"),
                "proguard-rules.pro",
            )
        }
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    kotlinOptions {
        jvmTarget = "17"
    }

    sourceSets["main"].assets.srcDirs("$rootDir/../../clients/web")
}

tasks.register<Copy>("syncWebClient") {
    from("$rootDir/../../clients/web")
    into("$buildDir/generated/assets/client")
}

tasks.named("preBuild") { dependsOn("syncWebClient") }
android.sourceSets["main"].assets.srcDir("$buildDir/generated/assets/client")
android.sourceSets["main"].assets.srcDirs("$rootDir/../../clients/web")

dependencies {
    implementation("androidx.core:core-ktx:1.13.1")
    implementation("androidx.appcompat:appcompat:1.7.0")
    implementation("androidx.webkit:webkit:1.11.0")
}