
package com.orbiscreen.android.player

import org.junit.Assert.assertArrayEquals
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertNull
import org.junit.Test

class PendingFecStoreTest {
    private fun shards(au: ByteArray, chunk: Int): Pair<Array<ByteArray>, Array<ByteArray>> {
        val prefix = ByteArray(4)
        val n = au.size
        prefix[0] = (n and 0xff).toByte()
        prefix[1] = ((n shr 8) and 0xff).toByte()
        prefix[2] = ((n shr 16) and 0xff).toByte()
        prefix[3] = ((n shr 24) and 0xff).toByte()
        val blob = prefix + au
        val parts = blob.toList().chunked(chunk).map { it.toByteArray() }.toTypedArray()
        val padded = Array(parts.size) { i ->
            ByteArray(chunk).also { parts[i].copyInto(it) }
        }
        val parity = Fec.encode(padded, Fec.parityCount(parts.size))
        return parts to parity
    }

    @Test
    fun allDataEmitsWithoutWaitingForParity() {
        val store = PendingFecStore()
        val au = ByteArray(36) { it.toByte() }
        val (data, _) = shards(au, 10)
        assertEquals(4, data.size)
        var out: PendingFecStore.Complete? = null
        for (i in data.indices) {
            out = store.offer(3, i, data.size, false, 9L, data[i])
        }
        assertNotNull(out)
        assertArrayEquals(au, out!!.au)
        assertEquals(0, store.size)
    }

    @Test
    fun lateParityDoesNotCreateANewPendingAu() {
        val store = PendingFecStore()
        val au = ByteArray(36) { it.toByte() }
        val (data, parity) = shards(au, 10)
        for (i in data.indices) store.offer(3, i, data.size, false, 9L, data[i])
        assertEquals(0, store.size)
        assertNull(store.offer(3, data.size, data.size, false, 9L, parity[0]))
        assertFalse(store.contains(3))
        assertEquals(0, store.size)
    }

    @Test
    fun parityFirstDoesNotOpenAPendingAu() {
        val store = PendingFecStore()
        val au = ByteArray(36) { it.toByte() }
        val (data, parity) = shards(au, 10)
        assertNull(store.offer(5, data.size, data.size, false, 1L, parity[0]))
        assertEquals(0, store.size)
        var out: PendingFecStore.Complete? = null
        for (i in data.indices) {
            out = store.offer(5, i, data.size, false, 1L, data[i])
        }
        assertNotNull(out)
        assertArrayEquals(au, out!!.au)
    }

    @Test
    fun missingDataPlusParityRecovers() {
        val store = PendingFecStore()
        val au = ByteArray(36) { it.toByte() }
        val (data, parity) = shards(au, 10)
        val k = data.size
        store.offer(1, 0, k, false, 2L, data[0])
        store.offer(1, 2, k, false, 2L, data[2])
        store.offer(1, 3, k, false, 2L, data[3])
        val out = store.offer(1, k, k, false, 2L, parity[0])
        assertNotNull(out)
        assertArrayEquals(au, out!!.au)
        assertEquals(0, store.size)
    }
}
