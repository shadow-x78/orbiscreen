# Orbiscreen Android - ProGuard / R8 rules.
# Keep Media3 / ExoPlayer codecs and HTTP stack; everything else can be shrunk.

-keep class com.orbiscreen.android.** { *; }

# Media3 / ExoPlayer - codecs, sources, and listeners use reflection
-keep class androidx.media3.** { *; }
-keep interface androidx.media3.** { *; }
-keep class * implements androidx.media3.common.** { *; }
-keep class * implements androidx.media3.exoplayer.** { *; }
-dontwarn androidx.media3.**

# OkHttp + Okio
-dontwarn okhttp3.**
-dontwarn okio.**
-dontwarn org.conscrypt.**
-dontwarn org.bouncycastle.**
-dontwarn org.openjsse.**

# Compose
-keep class kotlin.Metadata { *; }
-keepclassmembers class * {
    @androidx.compose.runtime.Composable <methods>;
}

# Keep JSON field names so the Rust host sees the same payload
-keepclassmembers class * {
    @org.json.* <fields>;
}

# NSD / Wifi
-keep class android.net.nsd.** { *; }
-keep class android.net.wifi.** { *; }