package com.orbiscreen.android.ui.stream

import androidx.activity.compose.BackHandler
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.text.selection.SelectionContainer
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Button
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.unit.dp

@Composable
fun LanLoginScreen(viewModel: StreamViewModel, login: LoginState, onBack: () -> Unit) {
    val cancel = { viewModel.disconnect(); onBack() }
    BackHandler(onBack = cancel)
    Surface(modifier = Modifier.fillMaxSize()) {
        Column(
            modifier = Modifier.fillMaxSize().verticalScroll(rememberScrollState()).padding(24.dp),
            verticalArrangement = Arrangement.spacedBy(16.dp),
        ) {
            Text("Host login", style = MaterialTheme.typography.headlineMedium)
            Text("${viewModel.host}:${viewModel.port}")
            when (login.stage) {
                LoginStage.Checking, LoginStage.Pending -> {
                    CircularProgressIndicator()
                    Text(login.message)
                    if (login.stage == LoginStage.Pending) {
                        Text("Keep this screen open. Approval expires after a few minutes. Login and streaming use encrypted HTTPS; USB loopback is unchanged.")
                    }
                }
                LoginStage.ConfirmFingerprint -> {
                    Text("Verify the host certificate", style = MaterialTheme.typography.titleLarge)
                    Text("Compare this SHA-256 fingerprint with the fingerprint shown locally on your host. Do not approve unless every character matches. This explicitly trusts this certificate for this host address, including its hostname identity.")
                    SelectionContainer {
                        Text(login.fingerprint.chunked(2).joinToString(":"), fontFamily = FontFamily.Monospace)
                    }
                    Button(onClick = viewModel::confirmFingerprint, modifier = Modifier.fillMaxWidth()) {
                        Text("Fingerprint matches — request approval")
                    }
                }
                LoginStage.Failed -> {
                    Text(login.message, color = MaterialTheme.colorScheme.error)
                    Button(onClick = viewModel::beginLogin) { Text("Retry secure login") }
                    TextButton(onClick = viewModel::forgetLogin) { Text("Forget saved login and verify again") }
                    Text("Forgetting removes this device's saved credential and certificate. It does not revoke a credential on the host; use the host's paired-client controls for revocation.")
                }
                LoginStage.Ready -> Unit
            }
            TextButton(onClick = cancel) { Text("Cancel and return to hosts") }
        }
    }
}
