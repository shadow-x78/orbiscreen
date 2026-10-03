
package com.orbiscreen.android.ui.stream

internal data class ContentRect(val offsetX: Float, val offsetY: Float, val w: Float, val h: Float)

internal fun computeContentRect(viewW: Int, viewH: Int, streamW: Int, streamH: Int, scaleMode: Int = 0): ContentRect {
    if (streamW <= 0 || streamH <= 0 || viewW <= 0 || viewH <= 0) {
        return ContentRect(0f, 0f, viewW.toFloat().coerceAtLeast(1f), viewH.toFloat().coerceAtLeast(1f))
    }
    if (scaleMode == 3) {
        return ContentRect(0f, 0f, viewW.toFloat(), viewH.toFloat())
    }
    val streamAspect = streamW.toFloat() / streamH.toFloat()
    val viewAspect = viewW.toFloat() / viewH.toFloat()
    return if (streamAspect > viewAspect) {
        val contentH = viewW / streamAspect
        ContentRect(0f, (viewH - contentH) / 2f, viewW.toFloat(), contentH)
    } else {
        val contentW = viewH * streamAspect
        ContentRect((viewW - contentW) / 2f, 0f, contentW, viewH.toFloat())
    }
}
