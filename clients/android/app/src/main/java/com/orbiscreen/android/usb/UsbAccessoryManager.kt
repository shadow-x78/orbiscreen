// Orbiscreen - UsbAccessoryManager.kt (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen

package com.orbiscreen.android.usb

import android.content.Context
import android.hardware.usb.UsbAccessory
import android.hardware.usb.UsbManager
import android.os.ParcelFileDescriptor
import android.util.Log
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import java.io.FileInputStream
import java.io.FileOutputStream
import java.io.InputStream
import java.io.OutputStream
import java.net.InetAddress
import java.net.ServerSocket
import java.net.Socket
import java.util.concurrent.ConcurrentHashMap
import java.util.concurrent.atomic.AtomicBoolean
import java.util.concurrent.atomic.AtomicInteger

object UsbAccessoryManager {
    private const val TAG = "UsbAccessoryManager"
    private const val FRAME_FLAG_DATA: Byte = 0x01
    private const val FRAME_FLAG_OPEN: Byte = 0x02
    private const val FRAME_FLAG_CLOSE: Byte = 0x04
    private const val FRAME_HEADER_LEN = 5
    private const val MAX_PAYLOAD_LEN = 16384

    private val scope = CoroutineScope(SupervisorJob() + Dispatchers.IO)
    private val _isAoaActive = MutableStateFlow(false)
    val isAoaActiveFlow: StateFlow<Boolean> = _isAoaActive.asStateFlow()
    val isAoaActive: Boolean get() = _isAoaActive.value

    private val _localProxyPort = MutableStateFlow(8789)
    val localProxyPort: Int get() = _localProxyPort.value

    private var activePfd: ParcelFileDescriptor? = null
    private var activeServer: ServerSocket? = null
    private val isRunning = AtomicBoolean(false)
    private val nextStreamId = AtomicInteger(1)
    private val activeStreams = ConcurrentHashMap<Short, Socket>()

    fun init(context: Context) {
        val usbManager = context.getSystemService(Context.USB_SERVICE) as? UsbManager ?: return
        val accessories = usbManager.accessoryList ?: return
        for (acc in accessories) {
            if (acc.manufacturer == "shadow-x78" || acc.model == "Orbiscreen") {
                startAccessory(context, acc)
                break
            }
        }
    }

    fun onAccessoryAttached(context: Context, accessory: UsbAccessory) {
        startAccessory(context, accessory)
    }

    fun onAccessoryDetached() {
        stopAccessory()
    }

    @Synchronized
    fun startAccessory(context: Context, accessory: UsbAccessory) {
        if (isRunning.get()) return
        val usbManager = context.getSystemService(Context.USB_SERVICE) as? UsbManager ?: return
        val pfd = try {
            usbManager.openAccessory(accessory)
        } catch (e: Exception) {
            Log.w(TAG, "openAccessory failed: ${e.message}")
            null
        } ?: return

        activePfd = pfd
        isRunning.set(true)

        val server = try {
            ServerSocket(0, 50, InetAddress.getByName("127.0.0.1"))
        } catch (e: Exception) {
            Log.e(TAG, "Failed to bind local loopback server: ${e.message}")
            stopAccessory()
            return
        }

        activeServer = server
        _localProxyPort.value = server.localPort
        _isAoaActive.value = true
        Log.i(TAG, "AOA proxy listening on 127.0.0.1:${server.localPort}")

        val inStream = FileInputStream(pfd.fileDescriptor)
        val outStream = FileOutputStream(pfd.fileDescriptor)

        scope.launch {
            handleUsbIncoming(inStream)
        }

        scope.launch {
            handleServerAccept(server, outStream)
        }
    }

    @Synchronized
    fun stopAccessory() {
        isRunning.set(false)
        _isAoaActive.value = false

        for ((_, socket) in activeStreams) {
            try { socket.close() } catch (_: Exception) {}
        }
        activeStreams.clear()

        try { activeServer?.close() } catch (_: Exception) {}
        activeServer = null

        try { activePfd?.close() } catch (_: Exception) {}
        activePfd = null

        Log.i(TAG, "AOA accessory stopped")
    }

    private fun handleServerAccept(server: ServerSocket, outStream: OutputStream) {
        while (isRunning.get() && !server.isClosed) {
            val clientSocket = try {
                server.accept()
            } catch (_: Exception) {
                break
            }

            val rawId = nextStreamId.getAndIncrement()
            if (rawId > 32700) nextStreamId.set(1)
            val streamId = rawId.toShort()
            activeStreams[streamId] = clientSocket

            scope.launch {
                sendDataFrame(outStream, streamId, FRAME_FLAG_OPEN, null, 0, 0)

                val inSock = clientSocket.getInputStream()
                val buf = ByteArray(MAX_PAYLOAD_LEN)
                try {
                    while (isRunning.get() && !clientSocket.isClosed) {
                        val n = inSock.read(buf)
                        if (n <= 0) break
                        sendDataFrame(outStream, streamId, FRAME_FLAG_DATA, buf, 0, n)
                    }
                } catch (_: Exception) {}

                sendDataFrame(outStream, streamId, FRAME_FLAG_CLOSE, null, 0, 0)
                activeStreams.remove(streamId)
                try { clientSocket.close() } catch (_: Exception) {}
            }
        }
    }

    private fun handleUsbIncoming(inStream: InputStream) {
        val rxBuf = ByteArray(MAX_PAYLOAD_LEN + FRAME_HEADER_LEN)
        var accBuf = ByteArray(65536)
        var accLen = 0

        while (isRunning.get()) {
            val bytesRead = try {
                inStream.read(rxBuf)
            } catch (_: Exception) {
                break
            }
            if (bytesRead <= 0) break

            if (accLen + bytesRead > accBuf.size) {
                val newCap = maxOf(accBuf.size * 2, accLen + bytesRead)
                val expanded = ByteArray(newCap)
                System.arraycopy(accBuf, 0, expanded, 0, accLen)
                accBuf = expanded
            }
            System.arraycopy(rxBuf, 0, accBuf, accLen, bytesRead)
            accLen += bytesRead

            var offset = 0
            while (accLen - offset >= FRAME_HEADER_LEN) {
                val streamId = (((accBuf[offset].toInt() and 0xFF) shl 8) or (accBuf[offset + 1].toInt() and 0xFF)).toShort()
                val flags = accBuf[offset + 2]
                val payloadLen = ((accBuf[offset + 3].toInt() and 0xFF) shl 8) or (accBuf[offset + 4].toInt() and 0xFF)
                val totalFrameLen = FRAME_HEADER_LEN + payloadLen

                if (accLen - offset < totalFrameLen) break

                if ((flags.toInt() and FRAME_FLAG_DATA.toInt()) != 0 && payloadLen > 0) {
                    val sock = activeStreams[streamId]
                    if (sock != null && !sock.isClosed) {
                        try {
                            sock.getOutputStream().write(accBuf, offset + FRAME_HEADER_LEN, payloadLen)
                            sock.getOutputStream().flush()
                        } catch (_: Exception) {
                            activeStreams.remove(streamId)
                            try { sock.close() } catch (_: Exception) {}
                        }
                    }
                } else if ((flags.toInt() and FRAME_FLAG_CLOSE.toInt()) != 0) {
                    val sock = activeStreams.remove(streamId)
                    try { sock?.close() } catch (_: Exception) {}
                }

                offset += totalFrameLen
            }

            if (offset > 0) {
                val remaining = accLen - offset
                if (remaining > 0) {
                    System.arraycopy(accBuf, offset, accBuf, 0, remaining)
                }
                accLen = remaining
            }
        }

        stopAccessory()
    }

    @Synchronized
    private fun sendDataFrame(
        outStream: OutputStream,
        streamId: Short,
        flags: Byte,
        payload: ByteArray?,
        payloadOffset: Int,
        payloadLen: Int,
    ) {
        try {
            val total = FRAME_HEADER_LEN + payloadLen
            val frame = ByteArray(total)
            frame[0] = (streamId.toInt() ushr 8).toByte()
            frame[1] = streamId.toByte()
            frame[2] = flags
            frame[3] = (payloadLen ushr 8).toByte()
            frame[4] = payloadLen.toByte()
            if (payload != null && payloadLen > 0) {
                System.arraycopy(payload, payloadOffset, frame, FRAME_HEADER_LEN, payloadLen)
            }
            outStream.write(frame)
            outStream.flush()
        } catch (_: Exception) {}
    }
}
