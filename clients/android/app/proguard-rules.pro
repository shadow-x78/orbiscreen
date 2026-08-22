# ─────────────────────────────────────────────
# Orbiscreen - Android ProGuard / R8 Rules
# https://github.com/shadow-x78/orbiscreen
# ─────────────────────────────────────────────

# ── App ──
-keep class com.orbiscreen.android.** { *; }

# ── Media3 ──
-keep class androidx.media3.** { *; }
-keep interface androidx.media3.** { *; }
-keep class * implements androidx.media3.common.** { *; }
-keep class * implements androidx.media3.exoplayer.** { *; }
-dontwarn androidx.media3.**

# ── OkHttp + Okio ──
-dontwarn okhttp3.**
-dontwarn okio.**
-dontwarn org.conscrypt.**
-dontwarn org.bouncycastle.**
-dontwarn org.openjsse.**

# ── Compose ──
-keep class kotlin.Metadata { *; }
-keepclassmembers class * {
    @androidx.compose.runtime.Composable <methods>;
}

# ── JSON ──
-keepclassmembers class * {
    @org.json.* <fields>;
}

# ── NSD + Wifi ──
-keep class android.net.nsd.** { *; }
-keep class android.net.wifi.** { *; }
