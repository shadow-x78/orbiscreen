// ─────────────────────────────────────────────
// Orbiscreen - Android App Build (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen
// ─────────────────────────────────────────────

plugins {
    id("com.android.application")
    id("org.jetbrains.kotlin.android")
    id("org.jetbrains.kotlin.plugin.compose")
}

android {
    namespace = "com.orbiscreen.android"
    compileSdk = 35

    defaultConfig {
        applicationId = "com.orbiscreen.android"
        minSdk = 26
        targetSdk = 35
        versionCode = 66
        versionName = "0.23.6"
    }

    signingConfigs {
        create("release") {
            val ksPath: String? =
                System.getenv("ORBISCREEN_KEYSTORE_PATH")
                    ?: (project.findProperty("orbiscreen.keystorePath") as String?)
            val storePw: String? =
                System.getenv("ORBISCREEN_STORE_PASSWORD")
                    ?: (project.findProperty("orbiscreen.storePassword") as String?)
            val alias: String? =
                System.getenv("ORBISCREEN_KEY_ALIAS")
                    ?: (project.findProperty("orbiscreen.keyAlias") as String?)
            val keyPw: String? =
                System.getenv("ORBISCREEN_KEY_PASSWORD")
                    ?: (project.findProperty("orbiscreen.keyPassword") as String?)
            val ksFile = ksPath?.let { file(it) }
            if (ksFile != null && ksFile.exists() &&
                !storePw.isNullOrBlank() && !alias.isNullOrBlank() && !keyPw.isNullOrBlank()
            ) {
                storeFile = ksFile
                storePassword = storePw
                keyAlias = alias
                keyPassword = keyPw
                enableV1Signing = false
                enableV2Signing = true
                enableV3Signing = true
            } else {
                logger.warn(
                    "No signing keystore configured - release APKs will be UNSIGNED. " +
                        "Set ORBISCREEN_KEYSTORE_PATH plus ORBISCREEN_STORE_PASSWORD, " +
                        "ORBISCREEN_KEY_ALIAS and ORBISCREEN_KEY_PASSWORD to sign."
                )
            }
        }
    }

    buildTypes {
        release {
            isMinifyEnabled = true
            isShrinkResources = true
            val releaseSigning = signingConfigs.getByName("release")
            signingConfig = if (releaseSigning.storeFile != null) {
                releaseSigning
            } else {
                null
            }
            proguardFiles(
                getDefaultProguardFile("proguard-android-optimize.txt"),
                "proguard-rules.pro",
            )
        }
        debug {
            isMinifyEnabled = false
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
        buildConfig = true
    }
}

dependencies {
    implementation("androidx.core:core-ktx:1.13.1")
    implementation("androidx.activity:activity-compose:1.9.2")
    implementation("androidx.lifecycle:lifecycle-runtime-ktx:2.8.6")
    implementation("androidx.lifecycle:lifecycle-viewmodel-compose:2.8.6")
    implementation("androidx.core:core-splashscreen:1.0.1")

    implementation(platform("androidx.compose:compose-bom:2024.09.03"))
    implementation("androidx.compose.ui:ui")
    implementation("androidx.compose.ui:ui-graphics")
    implementation("androidx.compose.material3:material3")
    implementation("androidx.compose.material:material-icons-extended")
    implementation("androidx.compose.runtime:runtime")

    implementation("androidx.navigation:navigation-compose:2.8.2")

    implementation("androidx.media3:media3-exoplayer:1.3.1")
    implementation("androidx.media3:media3-ui:1.3.1")
    implementation("androidx.media3:media3-datasource-okhttp:1.3.1")

    implementation("com.squareup.okhttp3:okhttp:4.12.0")

    implementation("org.jetbrains.kotlinx:kotlinx-coroutines-android:1.8.1")

    testImplementation("junit:junit:4.13.2")
}
