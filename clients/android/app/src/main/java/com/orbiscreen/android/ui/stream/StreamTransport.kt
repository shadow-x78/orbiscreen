// Orbiscreen - StreamTransport.kt (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen

package com.orbiscreen.android.ui.stream

/** Short label for the video path currently on screen. */
internal object StreamTransport {
    const val USB_ANNEX_B = "USB Annex-B"
    const val USB_HTTP_AU = "USB HTTP /au"
    const val UDP_ANNEX_B = "UDP Annex-B"
    const val MPEG_TS = "MPEG-TS"

    fun label(udp: Boolean, usbAoa: Boolean, usbHttp: Boolean, exo: Boolean): String = when {
        udp -> UDP_ANNEX_B
        usbAoa -> USB_ANNEX_B
        usbHttp -> USB_HTTP_AU
        exo -> MPEG_TS
        else -> ""
    }

    fun hostLabel(transport: String, host: String): String = when (transport) {
        USB_ANNEX_B, USB_HTTP_AU -> "USB · Orbiscreen"
        else -> host
    }

    fun statsTitle(base: String, transport: String): String =
        if (transport.isBlank()) base else "$base · $transport"
}
