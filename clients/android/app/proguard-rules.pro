# Orbiscreen Android - ProGuard / R8 rules.
# Keep WebView and WebRTC-related classes; everything else can be shrunk.
-keep class com.orbiscreen.android.** { *; }
-keepclassmembers class * extends android.webkit.WebViewClient { *; }
-keepclassmembers class * extends android.webkit.WebChromeClient { *; }
