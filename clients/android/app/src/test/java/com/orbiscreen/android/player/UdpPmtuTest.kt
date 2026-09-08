// Orbiscreen - UdpPmtuTest.kt (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen
package com.orbiscreen.android.player

import org.junit.Assert.assertEquals
import org.junit.Test
import java.net.DatagramPacket
import java.net.DatagramSocket
import java.net.InetAddress

class UdpPmtuTest {

    @Test
    fun probeAckEncodesReceivedDatagramSize() {
        val ack = UdpPlayer.encodeProbeAck(9, 900)
        assertEquals(9, ack.size)
        assertEquals('O'.code.toByte(), ack[0])
        assertEquals('R'.code.toByte(), ack[1])
        assertEquals('B'.code.toByte(), ack[2])
        assertEquals('1'.code.toByte(), ack[3])
        assertEquals(8.toByte(), ack[4])
        assertEquals(9, UdpPlayer.le16(ack, 5))
        assertEquals(900, UdpPlayer.le16(ack, 7))
    }

    @Test
    fun handshakeAcceptsAckAndKeepsWaitingOnProbe() {
        val ack = byteArrayOf(
            'O'.code.toByte(), 'R'.code.toByte(), 'B'.code.toByte(), '1'.code.toByte(), 3,
        )
        val probe = UdpPlayer.encodeCtrl(7)
        val junk = byteArrayOf(1, 2, 3)
        assertEquals(HandshakeAction.Ack, UdpPlayer.handshakeAction(ack))
        assertEquals(HandshakeAction.Control, UdpPlayer.handshakeAction(probe))
        assertEquals(HandshakeAction.Ignore, UdpPlayer.handshakeAction(junk))
    }

    @Test
    fun putLe16Roundtrip() {
        val buf = ByteArray(4)
        UdpPlayer.putLe16(buf, 0, 0x1234)
        UdpPlayer.putLe16(buf, 2, 1472)
        assertEquals(0x1234, UdpPlayer.le16(buf, 0))
        assertEquals(1472, UdpPlayer.le16(buf, 2))
    }

    @Test
    fun reusedPacketReportsFullSizeAfterPrepareReceive() {
        val server = DatagramSocket()
        val client = DatagramSocket()
        try {
            val buf = ByteArray(2048)
            val pkt = DatagramPacket(buf, buf.size)
            val addr = InetAddress.getByName("127.0.0.1")
            server.soTimeout = 2000
            client.send(DatagramPacket(ByteArray(21), 21, addr, server.localPort))
            server.receive(pkt)
            assertEquals(21, pkt.length)
            client.send(DatagramPacket(ByteArray(1400), 1400, addr, server.localPort))
            UdpPlayer.prepareReceive(pkt, buf)
            server.receive(pkt)
            assertEquals(1400, pkt.length)
        } finally {
            server.close()
            client.close()
        }
    }
}
