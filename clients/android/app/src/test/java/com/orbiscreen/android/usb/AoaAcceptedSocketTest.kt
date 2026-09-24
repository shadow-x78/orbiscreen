// Orbiscreen - AoaAcceptedSocketTest.kt (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen

package com.orbiscreen.android.usb

import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test
import java.net.InetAddress
import java.net.ServerSocket
import java.net.Socket
import java.util.concurrent.CountDownLatch
import java.util.concurrent.TimeUnit
import java.util.concurrent.atomic.AtomicReference

class AoaAcceptedSocketTest {

    @Test
    fun closedSocketDoesNotThrow() {
        val (accepted, client) = connected()
        try {
            accepted.close()
            AoaAcceptedSocket.readLoop(accepted) { _, _ ->
                error("closed socket must not deliver bytes")
            }
        } finally {
            client.close()
        }
    }

    @Test
    fun closeDuringReadDoesNotThrow() {
        val (accepted, client) = connected()
        val failure = AtomicReference<Throwable>()
        val started = CountDownLatch(1)
        val thread = Thread {
            try {
                started.countDown()
                AoaAcceptedSocket.readLoop(accepted) { _, _ -> }
            } catch (t: Throwable) {
                failure.set(t)
            }
        }
        thread.start()
        assertTrue(started.await(2, TimeUnit.SECONDS))
        Thread.sleep(50)
        accepted.close()
        thread.join(2000)
        client.close()
        assertTrue("read thread still running", !thread.isAlive)
        assertEquals(null, failure.get())
    }

    @Test
    fun deliversBytesThenStopsWhenPeerCloses() {
        val (accepted, client) = connected()
        val got = ArrayList<Byte>()
        val done = CountDownLatch(1)
        Thread {
            AoaAcceptedSocket.readLoop(accepted) { buf, n ->
                for (i in 0 until n) got.add(buf[i])
            }
            done.countDown()
        }.start()
        client.getOutputStream().write(byteArrayOf(1, 2, 3, 4))
        client.shutdownOutput()
        assertTrue(done.await(2, TimeUnit.SECONDS))
        assertEquals(listOf<Byte>(1, 2, 3, 4), got)
        accepted.close()
        client.close()
    }

    private fun connected(): Pair<Socket, Socket> {
        val server = ServerSocket(0, 1, InetAddress.getByName("127.0.0.1"))
        val client = Socket("127.0.0.1", server.localPort)
        val accepted = server.accept()
        server.close()
        return accepted to client
    }
}
