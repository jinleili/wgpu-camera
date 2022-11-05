package name.jinleili.gpuimage4

import android.view.Surface

class RustBridge {
    init {
        System.loadLibrary("camerafn")
        System.loadLibrary("gpu_image4")
    }
    external fun test()

    external fun createWgpuCanvas(surface: Surface): Long
    external fun enterFrame(rustObj: Long)

    external fun dropWgpuCanvas(rustObj: Long)
}