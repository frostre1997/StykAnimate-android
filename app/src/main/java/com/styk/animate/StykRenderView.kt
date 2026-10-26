package com.styk.animate

import androidx.compose.foundation.AndroidExternalSurface
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier

@Composable
fun StykRenderView(modifier: Modifier = Modifier) {
    var engineHandle by remember { mutableLongStateOf(0L) }

    DisposableEffect(Unit) {
        engineHandle = StykCore.nativeCreate()
        onDispose {
            if (engineHandle != 0L) {
                StykCore.nativeDestroy(engineHandle)
                engineHandle = 0L
            }
        }
    }

    AndroidExternalSurface(modifier = modifier.fillMaxSize()) {
        onSurface { surface, width, height ->
            if (engineHandle != 0L) {
                // Called every time a surface is created or recreated (rotation, resume)
                StykCore.nativeAttachSurface(engineHandle, surface)
            }
        }

        // Drive the render loop from Compose's frame clock
        LaunchedEffect(engineHandle) {
            while (engineHandle != 0L) {
                withFrameNanos { frameTimeNanos ->
                    StykCore.nativeRenderFrame(engineHandle, frameTimeNanos)
                }
            }
        }
    }
}
