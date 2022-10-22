package name.jinleili.gpuimage4

import android.view.Surface

class RustBridge {
    init {
        System.loadLibrary("gpu_image4")
    }

    external fun createWgpuCanvas(surface: Surface): Long
    external fun enterFrame(rustObj: Long)

    external fun dropWgpuCanvas(rustObj: Long)
}