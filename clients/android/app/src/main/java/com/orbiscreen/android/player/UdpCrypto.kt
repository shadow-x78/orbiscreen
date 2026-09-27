// Orbiscreen - UdpCrypto.kt (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen

package com.orbiscreen.android.player

import java.security.GeneralSecurityException
import javax.crypto.Cipher
import javax.crypto.spec.GCMParameterSpec
import javax.crypto.spec.SecretKeySpec

/** AES-256-GCM datagrams. Clear header is ORB2 || key id || nonce. Plaintext is ORB1. */
object UdpCrypto {
    const val DIR_CLIENT = 0
    const val DIR_HOST = 1
    const val HEADER = 32
    const val OVERHEAD = 48

    fun nonce(direction: Int, counter: Long): ByteArray {
        val out = ByteArray(12)
        out[0] = direction.toByte()
        var value = counter
        for (i in 1 until 9) {
            out[i] = (value and 0xff).toByte()
            value = value ushr 8
        }
        return out
    }

    fun peerCounter(nonce: ByteArray, expectedDirection: Int): Long? {
        if (nonce.size != 12 || nonce[0] != expectedDirection.toByte()) return null
        if (nonce[9] != 0.toByte() || nonce[10] != 0.toByte() || nonce[11] != 0.toByte()) return null
        var value = 0L
        for (i in 1 until 9) {
            value = value or ((nonce[i].toLong() and 0xff) shl (8 * (i - 1)))
        }
        return value.takeIf { it != 0L }
    }

    fun seal(keyId: ByteArray, secret: ByteArray, nonce: ByteArray, plain: ByteArray): ByteArray {
        val header = ByteArray(HEADER)
        header[0] = 'O'.code.toByte()
        header[1] = 'R'.code.toByte()
        header[2] = 'B'.code.toByte()
        header[3] = '2'.code.toByte()
        System.arraycopy(keyId, 0, header, 4, 16)
        System.arraycopy(nonce, 0, header, 20, 12)
        val cipher = Cipher.getInstance("AES/GCM/NoPadding")
        cipher.init(
            Cipher.ENCRYPT_MODE,
            SecretKeySpec(secret, "AES"),
            GCMParameterSpec(128, nonce),
        )
        cipher.updateAAD(header)
        return header + cipher.doFinal(plain)
    }

    fun open(secret: ByteArray, datagram: ByteArray): ByteArray? {
        if (datagram.size < OVERHEAD ||
            datagram[0] != 'O'.code.toByte() ||
            datagram[1] != 'R'.code.toByte() ||
            datagram[2] != 'B'.code.toByte() ||
            datagram[3] != '2'.code.toByte()
        ) {
            return null
        }
        return try {
            val header = datagram.copyOfRange(0, HEADER)
            val nonce = datagram.copyOfRange(20, 32)
            val body = datagram.copyOfRange(HEADER, datagram.size)
            val cipher = Cipher.getInstance("AES/GCM/NoPadding")
            cipher.init(
                Cipher.DECRYPT_MODE,
                SecretKeySpec(secret, "AES"),
                GCMParameterSpec(128, nonce),
            )
            cipher.updateAAD(header)
            cipher.doFinal(body)
        } catch (_: GeneralSecurityException) {
            null
        }
    }

    fun helloPlain(keyId: ByteArray, session: String?): ByteArray {
        val token = keyId.joinToString("") { "%02x".format(it) }.toByteArray(Charsets.US_ASCII)
        val extra = if (session.isNullOrBlank()) {
            ByteArray(0)
        } else {
            byteArrayOf(0) + session.toByteArray(Charsets.UTF_8)
        }
        return byteArrayOf(
            'O'.code.toByte(),
            'R'.code.toByte(),
            'B'.code.toByte(),
            '1'.code.toByte(),
            2,
        ) + token + extra
    }
}

/** Client or host side of one issued key. Nonces count up; replays inside the window are dropped. */
class UdpSessionCrypto(
    private val keyId: ByteArray,
    private val secret: ByteArray,
    private val localDirection: Int,
) {
    private val sendCounter = java.util.concurrent.atomic.AtomicLong(0)
    private val replay = UdpReplayWindow()
    private val peerDirection = if (localDirection == UdpCrypto.DIR_CLIENT) {
        UdpCrypto.DIR_HOST
    } else {
        UdpCrypto.DIR_CLIENT
    }

    fun sealNext(plain: ByteArray): ByteArray {
        val counter = sendCounter.incrementAndGet()
        return UdpCrypto.seal(keyId, secret, UdpCrypto.nonce(localDirection, counter), plain)
    }

    @Synchronized
    fun openPeer(datagram: ByteArray): ByteArray? {
        val plain = UdpCrypto.open(secret, datagram) ?: return null
        if (datagram.size < UdpCrypto.HEADER) return null
        val nonce = datagram.copyOfRange(20, 32)
        val counter = UdpCrypto.peerCounter(nonce, peerDirection) ?: return null
        if (!replay.accept(counter)) return null
        return plain
    }
}

class UdpReplayWindow {
    private var highest = 0L
    private val bits = LongArray(4)

    fun accept(counter: Long): Boolean {
        if (counter == 0L) return false
        if (highest == 0L) {
            highest = counter
            bits.fill(0)
            setBit(bits, 0L)
            return true
        }
        if (counter > highest) {
            val shift = counter - highest
            if (shift >= WINDOW) bits.fill(0) else shiftBits(shift)
            highest = counter
            setBit(bits, 0L)
            return true
        }
        val age = highest - counter
        if (age >= WINDOW || bitSet(bits, age)) return false
        setBit(bits, age)
        return true
    }

    private fun shiftBits(shift: Long) {
        val next = LongArray(4)
        for (age in 0L until WINDOW) {
            if (!bitSet(bits, age)) continue
            val moved = age + shift
            if (moved < WINDOW) setBit(next, moved)
        }
        next.copyInto(bits)
    }

    private companion object {
        const val WINDOW = 256L

        fun setBit(bits: LongArray, age: Long) {
            val word = (age / 64).toInt()
            val bit = (age % 64).toInt()
            bits[word] = bits[word] or (1L shl bit)
        }

        fun bitSet(bits: LongArray, age: Long): Boolean {
            val word = (age / 64).toInt()
            val bit = (age % 64).toInt()
            return bits[word] and (1L shl bit) != 0L
        }
    }
}

object UdpInbound {
    fun probeAckRecv(wireLength: Int, plain: ByteArray): Int? {
        if (plain.size < 7 || plain[4] != 7.toByte()) return null
        return wireLength
    }
}

/** LAN video target. HTTP stays on the pinned proxy; datagrams go to [host]. */
data class UdpVideoTarget(
    val host: String,
    val port: Int,
    val keyId: ByteArray,
    val secret: ByteArray,
    val sessionId: String,
    val httpHost: String,
    val httpPort: Int,
) {
    fun usable(): Boolean =
        keyId.size == 16 &&
            secret.size == 32 &&
            port in 1..65535 &&
            !com.orbiscreen.android.net.UsbLoopback.isLoopback(host)
}
