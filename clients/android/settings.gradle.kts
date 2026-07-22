// Orbiscreen — Android Gradle settings (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen

pluginManagement {
    repositories {
        google()
        mavenCentral()
        gradlePluginPortal()
    }
}

dependencyResolutionManagement {
    repositoriesMode.set(RepositoriesMode.FAIL_ON_PROJECT_REPOS)
    repositories {
        google()
        mavenCentral()
    }
}

rootProject.name = "Orbiscreen"
include(":app")