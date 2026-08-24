
package com.orbiscreen.android.ui.settings

import android.content.ClipData
import android.content.ClipboardManager
import android.content.Context
import android.os.Build
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.ArrowBack
import androidx.compose.material.icons.filled.Brightness4
import androidx.compose.material.icons.filled.Brightness5
import androidx.compose.material.icons.filled.Brightness6
import androidx.compose.material.icons.filled.ChevronRight
import androidx.compose.material.icons.filled.Code
import androidx.compose.material.icons.filled.Delete
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.RadioButton
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Surface
import androidx.compose.material3.Switch
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import com.orbiscreen.android.BuildConfig
import com.orbiscreen.android.R
import com.orbiscreen.android.data.PrefsStore
import com.orbiscreen.android.ui.theme.ThemeMode

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun SettingsScreen(
    prefs: PrefsStore,
    onBack: () -> Unit,
) {
    val context = LocalContext.current
    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text(stringResource(R.string.settings), fontWeight = FontWeight.SemiBold) },
                navigationIcon = {
                    IconButton(onClick = onBack) {
                        Icon(Icons.Filled.ArrowBack, contentDescription = stringResource(R.string.back))
                    }
                },
            )
        }
    ) { padding ->
        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(padding)
                .padding(horizontal = 16.dp),
            verticalArrangement = Arrangement.spacedBy(16.dp),
        ) {
            SectionCard(title = stringResource(R.string.theme_system).let { "Appearance" }) {
                ThemeRow(stringResource(R.string.theme_system), ThemeMode.System, prefs)
                HorizontalDivider()
                ThemeRow(stringResource(R.string.theme_light), ThemeMode.Light, prefs)
                HorizontalDivider()
                ThemeRow(stringResource(R.string.theme_dark), ThemeMode.Dark, prefs)
            }
            SectionCard(title = "Streaming") {
                SwitchRow(
                    title = "Force software decoder",
                    subtitle = "Enable on devices that struggle with hardware H.264",
                    checked = prefs.forceSoftwareDecoder,
                    onChange = { prefs.forceSoftwareDecoder = it },
                )
                HorizontalDivider()
                SwitchRow(
                    title = stringResource(R.string.enable_subnet_scanner),
                    subtitle = "Active scan of nearby hosts (may trigger network alarms)",
                    checked = prefs.enableSubnetScanner,
                    onChange = { prefs.enableSubnetScanner = it },
                )
            }
            SectionCard(title = "Recent host") {
                val recent = prefs.recentHost
                if (recent == null) {
                    Text("None yet", color = MaterialTheme.colorScheme.onSurfaceVariant, modifier = Modifier.padding(16.dp))
                } else {
                    Row(
                        Modifier
                            .fillMaxWidth()
                            .padding(16.dp),
                        verticalAlignment = Alignment.CenterVertically,
                    ) {
                        Column(Modifier.weight(1f)) {
                            Text("${recent.host}:${recent.port}", style = MaterialTheme.typography.titleMedium)
                            Text(recent.label ?: "Recent connection", style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.onSurfaceVariant)
                        }
                        IconButton(onClick = {
                            prefs.clearRecent()
                        }) {
                            Icon(Icons.Filled.Delete, contentDescription = stringResource(R.string.clear_recent))
                        }
                    }
                }
            }
            SectionCard(title = stringResource(R.string.about)) {
                Row(
                    Modifier
                        .fillMaxWidth()
                        .padding(16.dp),
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    Column(Modifier.weight(1f)) {
                        Text("Orbiscreen", style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.SemiBold)
                        Text("v${BuildConfig.VERSION_NAME} (${BuildConfig.VERSION_CODE}) · SDK ${Build.VERSION.SDK_INT}", style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.onSurfaceVariant)
                    }
                    IconButton(onClick = {
                        val cm = context.getSystemService(Context.CLIPBOARD_SERVICE) as ClipboardManager
                        cm.setPrimaryClip(ClipData.newPlainText("Orbiscreen Version", "v${BuildConfig.VERSION_NAME} (${BuildConfig.VERSION_CODE})"))
                    }) {
                        Icon(Icons.Filled.Code, contentDescription = null)
                    }
                }
            }
        }
    }
}

@Composable
private fun SectionCard(title: String, content: @Composable () -> Unit) {
    Card(
        modifier = Modifier.fillMaxWidth(),
        shape = RoundedCornerShape(20.dp),
        colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.surface),
    ) {
        Column {
            Text(
                text = title,
                style = MaterialTheme.typography.labelLarge,
                color = MaterialTheme.colorScheme.primary,
                modifier = Modifier.padding(start = 16.dp, top = 14.dp),
            )
            content()
        }
    }
}

@Composable
private fun ThemeRow(label: String, mode: ThemeMode, prefs: PrefsStore) {
    Row(
        Modifier
            .fillMaxWidth()
            .clickable {
                prefs.themePref = when (mode) {
                    ThemeMode.System -> PrefsStore.ThemePref.System
                    ThemeMode.Light -> PrefsStore.ThemePref.Light
                    ThemeMode.Dark -> PrefsStore.ThemePref.Dark
                }
            }
            .padding(horizontal = 16.dp, vertical = 14.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        RadioButton(
            selected = prefs.themePref == when (mode) {
                ThemeMode.System -> PrefsStore.ThemePref.System
                ThemeMode.Light -> PrefsStore.ThemePref.Light
                ThemeMode.Dark -> PrefsStore.ThemePref.Dark
            },
            onClick = null,
        )
        Spacer(Modifier.size(8.dp))
        Icon(
            imageVector = when (mode) {
                ThemeMode.System -> Icons.Filled.Brightness6
                ThemeMode.Light -> Icons.Filled.Brightness5
                ThemeMode.Dark -> Icons.Filled.Brightness4
            },
            contentDescription = null,
            tint = MaterialTheme.colorScheme.primary,
        )
        Spacer(Modifier.size(12.dp))
        Text(label, style = MaterialTheme.typography.bodyLarge)
    }
}

@Composable
private fun SwitchRow(title: String, subtitle: String, checked: Boolean, onChange: (Boolean) -> Unit) {
    Row(
        Modifier
            .fillMaxWidth()
            .clickable { onChange(!checked) }
            .padding(horizontal = 16.dp, vertical = 14.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Column(Modifier.weight(1f)) {
            Text(title, style = MaterialTheme.typography.bodyLarge)
            Text(subtitle, style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.onSurfaceVariant)
        }
        Switch(checked = checked, onCheckedChange = onChange)
    }
}
