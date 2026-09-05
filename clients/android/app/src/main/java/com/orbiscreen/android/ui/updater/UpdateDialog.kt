// Orbiscreen - UpdateDialog.kt (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen

package com.orbiscreen.android.ui.updater

import android.content.Intent
import android.net.Uri
import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.rounded.OpenInNew
import androidx.compose.material.icons.rounded.CheckCircle
import androidx.compose.material.icons.rounded.Security
import androidx.compose.material.icons.rounded.Update
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.FilledTonalButton
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.LinearProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextDecoration
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.window.Dialog
import androidx.compose.ui.window.DialogProperties
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.LifecycleEventObserver
import androidx.lifecycle.compose.LocalLifecycleOwner
import com.orbiscreen.android.R
import com.orbiscreen.android.ui.theme.ActiveGreen
import com.orbiscreen.android.updater.DownloadProgress
import com.orbiscreen.android.updater.ReleaseInfo
import com.orbiscreen.android.updater.UpdateManager
import kotlinx.coroutines.Job
import kotlinx.coroutines.launch
import java.io.File

@Composable
fun UpdateDialog(
    release: ReleaseInfo,
    onDismiss: () -> Unit,
) {
    val context = LocalContext.current
    val scope = rememberCoroutineScope()
    val updateManager = remember { UpdateManager(context) }
    val lifecycleOwner = LocalLifecycleOwner.current

    val cachedApk = remember(release) {
        val file = File(File(context.cacheDir, "updates"), "orbiscreen-${release.versionName}.apk")
        if (file.exists() && file.length() > 0 && (release.apkSize <= 0 || file.length() == release.apkSize)) {
            file
        } else {
            null
        }
    }

    var progress by remember {
        mutableStateOf<DownloadProgress>(
            if (cachedApk != null) DownloadProgress.Ready(cachedApk) else DownloadProgress.Idle
        )
    }
    var downloadJob by remember { mutableStateOf<Job?>(null) }
    var downloadedFile by remember { mutableStateOf<File?>(cachedApk) }
    var hasInstallPermission by remember { mutableStateOf(updateManager.canInstallPackages()) }
    var hasAttemptedAutoInstall by remember { mutableStateOf(false) }

    DisposableEffect(lifecycleOwner) {
        val observer = LifecycleEventObserver { _, event ->
            if (event == Lifecycle.Event.ON_RESUME) {
                val allowed = updateManager.canInstallPackages()
                hasInstallPermission = allowed
                if (allowed && !hasAttemptedAutoInstall) {
                    val fileToInstall = (progress as? DownloadProgress.Ready)?.file ?: downloadedFile
                    if (fileToInstall != null && fileToInstall.exists()) {
                        hasAttemptedAutoInstall = true
                        updateManager.installApk(fileToInstall)
                        onDismiss()
                    }
                }
            }
        }
        lifecycleOwner.lifecycle.addObserver(observer)
        onDispose {
            lifecycleOwner.lifecycle.removeObserver(observer)
        }
    }

    Dialog(
        onDismissRequest = {
            if (progress !is DownloadProgress.Downloading) {
                onDismiss()
            }
        },
        properties = DialogProperties(usePlatformDefaultWidth = false),
    ) {
        Surface(
            shape = RoundedCornerShape(24.dp),
            color = MaterialTheme.colorScheme.surfaceVariant,
            border = BorderStroke(1.dp, MaterialTheme.colorScheme.outlineVariant.copy(alpha = 0.3f)),
            shadowElevation = 16.dp,
            modifier = Modifier
                .fillMaxWidth()
                .padding(horizontal = 20.dp),
        ) {
            Column(
                modifier = Modifier.padding(20.dp),
                verticalArrangement = Arrangement.spacedBy(14.dp),
            ) {
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.SpaceBetween,
                ) {
                    Row(
                        verticalAlignment = Alignment.CenterVertically,
                        horizontalArrangement = Arrangement.spacedBy(10.dp),
                    ) {
                        Surface(
                            shape = CircleShape,
                            color = MaterialTheme.colorScheme.primaryContainer,
                            modifier = Modifier.size(42.dp),
                        ) {
                            Box(contentAlignment = Alignment.Center) {
                                Icon(
                                    imageVector = Icons.Rounded.Update,
                                    contentDescription = null,
                                    tint = MaterialTheme.colorScheme.primary,
                                    modifier = Modifier.size(24.dp),
                                )
                            }
                        }

                        Column {
                            Text(
                                text = stringResource(R.string.update_dialog_title),
                                style = MaterialTheme.typography.titleMedium,
                                fontWeight = FontWeight.Bold,
                                color = MaterialTheme.colorScheme.onSurface,
                            )
                            Spacer(Modifier.height(2.dp))
                            Surface(
                                shape = RoundedCornerShape(6.dp),
                                color = ActiveGreen.copy(alpha = 0.18f),
                            ) {
                                Text(
                                    text = release.tagName,
                                    style = MaterialTheme.typography.labelSmall,
                                    color = ActiveGreen,
                                    fontWeight = FontWeight.Bold,
                                    modifier = Modifier.padding(horizontal = 6.dp, vertical = 2.dp),
                                )
                            }
                        }
                    }

                    IconButton(
                        onClick = {
                            val intent = Intent(Intent.ACTION_VIEW, Uri.parse(release.htmlUrl))
                            context.startActivity(intent)
                        },
                        modifier = Modifier.size(36.dp),
                    ) {
                        Icon(
                            imageVector = Icons.AutoMirrored.Rounded.OpenInNew,
                            contentDescription = stringResource(R.string.update_open_in_browser),
                            tint = MaterialTheme.colorScheme.onSurfaceVariant,
                            modifier = Modifier.size(20.dp),
                        )
                    }
                }

                if (release.releaseNotes.isNotBlank()) {
                    Column(
                        modifier = Modifier.fillMaxWidth(),
                        verticalArrangement = Arrangement.spacedBy(6.dp),
                    ) {
                        Text(
                            text = stringResource(R.string.update_whats_new, release.tagName),
                            style = MaterialTheme.typography.labelMedium,
                            fontWeight = FontWeight.SemiBold,
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                        )

                        Surface(
                            shape = RoundedCornerShape(14.dp),
                            color = MaterialTheme.colorScheme.surface.copy(alpha = 0.7f),
                            border = BorderStroke(1.dp, MaterialTheme.colorScheme.outlineVariant.copy(alpha = 0.2f)),
                            modifier = Modifier
                                .fillMaxWidth()
                                .heightIn(min = 100.dp, max = 220.dp),
                        ) {
                            Column(
                                modifier = Modifier
                                    .padding(12.dp)
                                    .verticalScroll(rememberScrollState()),
                            ) {
                                MarkdownContent(content = release.releaseNotes)
                            }
                        }
                    }
                }

                when (val p = progress) {
                    is DownloadProgress.Idle -> {
                        if (release.apkSize > 0) {
                            val sizeMb = "%.1f".format(release.apkSize / (1024f * 1024f))
                            Row(
                                modifier = Modifier.fillMaxWidth(),
                                horizontalArrangement = Arrangement.End,
                            ) {
                                Text(
                                    text = "$sizeMb MB",
                                    style = MaterialTheme.typography.labelSmall,
                                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                                )
                            }
                        }
                    }
                    is DownloadProgress.Downloading -> {
                        Column(modifier = Modifier.fillMaxWidth()) {
                            Row(
                                modifier = Modifier.fillMaxWidth(),
                                horizontalArrangement = Arrangement.SpaceBetween,
                                verticalAlignment = Alignment.CenterVertically,
                            ) {
                                Text(
                                    text = stringResource(R.string.update_downloading),
                                    style = MaterialTheme.typography.labelSmall,
                                    color = MaterialTheme.colorScheme.primary,
                                    fontWeight = FontWeight.Medium,
                                )
                                Text(
                                    text = "${p.percent}%",
                                    style = MaterialTheme.typography.labelSmall,
                                    fontWeight = FontWeight.Bold,
                                    color = MaterialTheme.colorScheme.primary,
                                )
                            }
                            Spacer(Modifier.height(6.dp))
                            LinearProgressIndicator(
                                progress = { p.percent / 100f },
                                modifier = Modifier
                                    .fillMaxWidth()
                                    .height(8.dp)
                                    .clip(RoundedCornerShape(4.dp)),
                            )
                            Spacer(Modifier.height(4.dp))
                            val readMb = "%.1f".format(p.bytesRead / (1024f * 1024f))
                            val totalMb = "%.1f".format(p.totalBytes / (1024f * 1024f))
                            Text(
                                text = "$readMb / $totalMb MB",
                                style = MaterialTheme.typography.labelSmall,
                                color = MaterialTheme.colorScheme.onSurfaceVariant,
                            )
                        }
                    }
                    is DownloadProgress.Verifying -> {
                        Row(
                            verticalAlignment = Alignment.CenterVertically,
                            horizontalArrangement = Arrangement.spacedBy(8.dp),
                        ) {
                            CircularProgressIndicator(
                                modifier = Modifier.size(16.dp),
                                strokeWidth = 2.dp,
                            )
                            Text(
                                text = stringResource(R.string.update_verifying),
                                style = MaterialTheme.typography.bodySmall,
                                color = MaterialTheme.colorScheme.onSurfaceVariant,
                            )
                        }
                    }
                    is DownloadProgress.Ready -> {
                        if (!hasInstallPermission) {
                            Surface(
                                shape = RoundedCornerShape(12.dp),
                                color = MaterialTheme.colorScheme.errorContainer.copy(alpha = 0.35f),
                                border = BorderStroke(1.dp, MaterialTheme.colorScheme.error.copy(alpha = 0.2f)),
                                modifier = Modifier.fillMaxWidth(),
                            ) {
                                Row(
                                    modifier = Modifier.padding(10.dp),
                                    verticalAlignment = Alignment.CenterVertically,
                                    horizontalArrangement = Arrangement.spacedBy(10.dp),
                                ) {
                                    Icon(
                                        imageVector = Icons.Rounded.Security,
                                        contentDescription = null,
                                        tint = MaterialTheme.colorScheme.error,
                                        modifier = Modifier.size(24.dp),
                                    )
                                    Column {
                                        Text(
                                            text = stringResource(R.string.update_permission_required),
                                            style = MaterialTheme.typography.labelMedium,
                                            fontWeight = FontWeight.Bold,
                                            color = MaterialTheme.colorScheme.onErrorContainer,
                                        )
                                        Spacer(Modifier.height(2.dp))
                                        Text(
                                            text = stringResource(R.string.update_permission_desc),
                                            style = MaterialTheme.typography.bodySmall,
                                            color = MaterialTheme.colorScheme.onErrorContainer,
                                            fontSize = 12.sp,
                                        )
                                    }
                                }
                            }
                        } else {
                            Surface(
                                shape = RoundedCornerShape(12.dp),
                                color = ActiveGreen.copy(alpha = 0.15f),
                                modifier = Modifier.fillMaxWidth(),
                            ) {
                                Row(
                                    modifier = Modifier.padding(10.dp),
                                    verticalAlignment = Alignment.CenterVertically,
                                    horizontalArrangement = Arrangement.spacedBy(8.dp),
                                ) {
                                    Icon(
                                        imageVector = Icons.Rounded.CheckCircle,
                                        contentDescription = null,
                                        tint = ActiveGreen,
                                        modifier = Modifier.size(20.dp),
                                    )
                                    Text(
                                        text = stringResource(R.string.update_install_now),
                                        style = MaterialTheme.typography.labelMedium,
                                        fontWeight = FontWeight.Bold,
                                        color = ActiveGreen,
                                    )
                                }
                            }
                        }
                    }
                    is DownloadProgress.Failed -> {
                        Surface(
                            shape = RoundedCornerShape(10.dp),
                            color = MaterialTheme.colorScheme.errorContainer.copy(alpha = 0.25f),
                            modifier = Modifier.fillMaxWidth(),
                        ) {
                            Text(
                                text = p.reason,
                                style = MaterialTheme.typography.bodySmall,
                                color = MaterialTheme.colorScheme.error,
                                modifier = Modifier.padding(10.dp),
                            )
                        }
                    }
                }

                Row(
                    horizontalArrangement = Arrangement.spacedBy(10.dp),
                    modifier = Modifier.fillMaxWidth(),
                ) {
                    FilledTonalButton(
                        onClick = {
                            if (progress is DownloadProgress.Downloading) {
                                downloadJob?.cancel()
                                progress = DownloadProgress.Idle
                            }
                            onDismiss()
                        },
                        shape = RoundedCornerShape(12.dp),
                        colors = ButtonDefaults.filledTonalButtonColors(
                            containerColor = MaterialTheme.colorScheme.onSurface.copy(alpha = 0.08f),
                            contentColor = MaterialTheme.colorScheme.onSurface,
                        ),
                        modifier = Modifier
                            .weight(1f)
                            .height(42.dp),
                    ) {
                        Text(
                            text = if (progress is DownloadProgress.Downloading) {
                                stringResource(R.string.update_cancel)
                            } else {
                                stringResource(R.string.update_later)
                            },
                            fontWeight = FontWeight.SemiBold,
                            fontSize = 13.sp,
                        )
                    }

                    Button(
                        onClick = {
                            when (val p = progress) {
                                is DownloadProgress.Idle -> {
                                    downloadJob = scope.launch {
                                        val file = updateManager.downloadApk(release) { prog ->
                                            progress = prog
                                        }
                                        if (file != null) {
                                            downloadedFile = file
                                            if (updateManager.canInstallPackages()) {
                                                updateManager.installApk(file)
                                                onDismiss()
                                            }
                                        }
                                    }
                                }
                                is DownloadProgress.Ready -> {
                                    if (!hasInstallPermission) {
                                        updateManager.openInstallPermissionSettings()
                                    } else {
                                        val fileToInstall = p.file.takeIf { it.exists() } ?: downloadedFile
                                        if (fileToInstall != null) {
                                            updateManager.installApk(fileToInstall)
                                            onDismiss()
                                        }
                                    }
                                }
                                is DownloadProgress.Failed -> {
                                    val intent = Intent(Intent.ACTION_VIEW, Uri.parse(release.htmlUrl))
                                    context.startActivity(intent)
                                    onDismiss()
                                }
                                else -> Unit
                            }
                        },
                        enabled = progress !is DownloadProgress.Downloading && progress !is DownloadProgress.Verifying,
                        shape = RoundedCornerShape(12.dp),
                        colors = ButtonDefaults.buttonColors(
                            containerColor = MaterialTheme.colorScheme.primary,
                            contentColor = MaterialTheme.colorScheme.onPrimary,
                        ),
                        modifier = Modifier
                            .weight(1f)
                            .height(42.dp),
                    ) {
                        when (progress) {
                            is DownloadProgress.Idle -> {
                                Text(
                                    text = stringResource(R.string.update_now),
                                    fontWeight = FontWeight.Bold,
                                    fontSize = 13.sp,
                                )
                            }
                            is DownloadProgress.Downloading -> {
                                Text(
                                    text = stringResource(R.string.update_downloading),
                                    fontWeight = FontWeight.Bold,
                                    fontSize = 13.sp,
                                )
                            }
                            is DownloadProgress.Verifying -> {
                                CircularProgressIndicator(
                                    modifier = Modifier.size(16.dp),
                                    strokeWidth = 2.dp,
                                    color = MaterialTheme.colorScheme.onPrimary,
                                )
                            }
                            is DownloadProgress.Ready -> {
                                Text(
                                    text = if (!hasInstallPermission) {
                                        stringResource(R.string.update_grant_permission)
                                    } else {
                                        stringResource(R.string.update_install_now)
                                    },
                                    fontWeight = FontWeight.Bold,
                                    fontSize = 13.sp,
                                )
                            }
                            is DownloadProgress.Failed -> {
                                Text(
                                    text = stringResource(R.string.update_open_in_browser),
                                    fontWeight = FontWeight.Bold,
                                    fontSize = 13.sp,
                                )
                            }
                        }
                    }
                }
            }
        }
    }
}

@Composable
private fun MarkdownContent(
    content: String,
    modifier: Modifier = Modifier,
) {
    val primaryColor = MaterialTheme.colorScheme.primary
    val codeBackground = MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.8f)
    val codeColor = MaterialTheme.colorScheme.primary
    val lines = remember(content) { content.lines() }

    Column(
        modifier = modifier.fillMaxWidth(),
        verticalArrangement = Arrangement.spacedBy(4.dp),
    ) {
        var inCodeBlock = false
        val codeBlockLines = mutableListOf<String>()

        for (line in lines) {
            val trimmed = line.trim()
            if (trimmed.startsWith("```")) {
                if (inCodeBlock) {
                    val blockText = codeBlockLines.joinToString("\n")
                    codeBlockLines.clear()
                    Surface(
                        shape = RoundedCornerShape(8.dp),
                        color = MaterialTheme.colorScheme.surfaceVariant,
                        border = BorderStroke(1.dp, MaterialTheme.colorScheme.outlineVariant.copy(alpha = 0.3f)),
                        modifier = Modifier
                            .fillMaxWidth()
                            .padding(vertical = 4.dp),
                    ) {
                        Text(
                            text = blockText,
                            style = MaterialTheme.typography.bodySmall,
                            fontFamily = FontFamily.Monospace,
                            fontSize = 11.sp,
                            modifier = Modifier.padding(8.dp),
                        )
                    }
                    inCodeBlock = false
                } else {
                    inCodeBlock = true
                }
                continue
            }

            if (inCodeBlock) {
                codeBlockLines.add(line)
                continue
            }

            when {
                trimmed.isEmpty() -> {
                    Spacer(Modifier.height(2.dp))
                }
                trimmed.startsWith("# ") -> {
                    val headerText = trimmed.removePrefix("# ").trim()
                    Spacer(Modifier.height(4.dp))
                    Text(
                        text = buildAnnotatedMarkdown(headerText, primaryColor, codeBackground, codeColor),
                        style = MaterialTheme.typography.titleMedium,
                        fontWeight = FontWeight.Bold,
                        color = MaterialTheme.colorScheme.onSurface,
                    )
                }
                trimmed.startsWith("## ") -> {
                    val headerText = trimmed.removePrefix("## ").trim()
                    Spacer(Modifier.height(3.dp))
                    Text(
                        text = buildAnnotatedMarkdown(headerText, primaryColor, codeBackground, codeColor),
                        style = MaterialTheme.typography.titleSmall,
                        fontWeight = FontWeight.Bold,
                        color = MaterialTheme.colorScheme.primary,
                    )
                }
                trimmed.startsWith("### ") -> {
                    val headerText = trimmed.removePrefix("### ").trim()
                    Spacer(Modifier.height(2.dp))
                    Text(
                        text = buildAnnotatedMarkdown(headerText, primaryColor, codeBackground, codeColor),
                        style = MaterialTheme.typography.labelLarge,
                        fontWeight = FontWeight.Bold,
                        color = MaterialTheme.colorScheme.secondary,
                    )
                }
                trimmed.startsWith("#### ") -> {
                    val headerText = trimmed.removePrefix("#### ").trim()
                    Text(
                        text = buildAnnotatedMarkdown(headerText, primaryColor, codeBackground, codeColor),
                        style = MaterialTheme.typography.labelMedium,
                        fontWeight = FontWeight.SemiBold,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                }
                trimmed.startsWith("- ") || trimmed.startsWith("* ") || trimmed.startsWith("+ ") -> {
                    val itemText = trimmed.drop(2).trim()
                    Row(
                        modifier = Modifier
                            .fillMaxWidth()
                            .padding(vertical = 1.dp),
                        verticalAlignment = Alignment.Top,
                    ) {
                        Box(
                            modifier = Modifier
                                .padding(top = 7.dp, end = 8.dp)
                                .size(4.dp)
                                .background(primaryColor, shape = CircleShape)
                        )
                        Text(
                            text = buildAnnotatedMarkdown(itemText, primaryColor, codeBackground, codeColor),
                            style = MaterialTheme.typography.bodySmall,
                            color = MaterialTheme.colorScheme.onSurface,
                            lineHeight = 18.sp,
                        )
                    }
                }
                trimmed.matches(Regex("""^\d+\.\s+.*""")) -> {
                    val num = trimmed.substringBefore(".") + "."
                    val itemText = trimmed.substringAfter(".").trim()
                    Row(
                        modifier = Modifier
                            .fillMaxWidth()
                            .padding(vertical = 1.dp),
                        verticalAlignment = Alignment.Top,
                    ) {
                        Text(
                            text = num,
                            style = MaterialTheme.typography.labelSmall,
                            fontWeight = FontWeight.Bold,
                            color = primaryColor,
                            modifier = Modifier.padding(end = 6.dp),
                        )
                        Text(
                            text = buildAnnotatedMarkdown(itemText, primaryColor, codeBackground, codeColor),
                            style = MaterialTheme.typography.bodySmall,
                            color = MaterialTheme.colorScheme.onSurface,
                            lineHeight = 18.sp,
                        )
                    }
                }
                trimmed.startsWith("> ") -> {
                    val quoteText = trimmed.removePrefix("> ").trim()
                    Row(
                        modifier = Modifier
                            .fillMaxWidth()
                            .padding(vertical = 2.dp),
                        verticalAlignment = Alignment.CenterVertically,
                    ) {
                        Box(
                            modifier = Modifier
                                .width(3.dp)
                                .height(16.dp)
                                .background(primaryColor, shape = RoundedCornerShape(2.dp))
                        )
                        Spacer(Modifier.width(8.dp))
                        Text(
                            text = buildAnnotatedMarkdown(quoteText, primaryColor, codeBackground, codeColor),
                            style = MaterialTheme.typography.bodySmall,
                            fontStyle = FontStyle.Italic,
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                        )
                    }
                }
                trimmed == "---" || trimmed == "***" -> {
                    HorizontalDivider(
                        color = MaterialTheme.colorScheme.outlineVariant.copy(alpha = 0.2f),
                        modifier = Modifier.padding(vertical = 4.dp),
                    )
                }
                else -> {
                    Text(
                        text = buildAnnotatedMarkdown(trimmed, primaryColor, codeBackground, codeColor),
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.onSurface,
                        lineHeight = 18.sp,
                    )
                }
            }
        }
    }
}

private fun buildAnnotatedMarkdown(
    text: String,
    primaryColor: Color,
    codeBackground: Color,
    codeColor: Color,
): AnnotatedString = buildAnnotatedString {
    var index = 0
    val regex = Regex("""(`[^`]+`|\*\*[^*]+\*\*|\*[^*]+\*|\[[^\]]+\]\([^)]+\))""")
    val matches = regex.findAll(text)

    for (match in matches) {
        if (match.range.first > index) {
            append(text.substring(index, match.range.first))
        }
        val matchText = match.value
        when {
            matchText.startsWith("`") && matchText.endsWith("`") -> {
                val code = matchText.removeSurrounding("`")
                pushStyle(
                    SpanStyle(
                        fontFamily = FontFamily.Monospace,
                        background = codeBackground,
                        color = codeColor,
                        fontSize = 11.sp,
                    )
                )
                append(" $code ")
                pop()
            }
            matchText.startsWith("**") && matchText.endsWith("**") -> {
                val bold = matchText.removeSurrounding("**")
                pushStyle(SpanStyle(fontWeight = FontWeight.Bold))
                append(bold)
                pop()
            }
            matchText.startsWith("*") && matchText.endsWith("*") -> {
                val italic = matchText.removeSurrounding("*")
                pushStyle(SpanStyle(fontStyle = FontStyle.Italic))
                append(italic)
                pop()
            }
            matchText.startsWith("[") && matchText.contains("](") -> {
                val label = matchText.substringAfter("[").substringBefore("]")
                pushStyle(
                    SpanStyle(
                        color = primaryColor,
                        fontWeight = FontWeight.Medium,
                        textDecoration = TextDecoration.Underline,
                    )
                )
                append(label)
                pop()
            }
            else -> append(matchText)
        }
        index = match.range.last + 1
    }

    if (index < text.length) {
        append(text.substring(index))
    }
}
