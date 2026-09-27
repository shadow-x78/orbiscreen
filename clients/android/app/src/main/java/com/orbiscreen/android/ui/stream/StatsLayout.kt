// Orbiscreen - StatsLayout.kt (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen
package com.orbiscreen.android.ui.stream

/** Pixel/dp metrics shared with the web `.statsWidget` so both HUDs match. */
object StatsLayout {
    const val WIDTH_DP = 236
    const val PAD_H_DP = 12
    const val PAD_V_DP = 10
    const val ROW_GAP_DP = 2
    const val TITLE_BOTTOM_DP = 6
    const val GRAPH_HEIGHT_DP = 40
    const val GRAPH_MARGIN_TOP_DP = 4
    const val GRAPH_MARGIN_BOTTOM_DP = 8
    const val ROW_LINE_HEIGHT_SP = 14
    const val FOOTER_HEIGHT_DP = 14

    const val TITLE_BLOCK_DP = ROW_LINE_HEIGHT_SP + TITLE_BOTTOM_DP
    const val ROW_BLOCK_DP = ROW_LINE_HEIGHT_SP + ROW_GAP_DP
    const val GRAPH_BLOCK_DP = GRAPH_MARGIN_TOP_DP + GRAPH_HEIGHT_DP + GRAPH_MARGIN_BOTTOM_DP
    const val CONTENT_HEIGHT_DP =
        TITLE_BLOCK_DP + 4 * ROW_BLOCK_DP + 2 * GRAPH_BLOCK_DP + FOOTER_HEIGHT_DP
    const val HEIGHT_DP = PAD_V_DP * 2 + CONTENT_HEIGHT_DP
}
