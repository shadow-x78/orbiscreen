
package com.orbiscreen.android.player

import android.view.Surface

interface SurfaceTarget {
    fun attachSurface(surface: Surface)
    fun detachSurface()
}
