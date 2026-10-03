
package com.orbiscreen.android.player

import org.junit.Assert.assertArrayEquals
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class FecTest {
    @Test
    fun ladder() {
        assertEquals(0, Fec.parityCount(3))
        assertEquals(2, Fec.parityCount(4))
        assertEquals(2, Fec.parityCount(16))
        assertEquals(3, Fec.parityCount(17))
        assertEquals(3, Fec.parityCount(64))
        assertEquals(4, Fec.parityCount(65))
    }

    @Test
    fun recoversTwoErasures() {
        val k = 4
        val width = 16
        val src = Array(k) { i ->
            ByteArray(width) { b -> ((i * 31 + b) and 0xff).toByte() }
        }
        val parity = Fec.encode(src, 2)
        val data: Array<ByteArray?> = Array(k) { src[it].copyOf() }
        data[1] = null
        data[3] = null
        val pslots: Array<ByteArray?> = Array(parity.size) { parity[it] }
        assertTrue(Fec.recover(data, pslots))
        for (i in 0 until k) {
            assertArrayEquals(src[i], data[i])
        }
    }

    @Test
    fun failsWhenTooManyErasures() {
        val src = Array(4) { i -> ByteArray(8) { b -> (i + b).toByte() } }
        val parity = Fec.encode(src, 2)
        val data: Array<ByteArray?> = Array(4) { src[it].copyOf() }
        data[0] = null
        data[1] = null
        data[2] = null
        assertFalse(Fec.recover(data, Array(2) { parity[it] }))
    }
}
