package com.orbiscreen.android.data

import android.content.Context
import android.security.keystore.KeyGenParameterSpec
import android.security.keystore.KeyProperties
import android.util.Base64
import com.orbiscreen.android.net.HostApi
import com.orbiscreen.android.net.HostSpec
import org.json.JSONObject
import java.security.KeyStore
import javax.crypto.Cipher
import javax.crypto.KeyGenerator
import javax.crypto.SecretKey
import javax.crypto.spec.GCMParameterSpec

data class HostCredential(val fingerprint: String, val credential: String)

class HostCredentialStore(context: Context) {
    private val prefs = context.getSharedPreferences("host_credentials", Context.MODE_PRIVATE)

    private fun endpoint(host: String, port: Int) = HostSpec(host, port).pairingUrl().toString()

    @Synchronized
    fun load(host: String, port: Int): HostCredential? {
        val endpoint = endpoint(host, port)
        val encoded = prefs.getString(HostApi.fingerprint(endpoint.toByteArray()), null) ?: return null
        val bytes = Base64.decode(encoded, Base64.NO_WRAP)
        val key = key(false)
        val json = JSONObject(String(CredentialCipher.decrypt(key, endpoint, bytes), Charsets.UTF_8))
        return HostCredential(json.getString("fingerprint"), json.getString("credential"))
    }

    @Synchronized
    fun save(host: String, port: Int, value: HostCredential) {
        require(value.credential.isNotBlank())
        val endpoint = endpoint(host, port)
        val json = JSONObject().put("fingerprint", value.fingerprint).put("credential", value.credential)
        val encrypted = CredentialCipher.encrypt(key(true), endpoint, json.toString().toByteArray())
        check(prefs.edit().putString(HostApi.fingerprint(endpoint.toByteArray()),
            Base64.encodeToString(encrypted, Base64.NO_WRAP)).commit()) { "Could not save host login" }
    }

    @Synchronized
    fun forget(host: String, port: Int) {
        check(prefs.edit().remove(HostApi.fingerprint(endpoint(host, port).toByteArray())).commit())
    }

    private fun key(create: Boolean): SecretKey {
        val store = KeyStore.getInstance("AndroidKeyStore").apply { load(null) }
        (store.getKey(ALIAS, null) as? SecretKey)?.let { return it }
        check(create) { "Saved login key unavailable; forget this host and pair again" }
        return KeyGenerator.getInstance(KeyProperties.KEY_ALGORITHM_AES, "AndroidKeyStore").apply {
            init(KeyGenParameterSpec.Builder(ALIAS, KeyProperties.PURPOSE_ENCRYPT or KeyProperties.PURPOSE_DECRYPT)
                .setBlockModes(KeyProperties.BLOCK_MODE_GCM)
                .setEncryptionPaddings(KeyProperties.ENCRYPTION_PADDING_NONE)
                .setKeySize(256).build())
        }.generateKey()
    }

    companion object {
        private const val ALIAS = "orbiscreen.host.credentials.v1"
    }
}

internal object CredentialCipher {
    fun encrypt(key: SecretKey, endpoint: String, plaintext: ByteArray): ByteArray {
        val cipher = Cipher.getInstance("AES/GCM/NoPadding")
        cipher.init(Cipher.ENCRYPT_MODE, key)
        cipher.updateAAD(endpoint.toByteArray(Charsets.UTF_8))
        check(cipher.iv.size == 12)
        return cipher.iv + cipher.doFinal(plaintext)
    }

    fun decrypt(key: SecretKey, endpoint: String, ciphertext: ByteArray): ByteArray {
        require(ciphertext.size >= 28)
        val cipher = Cipher.getInstance("AES/GCM/NoPadding")
        cipher.init(Cipher.DECRYPT_MODE, key, GCMParameterSpec(128, ciphertext.copyOfRange(0, 12)))
        cipher.updateAAD(endpoint.toByteArray(Charsets.UTF_8))
        return cipher.doFinal(ciphertext.copyOfRange(12, ciphertext.size))
    }
}
