package com.orbiscreen.android.net

import android.content.Context
import android.net.ConnectivityManager
import java.net.Inet4Address

object WifiGatewayProvider {
    fun gateway(context: Context): String? {
        val cm = context.applicationContext
            .getSystemService(Context.CONNECTIVITY_SERVICE) as? ConnectivityManager
            ?: return null
        val props = cm.getLinkProperties(cm.activeNetwork) ?: return null
        val gateway = props.routes
            .firstOrNull { it.isDefaultRoute && it.gateway is Inet4Address }
            ?.gateway
            ?: props.routes.firstOrNull { it.gateway is Inet4Address }?.gateway
            ?: return null
        return gateway.hostAddress
    }
}
