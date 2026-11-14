package com.orbiscreen.android.net

data class DiscoveredHost(
    val name: String,
    val host: String,
    val port: Int,
    val isRecent: Boolean = false,
)
