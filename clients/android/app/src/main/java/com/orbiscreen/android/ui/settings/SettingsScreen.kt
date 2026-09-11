// Orbiscreen - SettingsScreen.kt (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen

package com.orbiscreen.android.ui.settings

import android.content.Context
import android.content.Intent
import android.graphics.BitmapFactory
import android.net.Uri
import android.os.PowerManager
import android.provider.Settings
import android.widget.Toast
import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.rounded.ArrowBack
import androidx.compose.material.icons.automirrored.rounded.OpenInNew
import androidx.compose.material.icons.rounded.BatteryAlert
import androidx.compose.material.icons.rounded.BatteryChargingFull
import androidx.compose.material.icons.rounded.BatteryFull
import androidx.compose.material.icons.rounded.BatterySaver
import androidx.compose.material.icons.rounded.ChevronRight
import androidx.compose.material.icons.rounded.Code
import androidx.compose.material.icons.rounded.Contrast
import androidx.compose.material.icons.rounded.DarkMode
import androidx.compose.material.icons.rounded.Delete
import androidx.compose.material.icons.rounded.HeadsetMic
import androidx.compose.material.icons.rounded.Info
import androidx.compose.material.icons.rounded.Language
import androidx.compose.material.icons.rounded.LightMode
import androidx.compose.material.icons.rounded.Palette
import androidx.compose.material.icons.rounded.Person
import androidx.compose.material.icons.rounded.PhoneAndroid
import androidx.compose.material.icons.rounded.Policy
import androidx.compose.material.icons.rounded.Smartphone
import androidx.compose.material.icons.rounded.Speed
import androidx.compose.material.icons.rounded.TouchApp
import androidx.compose.material.icons.rounded.Translate
import androidx.compose.material.icons.rounded.Update
import androidx.compose.material.icons.rounded.Usb
import androidx.compose.material.icons.rounded.Visibility
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.ElevatedCard
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.RadioButton
import androidx.compose.material3.Scaffold
import androidx.compose.material3.SegmentedButton
import androidx.compose.material3.SegmentedButtonDefaults
import androidx.compose.material3.SingleChoiceSegmentedButtonRow
import androidx.compose.material3.Surface
import androidx.compose.material3.Switch
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.TopAppBar
import androidx.compose.material3.TopAppBarDefaults
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.ImageBitmap
import androidx.compose.ui.graphics.asImageBitmap
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.LifecycleEventObserver
import androidx.lifecycle.compose.LocalLifecycleOwner
import com.orbiscreen.android.BuildConfig
import com.orbiscreen.android.R
import com.orbiscreen.android.data.PrefsStore
import com.orbiscreen.android.ui.theme.ActiveGreen
import com.orbiscreen.android.ui.updater.UpdateDialog
import com.orbiscreen.android.updater.ReleaseInfo
import com.orbiscreen.android.updater.UpdateManager
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import okhttp3.OkHttpClient
import okhttp3.Request
import java.util.concurrent.TimeUnit


@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun SettingsScreen(
    prefs: PrefsStore,
    onBack: () -> Unit,
) {
    val context = LocalContext.current
    val scope = rememberCoroutineScope()
    var isCheckingUpdates by remember { mutableStateOf(false) }
    var availableUpdate by remember { mutableStateOf<ReleaseInfo?>(null) }

    Scaffold(
        containerColor = MaterialTheme.colorScheme.background,
        topBar = {
            TopAppBar(
                title = {
                    Text(
                        stringResource(R.string.settings),
                        style = MaterialTheme.typography.titleLarge,
                        fontWeight = FontWeight.Bold,
                    )
                },
                navigationIcon = {
                    IconButton(onClick = onBack) {
                        Icon(
                            Icons.AutoMirrored.Rounded.ArrowBack,
                            contentDescription = stringResource(R.string.back),
                        )
                    }
                },
                colors = TopAppBarDefaults.topAppBarColors(
                    containerColor = MaterialTheme.colorScheme.background,
                ),
            )
        },
    ) { padding ->
        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(padding)
                .verticalScroll(rememberScrollState())
                .padding(horizontal = 16.dp)
                .padding(top = 8.dp, bottom = 32.dp),
            verticalArrangement = Arrangement.spacedBy(22.dp),
        ) {
            PreferenceSection(
                title = stringResource(R.string.settings_general),
                icon = Icons.Rounded.Palette,
            ) {
                var showThemeDialog by remember { mutableStateOf(false) }
                var showLangDialog by remember { mutableStateOf(false) }
                var currentLang by remember { mutableStateOf(prefs.appLanguage) }

                val themeLabel = when (prefs.themePref) {
                    PrefsStore.ThemePref.Light -> stringResource(R.string.theme_light)
                    PrefsStore.ThemePref.Dark -> stringResource(R.string.theme_dark)
                    else -> stringResource(R.string.theme_system)
                }
                val langLabel = when (currentLang) {
                    "en" -> stringResource(R.string.language_en)
                    "ar" -> stringResource(R.string.language_ar)
                    else -> stringResource(R.string.language_system)
                }

                ClickPreferenceRow(
                    title = stringResource(R.string.settings_appearance),
                    subtitle = themeLabel,
                    icon = Icons.Rounded.Contrast,
                    onClick = { showThemeDialog = true },
                )

                HorizontalDivider(
                    modifier = Modifier.padding(horizontal = 16.dp),
                    color = MaterialTheme.colorScheme.outlineVariant.copy(alpha = 0.35f),
                )

                ClickPreferenceRow(
                    title = stringResource(R.string.settings_language),
                    subtitle = langLabel,
                    icon = Icons.Rounded.Translate,
                    onClick = { showLangDialog = true },
                )

                if (showThemeDialog) {
                    val options = listOf(
                        PrefsStore.ThemePref.System to stringResource(R.string.theme_system),
                        PrefsStore.ThemePref.Light to stringResource(R.string.theme_light),
                        PrefsStore.ThemePref.Dark to stringResource(R.string.theme_dark),
                    )
                    AlertDialog(
                        onDismissRequest = { showThemeDialog = false },
                        title = { Text(stringResource(R.string.settings_appearance)) },
                        text = {
                            Column {
                                options.forEach { (pref, label) ->
                                    Row(
                                        modifier = Modifier
                                            .fillMaxWidth()
                                            .clip(RoundedCornerShape(12.dp))
                                            .clickable {
                                                prefs.themePref = pref
                                                showThemeDialog = false
                                            }
                                            .padding(horizontal = 8.dp, vertical = 10.dp),
                                        verticalAlignment = Alignment.CenterVertically,
                                    ) {
                                        RadioButton(selected = prefs.themePref == pref, onClick = {
                                            prefs.themePref = pref
                                            showThemeDialog = false
                                        })
                                        Spacer(Modifier.width(8.dp))
                                        Text(label, style = MaterialTheme.typography.bodyLarge)
                                    }
                                }
                            }
                        },
                        confirmButton = {},
                        dismissButton = {
                            TextButton(onClick = { showThemeDialog = false }) {
                                Text(stringResource(R.string.cancel))
                            }
                        },
                    )
                }

                if (showLangDialog) {
                    val langOptions = listOf(
                        "system" to stringResource(R.string.language_system),
                        "en" to stringResource(R.string.language_en),
                        "ar" to stringResource(R.string.language_ar),
                    )
                    AlertDialog(
                        onDismissRequest = { showLangDialog = false },
                        title = { Text(stringResource(R.string.settings_language)) },
                        text = {
                            Column {
                                langOptions.forEach { (key, label) ->
                                    Row(
                                        modifier = Modifier
                                            .fillMaxWidth()
                                            .clip(RoundedCornerShape(12.dp))
                                            .clickable {
                                                currentLang = key
                                                prefs.appLanguage = key
                                                showLangDialog = false
                                            }
                                            .padding(horizontal = 8.dp, vertical = 10.dp),
                                        verticalAlignment = Alignment.CenterVertically,
                                    ) {
                                        RadioButton(selected = currentLang == key, onClick = {
                                            currentLang = key
                                            prefs.appLanguage = key
                                            showLangDialog = false
                                        })
                                        Spacer(Modifier.width(8.dp))
                                        Text(label, style = MaterialTheme.typography.bodyLarge)
                                    }
                                }
                            }
                        },
                        confirmButton = {},
                        dismissButton = {
                            TextButton(onClick = { showLangDialog = false }) {
                                Text(stringResource(R.string.cancel))
                            }
                        },
                    )
                }
            }


            PreferenceSection(
                title = stringResource(R.string.settings_display_session),
                icon = Icons.Rounded.Visibility,
            ) {
                var keepScreen by remember { mutableStateOf(prefs.keepScreenAwake) }
                SwitchPreferenceRow(
                    title = stringResource(R.string.keep_screen_awake),
                    subtitle = stringResource(R.string.keep_screen_awake_desc),
                    checked = keepScreen,
                    icon = Icons.Rounded.PhoneAndroid,
                    onCheckedChange = {
                        keepScreen = it
                        prefs.keepScreenAwake = it
                    },
                )

                HorizontalDivider(
                    modifier = Modifier.padding(horizontal = 16.dp),
                    color = MaterialTheme.colorScheme.outlineVariant.copy(alpha = 0.35f),
                )

                val powerManager = remember { context.getSystemService(Context.POWER_SERVICE) as? PowerManager }
                var isIgnoringBattery by remember {
                    mutableStateOf(powerManager?.isIgnoringBatteryOptimizations(context.packageName) == true)
                }
                val lifecycleOwner = LocalLifecycleOwner.current
                DisposableEffect(lifecycleOwner) {
                    val observer = LifecycleEventObserver { _, event ->
                        if (event == Lifecycle.Event.ON_RESUME) {
                            isIgnoringBattery = powerManager?.isIgnoringBatteryOptimizations(context.packageName) == true
                        }
                    }
                    lifecycleOwner.lifecycle.addObserver(observer)
                    onDispose {
                        lifecycleOwner.lifecycle.removeObserver(observer)
                    }
                }

                SwitchPreferenceRow(
                    title = stringResource(R.string.battery_optimization_title),
                    subtitle = if (isIgnoringBattery) {
                        stringResource(R.string.battery_optimization_unrestricted)
                    } else {
                        stringResource(R.string.battery_optimization_restricted)
                    },
                    checked = isIgnoringBattery,
                    icon = if (isIgnoringBattery) Icons.Rounded.BatteryChargingFull else Icons.Rounded.BatterySaver,
                    onCheckedChange = { enable ->
                        if (enable) {
                            try {
                                val intent = Intent(Settings.ACTION_REQUEST_IGNORE_BATTERY_OPTIMIZATIONS).apply {
                                    data = Uri.parse("package:${context.packageName}")
                                }
                                context.startActivity(intent)
                            } catch (_: Exception) {
                                try {
                                    val intent = Intent(Settings.ACTION_IGNORE_BATTERY_OPTIMIZATION_SETTINGS)
                                    context.startActivity(intent)
                                } catch (_: Exception) {}
                            }
                        } else {
                            try {
                                val intent = Intent(Settings.ACTION_IGNORE_BATTERY_OPTIMIZATION_SETTINGS)
                                context.startActivity(intent)
                            } catch (_: Exception) {}
                        }
                    },
                )

                HorizontalDivider(
                    modifier = Modifier.padding(horizontal = 16.dp),
                    color = MaterialTheme.colorScheme.outlineVariant.copy(alpha = 0.35f),
                )

                var touchMode by remember { mutableStateOf(prefs.touchMode) }
                val inputOptions = listOf(
                    stringResource(R.string.mode_touch),
                    stringResource(R.string.mode_trackpad),
                )
                Row(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(horizontal = 16.dp, vertical = 12.dp),
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    Surface(
                        shape = RoundedCornerShape(12.dp),
                        color = MaterialTheme.colorScheme.primaryContainer,
                        modifier = Modifier.size(38.dp),
                    ) {
                        Box(contentAlignment = Alignment.Center) {
                            Icon(
                                Icons.Rounded.TouchApp,
                                contentDescription = null,
                                tint = MaterialTheme.colorScheme.primary,
                                modifier = Modifier.size(20.dp),
                            )
                        }
                    }
                    Spacer(Modifier.width(16.dp))
                    Column(Modifier.weight(1f)) {
                        Text(
                            text = stringResource(R.string.default_input_mode),
                            style = MaterialTheme.typography.bodyLarge,
                            fontWeight = FontWeight.SemiBold,
                        )
                        Spacer(Modifier.height(8.dp))
                        SingleChoiceSegmentedButtonRow(modifier = Modifier.fillMaxWidth()) {
                            inputOptions.forEachIndexed { index, label ->
                                SegmentedButton(
                                    selected = if (index == 0) touchMode else !touchMode,
                                    onClick = {
                                        touchMode = index == 0
                                        prefs.touchMode = index == 0
                                    },
                                    shape = SegmentedButtonDefaults.itemShape(
                                        index = index,
                                        count = inputOptions.size,
                                    ),
                                ) {
                                    Text(label, style = MaterialTheme.typography.labelMedium)
                                }
                            }
                        }
                    }
                }
            }

            PreferenceSection(
                title = stringResource(R.string.settings_audio_connection),
                icon = Icons.Rounded.Usb,
            ) {
                var autoConnect by remember { mutableStateOf(prefs.autoConnectUsb) }
                SwitchPreferenceRow(
                    title = stringResource(R.string.auto_connect_usb_title),
                    subtitle = stringResource(R.string.auto_connect_usb_desc),
                    checked = autoConnect,
                    icon = Icons.Rounded.Usb,
                    onCheckedChange = {
                        autoConnect = it
                        prefs.autoConnectUsb = it
                    },
                )
            }


            PreferenceSection(
                title = stringResource(R.string.about),
                icon = Icons.Rounded.Info,
            ) {
                ProjectCardContent(
                    isCheckingUpdates = isCheckingUpdates,
                    onCheckUpdates = {
                        scope.launch {
                            isCheckingUpdates = true
                            val release = try {
                                UpdateManager(context).checkForUpdates()
                            } catch (_: Exception) {
                                null
                            }
                            isCheckingUpdates = false
                            if (release != null) {
                                availableUpdate = release
                            } else {
                                Toast.makeText(
                                    context,
                                    context.getString(R.string.latest_version_installed),
                                    Toast.LENGTH_SHORT,
                                ).show()
                            }
                        }
                    },
                    onOpenRepo = {
                        val intent = Intent(Intent.ACTION_VIEW, Uri.parse("https://github.com/shadow-x78/orbiscreen"))
                        context.startActivity(intent)
                    },
                    onOpenLicense = {
                        val intent = Intent(Intent.ACTION_VIEW, Uri.parse("https://github.com/shadow-x78/orbiscreen/blob/main/LICENSE"))
                        context.startActivity(intent)
                    },
                )

                var recentHost by remember { mutableStateOf(prefs.recentHost) }
                if (recentHost != null) {
                    HorizontalDivider(
                        modifier = Modifier.padding(horizontal = 16.dp),
                        color = MaterialTheme.colorScheme.outlineVariant.copy(alpha = 0.35f),
                    )
                    Row(
                        modifier = Modifier
                            .fillMaxWidth()
                            .padding(horizontal = 4.dp, vertical = 2.dp)
                            .clip(RoundedCornerShape(16.dp))
                            .clickable {
                                prefs.recentHost = null
                                recentHost = null
                                Toast.makeText(context, context.getString(R.string.clear_history), Toast.LENGTH_SHORT).show()
                            }
                            .padding(horizontal = 12.dp, vertical = 12.dp),
                        verticalAlignment = Alignment.CenterVertically,
                    ) {
                        Surface(
                            shape = RoundedCornerShape(12.dp),
                            color = MaterialTheme.colorScheme.errorContainer.copy(alpha = 0.35f),
                            modifier = Modifier.size(38.dp),
                        ) {
                            Box(contentAlignment = Alignment.Center) {
                                Icon(
                                    Icons.Rounded.Delete,
                                    contentDescription = null,
                                    tint = MaterialTheme.colorScheme.error,
                                    modifier = Modifier.size(20.dp),
                                )
                            }
                        }
                        Spacer(Modifier.width(16.dp))
                        Column(Modifier.weight(1f).padding(end = 8.dp)) {
                            Text(
                                text = stringResource(R.string.clear_history),
                                style = MaterialTheme.typography.titleSmall,
                                fontWeight = FontWeight.SemiBold,
                                color = MaterialTheme.colorScheme.error,
                            )
                            Spacer(Modifier.height(2.dp))
                            Text(
                                text = stringResource(R.string.clear_history_desc),
                                style = MaterialTheme.typography.bodySmall,
                                color = MaterialTheme.colorScheme.onSurfaceVariant,
                            )
                        }
                    }
                }
            }

            DeveloperCard(
                onOpenProfile = {
                    val intent = Intent(Intent.ACTION_VIEW, Uri.parse("https://github.com/shadow-x78"))
                    context.startActivity(intent)
                },
            )
        }
    }

    availableUpdate?.let { release ->
        UpdateDialog(
            release = release,
            onDismiss = { availableUpdate = null },
        )
    }
}

@Composable
private fun PreferenceSection(
    title: String,
    icon: ImageVector,
    betaBadge: Boolean = false,
    content: @Composable () -> Unit,
) {
    Column(verticalArrangement = Arrangement.spacedBy(10.dp)) {
        Row(
            verticalAlignment = Alignment.CenterVertically,
            modifier = Modifier.padding(start = 4.dp),
        ) {
            Icon(
                imageVector = icon,
                contentDescription = null,
                tint = MaterialTheme.colorScheme.primary,
                modifier = Modifier.size(16.dp),
            )
            Spacer(Modifier.width(8.dp))
            Text(
                text = title,
                style = MaterialTheme.typography.titleSmall,
                color = MaterialTheme.colorScheme.primary,
                fontWeight = FontWeight.Bold,
            )
            if (betaBadge) {
                Spacer(Modifier.width(8.dp))
                Surface(
                    shape = RoundedCornerShape(50),
                    color = MaterialTheme.colorScheme.tertiaryContainer,
                ) {
                    Text(
                        text = "BETA",
                        style = MaterialTheme.typography.labelSmall,
                        color = MaterialTheme.colorScheme.onTertiaryContainer,
                        fontWeight = FontWeight.Bold,
                        fontSize = 9.sp,
                        modifier = Modifier.padding(horizontal = 6.dp, vertical = 2.dp),
                    )
                }
            }
        }
        ElevatedCard(
            modifier = Modifier
                .fillMaxWidth()
                .clip(RoundedCornerShape(22.dp)),
            shape = RoundedCornerShape(22.dp),
            colors = CardDefaults.elevatedCardColors(
                containerColor = MaterialTheme.colorScheme.surface,
            ),
            elevation = CardDefaults.elevatedCardElevation(defaultElevation = 2.dp),
        ) {
            Column {
                content()
            }
        }
    }
}

@Composable
private fun ClickPreferenceRow(
    title: String,
    subtitle: String,
    icon: ImageVector,
    onClick: () -> Unit,
    trailing: @Composable (() -> Unit)? = null,
) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .padding(horizontal = 4.dp, vertical = 2.dp)
            .clip(RoundedCornerShape(16.dp))
            .clickable(onClick = onClick)
            .padding(horizontal = 12.dp, vertical = 13.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Surface(
            shape = RoundedCornerShape(12.dp),
            color = MaterialTheme.colorScheme.primaryContainer,
            modifier = Modifier.size(38.dp),
        ) {
            Box(contentAlignment = Alignment.Center) {
                Icon(
                    imageVector = icon,
                    contentDescription = null,
                    tint = MaterialTheme.colorScheme.primary,
                    modifier = Modifier.size(20.dp),
                )
            }
        }
        Spacer(Modifier.width(16.dp))
        Column(
            modifier = Modifier
                .weight(1f)
                .padding(end = 12.dp),
        ) {
            Text(
                text = title,
                style = MaterialTheme.typography.bodyLarge,
                fontWeight = FontWeight.SemiBold,
            )
            Spacer(Modifier.height(2.dp))
            Text(
                text = subtitle,
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
        }
        if (trailing != null) {
            trailing()
        } else {
            Icon(
                imageVector = Icons.Rounded.ChevronRight,
                contentDescription = null,
                tint = MaterialTheme.colorScheme.onSurfaceVariant,
                modifier = Modifier.size(20.dp),
            )
        }
    }
}

@Composable
private fun SwitchPreferenceRow(
    title: String,
    subtitle: String,
    checked: Boolean,
    icon: ImageVector,
    onCheckedChange: (Boolean) -> Unit,
) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .padding(horizontal = 4.dp, vertical = 2.dp)
            .clip(RoundedCornerShape(16.dp))
            .clickable { onCheckedChange(!checked) }
            .padding(horizontal = 12.dp, vertical = 13.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Surface(
            shape = RoundedCornerShape(12.dp),
            color = if (checked) MaterialTheme.colorScheme.primaryContainer else MaterialTheme.colorScheme.surfaceVariant,
            modifier = Modifier.size(38.dp),
        ) {
            Box(contentAlignment = Alignment.Center) {
                Icon(
                    imageVector = icon,
                    contentDescription = null,
                    tint = if (checked) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.onSurfaceVariant,
                    modifier = Modifier.size(20.dp),
                )
            }
        }
        Spacer(Modifier.width(16.dp))
        Column(
            modifier = Modifier
                .weight(1f)
                .padding(end = 12.dp),
        ) {
            Text(
                text = title,
                style = MaterialTheme.typography.bodyLarge,
                fontWeight = FontWeight.SemiBold,
            )
            Spacer(Modifier.height(2.dp))
            Text(
                text = subtitle,
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
        }
        Switch(
            checked = checked,
            onCheckedChange = onCheckedChange,
        )
    }
}

@Composable
private fun DeveloperCard(onOpenProfile: () -> Unit) {
    var avatarBitmap by remember { mutableStateOf<ImageBitmap?>(null) }

    LaunchedEffect(Unit) {
        withContext(Dispatchers.IO) {
            try {
                val client = OkHttpClient.Builder()
                    .connectTimeout(5, TimeUnit.SECONDS)
                    .readTimeout(5, TimeUnit.SECONDS)
                    .build()
                val req = Request.Builder()
                    .url("https://github.com/shadow-x78.png")
                    .header("User-Agent", "Orbiscreen-Android")
                    .build()
                val res = client.newCall(req).execute()
                if (res.isSuccessful) {
                    res.body?.byteStream()?.use { stream ->
                        val bmp = BitmapFactory.decodeStream(stream)
                        bmp?.let { avatarBitmap = it.asImageBitmap() }
                    }
                }
            } catch (_: Exception) {}
        }
    }

    ElevatedCard(
        modifier = Modifier.fillMaxWidth(),
        shape = RoundedCornerShape(22.dp),
        colors = CardDefaults.elevatedCardColors(
            containerColor = MaterialTheme.colorScheme.surface,
        ),
        elevation = CardDefaults.elevatedCardElevation(defaultElevation = 2.dp),
    ) {
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .padding(18.dp),
        ) {
            Row(
                verticalAlignment = Alignment.CenterVertically,
                modifier = Modifier.fillMaxWidth(),
            ) {
                Box(
                    modifier = Modifier
                        .size(56.dp)
                        .clip(CircleShape)
                        .background(MaterialTheme.colorScheme.primaryContainer),
                    contentAlignment = Alignment.Center,
                ) {
                    val bmp = avatarBitmap
                    if (bmp != null) {
                        Image(
                            bitmap = bmp,
                            contentDescription = "shadow-x78",
                            modifier = Modifier.fillMaxSize().clip(CircleShape),
                            contentScale = ContentScale.Crop,
                        )
                    } else {
                        Icon(
                            Icons.Rounded.Person,
                            contentDescription = null,
                            tint = MaterialTheme.colorScheme.primary,
                            modifier = Modifier.size(30.dp),
                        )
                    }
                }
                Spacer(Modifier.width(16.dp))
                Column(Modifier.weight(1f)) {
                    Text(
                        text = stringResource(R.string.developer_tagline),
                        style = MaterialTheme.typography.labelMedium,
                        color = MaterialTheme.colorScheme.primary,
                        fontWeight = FontWeight.Bold,
                    )
                    Spacer(Modifier.height(2.dp))
                    Text(
                        text = stringResource(R.string.developer_name),
                        style = MaterialTheme.typography.titleLarge,
                        fontWeight = FontWeight.Bold,
                    )
                    Spacer(Modifier.height(2.dp))
                    Text(
                        text = "github.com/shadow-x78",
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                }
            }

            Spacer(Modifier.height(14.dp))

            Surface(
                onClick = onOpenProfile,
                shape = RoundedCornerShape(12.dp),
                color = MaterialTheme.colorScheme.primaryContainer.copy(alpha = 0.5f),
                border = BorderStroke(1.dp, MaterialTheme.colorScheme.primary.copy(alpha = 0.3f)),
                modifier = Modifier.fillMaxWidth(),
            ) {
                Row(
                    modifier = Modifier.padding(horizontal = 14.dp, vertical = 10.dp),
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.Center,
                ) {
                    Icon(
                        Icons.AutoMirrored.Rounded.OpenInNew,
                        contentDescription = null,
                        tint = MaterialTheme.colorScheme.primary,
                        modifier = Modifier.size(16.dp),
                    )
                    Spacer(Modifier.width(8.dp))
                    Text(
                        text = stringResource(R.string.view_github_profile),
                        style = MaterialTheme.typography.labelLarge,
                        fontWeight = FontWeight.Bold,
                        color = MaterialTheme.colorScheme.primary,
                    )
                }
            }
        }
    }
}

@Composable
private fun ProjectCardContent(
    isCheckingUpdates: Boolean,
    onCheckUpdates: () -> Unit,
    onOpenRepo: () -> Unit,
    onOpenLicense: () -> Unit,
) {
    Column(
        modifier = Modifier
            .fillMaxWidth()
            .padding(18.dp),
    ) {
        Row(
            verticalAlignment = Alignment.CenterVertically,
            modifier = Modifier.fillMaxWidth(),
        ) {
            Icon(
                painter = painterResource(R.drawable.ic_orbiscreen_logo),
                contentDescription = null,
                tint = MaterialTheme.colorScheme.primary,
                modifier = Modifier.size(46.dp),
            )
            Spacer(Modifier.width(16.dp))
            Column(Modifier.weight(1f)) {
                Text(
                    text = "Orbiscreen",
                    style = MaterialTheme.typography.titleLarge,
                    fontWeight = FontWeight.Bold,
                )
                Spacer(Modifier.height(2.dp))
                Text(
                    text = "v${BuildConfig.VERSION_NAME} (Build ${BuildConfig.VERSION_CODE})",
                    style = MaterialTheme.typography.bodyMedium,
                    fontWeight = FontWeight.SemiBold,
                    color = MaterialTheme.colorScheme.primary,
                )
            }
            Surface(
                onClick = onOpenLicense,
                shape = RoundedCornerShape(8.dp),
                color = MaterialTheme.colorScheme.surfaceVariant,
            ) {
                Text(
                    text = stringResource(R.string.license_chip),
                    style = MaterialTheme.typography.labelSmall,
                    fontWeight = FontWeight.Bold,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    modifier = Modifier.padding(horizontal = 8.dp, vertical = 5.dp),
                )
            }
        }

        Spacer(Modifier.height(16.dp))

        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.spacedBy(10.dp),
        ) {
            Surface(
                onClick = onOpenRepo,
                shape = RoundedCornerShape(12.dp),
                color = MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.7f),
                modifier = Modifier.weight(1f).height(44.dp),
            ) {
                Row(
                    modifier = Modifier.fillMaxSize().padding(horizontal = 8.dp),
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.Center,
                ) {
                    Icon(
                        Icons.Rounded.Code,
                        contentDescription = null,
                        tint = MaterialTheme.colorScheme.onSurfaceVariant,
                        modifier = Modifier.size(18.dp),
                    )
                    Spacer(Modifier.width(6.dp))
                    Text(
                        text = stringResource(R.string.view_repository),
                        style = MaterialTheme.typography.labelMedium,
                        fontWeight = FontWeight.Bold,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                }
            }

            Surface(
                onClick = onCheckUpdates,
                shape = RoundedCornerShape(12.dp),
                color = MaterialTheme.colorScheme.primary,
                modifier = Modifier.weight(1.2f).height(44.dp),
            ) {
                Row(
                    modifier = Modifier.fillMaxSize().padding(horizontal = 8.dp),
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.Center,
                ) {
                    if (isCheckingUpdates) {
                        CircularProgressIndicator(
                            modifier = Modifier.size(16.dp),
                            strokeWidth = 2.dp,
                            color = MaterialTheme.colorScheme.onPrimary,
                        )
                    } else {
                        Icon(
                            Icons.Rounded.Update,
                            contentDescription = null,
                            tint = MaterialTheme.colorScheme.onPrimary,
                            modifier = Modifier.size(18.dp),
                        )
                    }
                    Spacer(Modifier.width(6.dp))
                    Text(
                        text = if (isCheckingUpdates) stringResource(R.string.checking_updates) else stringResource(R.string.check_for_updates),
                        style = MaterialTheme.typography.labelMedium,
                        fontWeight = FontWeight.Bold,
                        color = MaterialTheme.colorScheme.onPrimary,
                    )
                }
            }
        }
    }
}
