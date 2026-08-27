# ─────────────────────────────────────────────
# Orbiscreen - Android ProGuard / R8 Rules
# https://github.com/shadow-x78/orbiscreen
# ─────────────────────────────────────────────

# ── Media3 ──
-dontwarn androidx.media3.**

# ── OkHttp + Okio ──
-dontwarn okhttp3.**
-dontwarn okio.**
-dontwarn org.conscrypt.**
-dontwarn org.bouncycastle.**
-dontwarn org.openjsse.**
