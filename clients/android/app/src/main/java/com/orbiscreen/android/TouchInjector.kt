package com.orbiscreen.android

import android.view.MotionEvent
import android.view.View
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import org.json.JSONObject
import java.net.HttpURLConnection
import java.net.URL
import kotlin.math.abs

class TouchInjector(private val host: String) : View.OnTouchListener {
    private var lastX = 0f
    private var lastY = 0f

    override fun onTouch(v: View, event: MotionEvent): Boolean {
        when (event.actionMasked) {
            MotionEvent.ACTION_DOWN -> {
                lastX = event.x
                lastY = event.y
            }
            MotionEvent.ACTION_MOVE -> {
                val deltaX = event.x - lastX
                val deltaY = event.y - lastY
                if (abs(deltaX) > 1f || abs(deltaY) > 1f) {
                    sendMove(deltaX, deltaY)
                    lastX = event.x
                    lastY = event.y
                }
            }
            MotionEvent.ACTION_UP -> {
                val duration = event.eventTime - event.downTime
                if (duration < 200) {
                    sendClick()
                }
            }
        }
        return true
    }

    private fun sendMove(dx: Float, dy: Float) {
        val payload = JSONObject()
        val moveObj = JSONObject()
        moveObj.put("x", dx.toDouble())
        moveObj.put("y", dy.toDouble())
        payload.put("Move", moveObj)
        sendPayload(payload)
    }

    private fun sendClick() {
        CoroutineScope(Dispatchers.IO).launch {
            try {
                val downPayload = JSONObject().apply {
                    put("Button", JSONObject().apply {
                        put("button", 272)
                        put("pressed", true)
                    })
                }
                postPayload(downPayload)
                
                Thread.sleep(50)
                
                val upPayload = JSONObject().apply {
                    put("Button", JSONObject().apply {
                        put("button", 272)
                        put("pressed", false)
                    })
                }
                postPayload(upPayload)
            } catch (e: Exception) {
                e.printStackTrace()
            }
        }
    }

    private fun sendPayload(payload: JSONObject) {
        CoroutineScope(Dispatchers.IO).launch {
            try {
                postPayload(payload)
            } catch (e: Exception) {
                e.printStackTrace()
            }
        }
    }

    private fun postPayload(payload: JSONObject) {
        val url = URL("http://$host:8788/input")
        val conn = url.openConnection() as HttpURLConnection
        conn.requestMethod = "POST"
        conn.setRequestProperty("Content-Type", "application/json")
        conn.doOutput = true
        
        conn.outputStream.use { os ->
            val input = payload.toString().toByteArray(Charsets.UTF_8)
            os.write(input, 0, input.size)
        }
        
        conn.responseCode // Read response to execute request
        conn.disconnect()
    }
}
