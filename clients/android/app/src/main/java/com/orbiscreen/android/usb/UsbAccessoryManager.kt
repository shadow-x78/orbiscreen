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
import java.nio.ByteBuffer
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
            try {
                socket.close()
            } catch (_: Exception) {}
        }
        activeStreams.clear()

        try {
            activeServer?.close()
        } catch (_: Exception) {}
        activeServer = null

        try {
            activePfd?.close()
        } catch (_: Exception) {}
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
                sendFrame(outStream, streamId, FRAME_FLAG_OPEN, ByteArray(0))

                val inSock = clientSocket.getInputStream()
                val buf = ByteArray(MAX_PAYLOAD_LEN)
                try {
                    while (isRunning.get() && !clientSocket.isClosed) {
                        val n = inSock.read(buf)
                        if (n <= 0) break
                        val chunk = buf.copyOf(n)
                        sendFrame(outStream, streamId, FRAME_FLAG_DATA, chunk)
                    }
                } catch (_: Exception) {}

                sendFrame(outStream, streamId, FRAME_FLAG_CLOSE, ByteArray(0))
                activeStreams.remove(streamId)
                try {
                    clientSocket.close()
                } catch (_: Exception) {}
            }
        }
    }

    private fun handleUsbIncoming(inStream: InputStream) {
        val headerBuf = ByteArray(FRAME_HEADER_LEN)
        while (isRunning.get()) {
            try {
                if (!readFully(inStream, headerBuf)) break
                val streamId = ByteBuffer.wrap(headerBuf, 0, 2).short
                val flags = headerBuf[2]
                val payloadLen = ByteBuffer.wrap(headerBuf, 3, 2).short.toInt() and 0xFFFF

                val payload = if (payloadLen > 0) {
                    val p = ByteArray(payloadLen)
                    if (!readFully(inStream, p)) break
                    p
                } else {
                    ByteArray(0)
                }

                if ((flags.toInt() and FRAME_FLAG_DATA.toInt()) != 0) {
                    val sock = activeStreams[streamId]
                    if (sock != null && !sock.isClosed) {
                        try {
                            sock.getOutputStream().write(payload)
                            sock.getOutputStream().flush()
                        } catch (_: Exception) {
                            activeStreams.remove(streamId)
                        }
                    }
                } else if ((flags.toInt() and FRAME_FLAG_CLOSE.toInt()) != 0) {
                    val sock = activeStreams.remove(streamId)
                    try {
                        sock?.close()
                    } catch (_: Exception) {}
                }
            } catch (_: Exception) {
                break
            }
        }

        stopAccessory()
    }

    @Synchronized
    private fun sendFrame(outStream: OutputStream, streamId: Short, flags: Byte, payload: ByteArray) {
        try {
            val total = FRAME_HEADER_LEN + payload.size
            val frame = ByteArray(total)
            ByteBuffer.wrap(frame).apply {
                putShort(streamId)
                put(flags)
                putShort(payload.size.toShort())
                if (payload.isNotEmpty()) {
                    put(payload)
                }
            }
            outStream.write(frame)
            outStream.flush()
        } catch (_: Exception) {}
    }

    private fun readFully(stream: InputStream, buf: ByteArray): Boolean {
        var offset = 0
        while (offset < buf.size) {
            val r = stream.read(buf, offset, buf.size - offset)
            if (r < 0) return false
            offset += r
        }
        return true
    }
}

