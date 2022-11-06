package name.jinleili.gpuimage4

import android.view.Surface

class RustBridge private constructor() {

    companion object {
        @Volatile
        private lateinit var instance: RustBridge

        fun getInstance(): RustBridge {
            synchronized(this) {
                if (!::instance.isInitialized) {
                    instance = RustBridge()
                }
                return instance
            }
        }
    }
    init {
        System.loadLibrary("gpu_image4")
    }

    external fun createWgpuCanvas(surface: Surface): Long
    external fun enterFrame(rustObj: Long)

    external fun dropWgpuCanvas(rustObj: Long)
}