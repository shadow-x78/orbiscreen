package com.orbiscreen.android.net

import android.content.Context
import android.net.wifi.WifiManager
import android.text.format.Formatter

object WifiGatewayProvider {
    fun gateway(context: Context): String? {
        val wm = context.applicationContext.getSystemService(Context.WIFI_SERVICE) as? WifiManager
            ?: return null
        @Suppress("DEPRECATION")
        val dhcp = wm.dhcpInfo
        if (dhcp == null || dhcp.gateway == 0) return null
        return Formatter.formatIpAddress(dhcp.gateway)
    }
}