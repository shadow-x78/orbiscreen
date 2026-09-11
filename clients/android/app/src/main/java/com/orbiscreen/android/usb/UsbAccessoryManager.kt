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
    const val ACTION_USB_PERMISSION = "com.orbiscreen.android.USB_PERMISSION"
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

    private val _accessoryDetachedEvent = kotlinx.coroutines.flow.MutableSharedFlow<Unit>(extraBufferCapacity = 1)
    val accessoryDetachedEvent: kotlinx.coroutines.flow.SharedFlow<Unit> = _accessoryDetachedEvent

    private val _autoConnectEvent = kotlinx.coroutines.flow.MutableSharedFlow<Int>(
        replay = 1,
        extraBufferCapacity = 1,
        onBufferOverflow = kotlinx.coroutines.channels.BufferOverflow.DROP_OLDEST,
    )
    val autoConnectEvent: kotlinx.coroutines.flow.SharedFlow<Int> = _autoConnectEvent

    private var activePfd: ParcelFileDescriptor? = null
    private var activeServer: ServerSocket? = null
    private val isRunning = AtomicBoolean(false)
    private val nextStreamId = AtomicInteger(1)
    private val activeStreams = ConcurrentHashMap<Short, Socket>()

    fun init(context: Context) {
        val usbManager = context.getSystemService(Context.USB_SERVICE) as? UsbManager ?: return
        val accessories = usbManager.accessoryList ?: return
        for (acc in accessories) {
            val matches = acc.manufacturer?.equals("shadow-x78", ignoreCase = true) == true ||
                acc.model?.equals("Orbiscreen", ignoreCase = true) == true ||
                accessories.size == 1
            if (matches) {
                if (usbManager.hasPermission(acc)) {
                    startAccessory(context, acc)
                } else {
                    requestPermission(context, acc)
                }
                break
            }
        }
    }

    fun requestPermission(context: Context, accessory: UsbAccessory) {
        val usbManager = context.getSystemService(Context.USB_SERVICE) as? UsbManager ?: return
        val flags = if (android.os.Build.VERSION.SDK_INT >= android.os.Build.VERSION_CODES.S) {
            android.app.PendingIntent.FLAG_MUTABLE
        } else {
            0
        }
        val intent = android.content.Intent(ACTION_USB_PERMISSION).apply {
            `package` = context.packageName
        }
        val permissionIntent = android.app.PendingIntent.getBroadcast(
            context,
            0,
            intent,
            flags
        )
        usbManager.requestPermission(accessory, permissionIntent)
    }

    fun onAccessoryAttached(context: Context, accessory: UsbAccessory) {
        val usbManager = context.getSystemService(Context.USB_SERVICE) as? UsbManager ?: return
        if (usbManager.hasPermission(accessory)) {
            startAccessory(context, accessory)
        } else {
            requestPermission(context, accessory)
        }
    }

    fun isAccessoryConnected(context: Context): Boolean {
        val usbManager = context.getSystemService(Context.USB_SERVICE) as? UsbManager ?: return false
        val list = usbManager.accessoryList ?: return false
        return list.any {
            it.manufacturer?.equals("shadow-x78", ignoreCase = true) == true ||
                it.model?.equals("Orbiscreen", ignoreCase = true) == true ||
                list.size == 1
        }
    }

    @OptIn(kotlinx.coroutines.ExperimentalCoroutinesApi::class)
    fun onAccessoryDetached() {
        stopAccessory()
        _autoConnectEvent.resetReplayCache()
        _accessoryDetachedEvent.tryEmit(Unit)
    }

    @Synchronized
    fun startAccessory(context: Context, accessory: UsbAccessory) {
        if (isRunning.get()) return
        val usbManager = context.getSystemService(Context.USB_SERVICE) as? UsbManager ?: return
        if (!usbManager.hasPermission(accessory)) {
            requestPermission(context, accessory)
            return
        }
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

        _autoConnectEvent.tryEmit(server.localPort)
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
                server.accept().apply {
                    tcpNoDelay = true
                    sendBufferSize = 16384
                    receiveBufferSize = 16384
                }
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
                val newCap = minOf(131072, maxOf(accBuf.size * 2, accLen + bytesRead))
                if (newCap > accBuf.size) {
                    val expanded = ByteArray(newCap)
                    System.arraycopy(accBuf, 0, expanded, 0, accLen)
                    accBuf = expanded
                } else {
                    accLen = 0
                    continue
                }
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
