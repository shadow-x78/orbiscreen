package com.orbiscreen.android.net

object Protocol {
    const val UDP_MAGIC = "ORB1"
    const val SEAL_MAGIC = "ORB2"

    const val UDP_HEADER_LEN = 5
    const val UDP_TYPE_VIDEO: Byte = 1
    const val UDP_TYPE_HELLO: Byte = 2
    const val UDP_TYPE_HELLO_ACK: Byte = 3
    const val UDP_TYPE_PING: Byte = 4
    const val UDP_TYPE_PONG: Byte = 5
    const val UDP_TYPE_IDR: Byte = 6
    const val UDP_TYPE_PROBE: Byte = 7
    const val UDP_TYPE_PROBE_ACK: Byte = 8
    const val UDP_TYPE_PMTU: Byte = 9
    const val UDP_TYPE_BYE: Byte = 10

    const val UDP_PING_LEN = 13
    const val UDP_PONG_LEN = 21
    const val UDP_PROBE_ACK_LEN = 9
    const val UDP_PMTU_LEN = 7

    const val UDP_VIDEO_HEADER_LEN = 28
    const val UDP_VIDEO_FLAGS_OFFSET = 5
    const val UDP_VIDEO_SEQ_OFFSET = 6
    const val UDP_VIDEO_FRAG_OFFSET = 8
    const val UDP_VIDEO_FRAGS_OFFSET = 10
    const val UDP_VIDEO_SENT_NS_OFFSET = 20
    const val VIDEO_FLAG_KEY = 0x01

    const val SEAL_HEADER_LEN = 32
    const val SEAL_OVERHEAD = 48
    const val SEAL_KEY_ID_OFFSET = 4
    const val SEAL_KEY_ID_LEN = 16
    const val SEAL_NONCE_OFFSET = 20
    const val SEAL_NONCE_LEN = 12

    const val AOA_FLAG_DATA = 0x01
    const val AOA_FLAG_OPEN = 0x02
    const val AOA_FLAG_CLOSE = 0x04
    const val AOA_FLAG_RESET = 0x08
    const val AOA_FLAG_VIDEO = 0x10
    const val AOA_HEADER_LEN = 5
    const val AOA_MAX_FRAME = 16384
    const val AOA_MAX_PAYLOAD = AOA_MAX_FRAME - AOA_HEADER_LEN
    const val AOA_VIDEO_STREAM_ID: Short = 0
    const val AOA_MAX_STREAM_ID = 32700

    const val VIDEO_FRAME_LENGTH_PREFIX_LEN = 4
    const val VIDEO_FRAME_HEADER_LEN = 18
    const val VIDEO_FRAME_MAX = 4 * 1024 * 1024
    const val VIDEO_FRAME_TYPE = 1

    const val FEC_MAX_DATA_SHARDS = 252
    const val FEC_BLOCK_FLAG = 0x02
    const val FEC_PARITY_TIER_1 = 3
    const val FEC_PARITY_TIER_2 = 16
    const val FEC_PARITY_TIER_3 = 64
    const val FEC_PARITY_MAX = 4

    const val PATH_STREAM = "/stream"
    const val PATH_AU = "/au"
    const val PATH_IDR = "/idr"
    const val PATH_INPUT = "/input"
    const val PATH_INPUT_WS = "/input/ws"
    const val PATH_CONTROL = "/api/control"
    const val PATH_SESSION = "/api/session"
    const val PATH_INFO = "/api/info"
    const val PATH_HEALTH = "/health"
    const val PATH_UDP_KEY = "/api/udp-key"
    const val PATH_CLIENT_CONFIG = "/client/config.json"
    const val PATH_PAIR_REQUEST = "/api/pair/request"
    const val PATH_PAIR_STATUS = "/api/pair/status"

    const val HEADER_AUTHORIZATION = "Authorization"
    const val HEADER_SESSION = "X-Orbiscreen-Session"
    const val HEADER_CLIENT_KEY = "X-Orbiscreen-Client-Key"

    const val QUERY_TOKEN = "token"
    const val QUERY_SESSION = "session"
    const val QUERY_KEY = "key"

    fun bearer(token: String): String = "Bearer $token"

    fun hasMagic(data: ByteArray, magic: String, offset: Int = 0): Boolean {
        if (offset < 0 || offset + magic.length > data.size) return false
        for (i in magic.indices) {
            if (data[offset + i] != magic[i].code.toByte()) return false
        }
        return true
    }

    fun hasUdpMagic(data: ByteArray): Boolean = hasMagic(data, UDP_MAGIC)

    fun hasSealMagic(data: ByteArray): Boolean = hasMagic(data, SEAL_MAGIC)

    fun udpDatagram(type: Byte, size: Int = UDP_HEADER_LEN): ByteArray {
        require(size >= UDP_HEADER_LEN) { "UDP datagram shorter than its header" }
        val out = ByteArray(size)
        UDP_MAGIC.toByteArray(Charsets.US_ASCII).copyInto(out)
        out[4] = type
        return out
    }

    fun fecParityCount(dataShards: Int): Int = when {
        dataShards <= FEC_PARITY_TIER_1 -> 0
        dataShards <= FEC_PARITY_TIER_2 -> 2
        dataShards <= FEC_PARITY_TIER_3 -> 3
        else -> FEC_PARITY_MAX
    }
}
