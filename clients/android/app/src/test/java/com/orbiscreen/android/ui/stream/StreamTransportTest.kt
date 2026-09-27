// Orbiscreen - StreamTransportTest.kt (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen

package com.orbiscreen.android.ui.stream

import org.junit.Assert.assertEquals
import org.junit.Test

class StreamTransportTest {

    @Test
    fun labelsTheActivePlayer() {
        assertEquals(StreamTransport.USB_ANNEX_B, StreamTransport.label(udp = false, usbAoa = true, usbHttp = false, exo = false))
        assertEquals(StreamTransport.USB_HTTP_AU, StreamTransport.label(udp = false, usbAoa = false, usbHttp = true, exo = false))
        assertEquals(StreamTransport.UDP_ANNEX_B, StreamTransport.label(udp = true, usbAoa = false, usbHttp = false, exo = false))
        assertEquals(StreamTransport.MPEG_TS, StreamTransport.label(udp = false, usbAoa = false, usbHttp = false, exo = true))
        assertEquals("", StreamTransport.label(udp = false, usbAoa = false, usbHttp = false, exo = false))
    }

    @Test
    fun loopbackLanProxyIsNotLabeledUsb() {
        val transport = StreamTransport.MPEG_TS
        assertEquals("127.0.0.1", StreamTransport.hostLabel(transport, "127.0.0.1"))
        assertEquals("USB · Orbiscreen", StreamTransport.hostLabel(StreamTransport.USB_ANNEX_B, "127.0.0.1"))
    }

    @Test
    fun statsTitleIncludesTransport() {
        assertEquals("Statistics", StreamTransport.statsTitle("Statistics", ""))
        assertEquals("Statistics · USB Annex-B", StreamTransport.statsTitle("Statistics", StreamTransport.USB_ANNEX_B))
    }
}
