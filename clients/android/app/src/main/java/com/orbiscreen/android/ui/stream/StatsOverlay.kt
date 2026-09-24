// Orbiscreen - StatsOverlay.kt (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen

package com.orbiscreen.android.ui.stream

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.Path
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.PlatformTextStyle
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.orbiscreen.android.R
import com.orbiscreen.android.player.StatsSnapshot
import com.orbiscreen.android.player.StreamStats
import com.orbiscreen.android.ui.theme.ActiveGreen
import com.orbiscreen.android.ui.theme.DarkError
import com.orbiscreen.android.ui.theme.OrbiLogoBlue
import kotlinx.coroutines.delay

private val StatsGlass = Color(0x9E11111B)
private val StatsBorder = Color(0x6689B4FA)
private val BytesLine = Color(0xFF74C7EC)
private val BytesFill = Color(0x5274C7EC)
private val OtherGray = Color(0xFF6C7086)

@Composable
fun StatsOverlay(
    stats: StreamStats,
    transport: String = "",
    modifier: Modifier = Modifier,
) {
    var snap by remember { mutableStateOf(stats.snapshot()) }
    LaunchedEffect(stats) {
        while (true) {
            snap = stats.snapshot()
            delay(100)
        }
    }
    Column(
        modifier = modifier
            .width(StatsLayout.WIDTH_DP.dp)
            .height(StatsLayout.HEIGHT_DP.dp)
            .clip(RoundedCornerShape(14.dp))
            .background(StatsGlass)
            .border(1.dp, StatsBorder, RoundedCornerShape(14.dp))
            .padding(horizontal = StatsLayout.PAD_H_DP.dp, vertical = StatsLayout.PAD_V_DP.dp),
    ) {
        Text(
            text = StreamTransport.statsTitle(stringResource(R.string.stats_title), transport),
            maxLines = 1,
            style = compactText(11.sp, FontWeight.SemiBold),
            color = Color.White,
            letterSpacing = 0.4.sp,
            modifier = Modifier
                .padding(bottom = StatsLayout.TITLE_BOTTOM_DP.dp)
                .height(StatsLayout.ROW_LINE_HEIGHT_SP.dp),
        )
        StatRow(stringResource(R.string.stats_delay), StreamStats.formatMs(snap.delayMs))
        StatRow(stringResource(R.string.stats_age), StreamStats.formatMs(snap.ageMs))
        StatRow(stringResource(R.string.stats_received), StreamStats.formatRate(snap.bytesPerSec))
        BytesSparkline(
            snap.bytesSeries,
            Modifier
                .fillMaxWidth()
                .padding(top = StatsLayout.GRAPH_MARGIN_TOP_DP.dp, bottom = StatsLayout.GRAPH_MARGIN_BOTTOM_DP.dp)
                .height(StatsLayout.GRAPH_HEIGHT_DP.dp),
        )
        StatRow(
            stringResource(R.string.stats_frames),
            stringResource(R.string.stats_fps, snap.fps),
        )
        FramesStack(
            snap,
            Modifier
                .fillMaxWidth()
                .padding(top = StatsLayout.GRAPH_MARGIN_TOP_DP.dp, bottom = StatsLayout.GRAPH_MARGIN_BOTTOM_DP.dp)
                .height(StatsLayout.GRAPH_HEIGHT_DP.dp),
        )
        Row(
            horizontalArrangement = Arrangement.spacedBy(10.dp),
            verticalAlignment = Alignment.CenterVertically,
            modifier = Modifier
                .fillMaxWidth()
                .height(StatsLayout.FOOTER_HEIGHT_DP.dp),
        ) {
            LegendSwatch(ActiveGreen, "I ${snap.lastMinuteI}")
            LegendSwatch(OrbiLogoBlue, "P ${snap.lastMinuteP}")
            LegendSwatch(DarkError, "D ${snap.lastMinuteDropped}")
            Spacer(Modifier.weight(1f))
            Text(
                text = stringResource(R.string.stats_window),
                style = compactText(10.sp, FontWeight.Normal),
                color = Color.White.copy(alpha = 0.48f),
            )
        }
    }
}

private fun compactText(size: androidx.compose.ui.unit.TextUnit, weight: FontWeight): TextStyle =
    TextStyle(
        fontSize = size,
        lineHeight = StatsLayout.ROW_LINE_HEIGHT_SP.sp,
        fontWeight = weight,
        platformStyle = PlatformTextStyle(includeFontPadding = false),
    )

@Composable
private fun StatRow(label: String, value: String) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .padding(bottom = StatsLayout.ROW_GAP_DP.dp)
            .height(StatsLayout.ROW_LINE_HEIGHT_SP.dp),
        horizontalArrangement = Arrangement.SpaceBetween,
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Text(
            text = label,
            style = compactText(11.sp, FontWeight.Medium),
            color = Color.White.copy(alpha = 0.62f),
        )
        Text(
            text = value,
            style = compactText(12.sp, FontWeight.SemiBold),
            color = Color.White,
            fontFamily = FontFamily.Monospace,
        )
    }
}

@Composable
private fun LegendSwatch(color: Color, label: String) {
    Row(verticalAlignment = Alignment.CenterVertically) {
        Spacer(
            Modifier
                .size(7.dp)
                .clip(CircleShape)
                .background(color),
        )
        Spacer(Modifier.width(4.dp))
        Text(
            label,
            style = compactText(10.sp, FontWeight.Normal),
            color = Color.White.copy(alpha = 0.78f),
        )
    }
}

@Composable
private fun BytesSparkline(series: LongArray, modifier: Modifier = Modifier) {
    Canvas(modifier) {
        if (series.isEmpty()) return@Canvas
        val max = series.max().coerceAtLeast(1L).toFloat()
        val path = Path()
        val n = series.size
        path.moveTo(0f, size.height)
        for (i in series.indices) {
            val x = if (n == 1) 0f else i.toFloat() / (n - 1) * size.width
            val y = size.height - (series[i] / max) * (size.height - 2f)
            path.lineTo(x, y)
        }
        path.lineTo(size.width, size.height)
        path.close()
        drawPath(path, BytesFill)
        val line = Path()
        for (i in series.indices) {
            val x = if (n == 1) 0f else i.toFloat() / (n - 1) * size.width
            val y = size.height - (series[i] / max) * (size.height - 2f)
            if (i == 0) line.moveTo(x, y) else line.lineTo(x, y)
        }
        drawPath(line, BytesLine, style = Stroke(width = 2f))
    }
}

@Composable
private fun FramesStack(snap: StatsSnapshot, modifier: Modifier = Modifier) {
    Canvas(modifier) {
        val n = snap.frameI.size
        if (n == 0) return@Canvas
        var max = 1
        for (i in 0 until n) {
            val t = snap.frameI[i] + snap.frameP[i] + snap.frameOther[i] + snap.frameDropped[i]
            if (t > max) max = t
        }
        val barW = size.width / n
        for (i in 0 until n) {
            var y = size.height
            val layers = arrayOf(
                snap.frameI[i] to ActiveGreen,
                snap.frameP[i] to OrbiLogoBlue,
                snap.frameOther[i] to OtherGray,
                snap.frameDropped[i] to DarkError,
            )
            for ((count, color) in layers) {
                if (count <= 0) continue
                val bh = count.toFloat() / max * size.height
                y -= bh
                drawRect(
                    color = color,
                    topLeft = Offset(i * barW, y),
                    size = Size((barW - 0.6f).coerceAtLeast(1f), bh),
                )
            }
        }
    }
}
