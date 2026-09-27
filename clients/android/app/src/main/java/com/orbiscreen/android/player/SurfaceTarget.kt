// Orbiscreen - SurfaceTarget.kt (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen

package com.orbiscreen.android.player

import android.view.Surface

interface SurfaceTarget {
    fun attachSurface(surface: Surface)
    fun detachSurface()
}
