// Orbiscreen - Android build (GPL-3.0-or-later)
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
        versionCode = 2
        versionName = "0.6.0"
    }

    signingConfigs {
        create("release") {
            val ksFile = file("orbiscreen-release.keystore")
            if (ksFile.exists()) {
                storeFile = ksFile
                storePassword = "orbiscreen123"
                keyAlias = "orbiscreen"
                keyPassword = "orbiscreen123"
                v1SigningEnabled = true
                v2SigningEnabled = true
                v3SigningEnabled = true
            } else {
                storeFile = signingConfigs.getByName("debug").storeFile
                storePassword = signingConfigs.getByName("debug").storePassword
                keyAlias = signingConfigs.getByName("debug").keyAlias
                keyPassword = signingConfigs.getByName("debug").keyPassword
            }
        }
    }

    buildTypes {
        release {
            isMinifyEnabled = false
            signingConfig = signingConfigs.getByName("release")
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

    sourceSets["main"].assets.srcDir(layout.buildDirectory.dir("generated/assets"))
}

tasks.register<Copy>("syncWebClient") {
    from("$rootDir/../../clients/web")
    into(layout.buildDirectory.dir("generated/assets/client"))
}

tasks.named("preBuild") {
    dependsOn("syncWebClient")
}

dependencies {
    implementation("androidx.core:core-ktx:1.13.1")
    implementation("androidx.appcompat:appcompat:1.7.0")
    implementation("androidx.webkit:webkit:1.11.0")
}