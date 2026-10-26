package com.styk.animate

import android.view.Surface

/**
 * Thin JNI bridge. Handles are opaque `Long` pointers to Rust objects.
 * Never add per-frame setters here — keep JNI calls coarse-grained.
 */
object StykCore {
    init {
        System.loadLibrary("styk_core")
    }

    external fun nativeCreate(): Long
    external fun nativeDestroy(handle: Long)
    external fun nativeAttachSurface(handle: Long, surface: Surface)
    external fun nativeDetachSurface(handle: Long)
    external fun nativeRenderFrame(handle: Long, frameTimeNanos: Long)
}
