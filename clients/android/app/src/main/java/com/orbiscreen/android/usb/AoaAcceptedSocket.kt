// Orbiscreen - AoaAcceptedSocket.kt (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen

package com.orbiscreen.android.usb

import java.net.Socket

/**
 * Reads one accepted AOA proxy socket. Closing the socket, including from
 * [UsbAccessoryManager.stopAccessory], must return instead of throwing:
 * an exception here is an uncaught coroutine failure and Android kills the process.
 */
internal object AoaAcceptedSocket {
    fun readLoop(socket: Socket, running: () -> Boolean = { true }, onChunk: (ByteArray, Int) -> Unit) {
        val input = try {
            if (socket.isClosed) return
            socket.getInputStream()
        } catch (_: Exception) {
            return
        }
        val buf = ByteArray(16384)
        try {
            while (running() && !socket.isClosed) {
                val n = input.read(buf)
                if (n <= 0) break
                onChunk(buf, n)
            }
        } catch (_: Exception) {
        }
    }
}
