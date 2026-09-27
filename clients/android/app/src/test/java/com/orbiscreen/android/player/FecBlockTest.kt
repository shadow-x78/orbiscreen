// Orbiscreen - FecBlockTest.kt (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen

package com.orbiscreen.android.player

import org.junit.Assert.assertArrayEquals
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

class FecBlockTest {
    @Test
    fun twoHundredFiftyTwoShardsRecoverTwoErasures() {
        val src = Array(Fec.MAX_DATA_SHARDS) { i -> ByteArray(4) { b -> (i + b).toByte() } }
        val parity = Fec.encode(src, 4)
        val data = arrayOfNulls<ByteArray>(src.size)
        for (i in src.indices) data[i] = src[i]
        data[1] = null
        data[250] = null
        val paritySlots = arrayOfNulls<ByteArray>(parity.size)
        for (i in parity.indices) paritySlots[i] = parity[i]
        assertTrue(Fec.recover(data, paritySlots))
        assertArrayEquals(src[1], data[1])
        assertArrayEquals(src[250], data[250])
    }

    @Test
    fun oversizedAuSplitsAndRecoversAMissingShard() {
        val au = ByteArray(2013) { 7 }
        val blocks = Fec.blocks(au, 8)
        assertTrue(blocks.size > 1)
        assertTrue(blocks.all { it.data.size <= Fec.MAX_DATA_SHARDS && it.count == blocks.size })
        val store = PendingFecStore()
        var done: PendingFecStore.Complete? = null
        for (block in blocks) {
            val dropFirst = block.index == 0 && block.parity.isNotEmpty()
            for (i in block.data.indices) {
                if (dropFirst && i == 0) continue
                done = store.offer(
                    9, i, block.data.size, true, 4L, block.data[i], block.index, block.count,
                )
            }
            if (dropFirst) {
                done = store.offer(
                    9,
                    block.data.size,
                    block.data.size,
                    true,
                    4L,
                    block.parity[0],
                    block.index,
                    block.count,
                )
            }
        }
        assertNotNull(done)
        assertArrayEquals(au, done!!.au)
    }

    @Test
    fun blockHeaderIsOnlyPresentWhenTheFrameIsSplit() {
        val plain = byteArrayOf(1, 2, 3)
        val single = Fec.viewBlock(0, plain)
        assertNotNull(single)
        assertArrayEquals(plain, single!!.body)
        val split = Fec.viewBlock(Fec.BLOCK_FLAG, byteArrayOf(1, 2, 9))
        assertNotNull(split)
        assertTrue(split!!.index == 1 && split.count == 2)
        assertArrayEquals(byteArrayOf(9), split.body)
        assertNull(Fec.viewBlock(Fec.BLOCK_FLAG, byteArrayOf(1)))
    }
}
