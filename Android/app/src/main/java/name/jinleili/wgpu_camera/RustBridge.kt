package name.jinleili.wgpu_camera

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
        System.loadLibrary("wgpu_camera")
    }

    external fun create_cameracanvas(surface: Surface): Long
    external fun capture_one_frame(rustObj: Long)
    external fun start_capturing(rustObj: Long)
    external fun stop_capturing(rustObj: Long)
    external fun enter_frame(rustObj: Long)

    external fun drop_camera_canvas(rustObj: Long)
}