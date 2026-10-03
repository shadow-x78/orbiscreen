
package com.orbiscreen.android.player

import org.junit.Assert.assertEquals
import org.junit.Test

class H264Test {

    @Test
    fun idrIsIFrame() {
        val au = withStart(byteArrayOf(0x65, 0x88.toByte()))
        assertEquals(FrameKind.I, H264.classifyAccessUnit(au))
    }

    @Test
    fun pSliceIsPFrame() {
        val au = sliceAu(nalType = 1, sliceType = 0)
        assertEquals(FrameKind.P, H264.classifyAccessUnit(au))
    }

    @Test
    fun bSliceIsBFrame() {
        val au = sliceAu(nalType = 1, sliceType = 1)
        assertEquals(FrameKind.B, H264.classifyAccessUnit(au))
    }

    @Test
    fun nonIdrISliceIsIFrame() {
        val au = sliceAu(nalType = 1, sliceType = 2)
        assertEquals(FrameKind.I, H264.classifyAccessUnit(au))
    }

    @Test
    fun iAllSliceTypeSevenIsIFrame() {
        val au = sliceAu(nalType = 1, sliceType = 7)
        assertEquals(FrameKind.I, H264.classifyAccessUnit(au))
    }

    @Test
    fun spsPpsOnlyIsOther() {
        val sps = withStart(byteArrayOf(0x67, 0x64, 0x00, 0x28))
        val pps = withStart(byteArrayOf(0x68, 0xEE.toByte(), 0x3C))
        assertEquals(FrameKind.Other, H264.classifyAccessUnit(sps + pps))
    }

    @Test
    fun keyframeAuWithSpsIsI() {
        val sps = withStart(byteArrayOf(0x67, 0x64, 0x00, 0x28))
        val pps = withStart(byteArrayOf(0x68, 0xEE.toByte(), 0x3C))
        val idr = withStart(byteArrayOf(0x65, 0x88.toByte()))
        assertEquals(FrameKind.I, H264.classifyAccessUnit(sps + pps + idr))
    }

    @Test
    fun emulationPreventionDoesNotBreakSliceType() {
        
        val nal = byteArrayOf(0x61, 0xC0.toByte())
        assertEquals(FrameKind.P, H264.sliceKind(nal))
    }

    private fun sliceAu(nalType: Int, sliceType: Int): ByteArray {
        val bits = encodeUe(0) + encodeUe(sliceType)
        val payload = bitsToBytes(bits)
        val header = (0x60 or (nalType and 0x1F)).toByte()
        return withStart(byteArrayOf(header) + payload)
    }

    private fun withStart(nal: ByteArray): ByteArray = byteArrayOf(0, 0, 0, 1) + nal

    private fun encodeUe(codeNum: Int): String {
        val x = codeNum + 1
        val zeros = 31 - Integer.numberOfLeadingZeros(x)
        val suffix = Integer.toBinaryString(x).substring(1)
        return "0".repeat(zeros) + "1" + suffix
    }

    private fun bitsToBytes(bits: String): ByteArray {
        val padded = bits.padEnd(((bits.length + 7) / 8) * 8, '0')
        return ByteArray(padded.length / 8) { i ->
            padded.substring(i * 8, i * 8 + 8).toInt(2).toByte()
        }
    }
}
