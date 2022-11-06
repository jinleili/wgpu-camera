package name.jinleili.gpuimage4

import android.Manifest
import android.content.Context
import android.content.pm.PackageManager
import android.os.Bundle
import android.util.Log
import androidx.activity.ComponentActivity
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.compose.setContent
import androidx.activity.result.contract.ActivityResultContracts
import androidx.camera.core.CameraSelector
import androidx.camera.lifecycle.ProcessCameraProvider
import androidx.compose.foundation.layout.*
import androidx.compose.material.ExperimentalMaterialApi
import androidx.compose.material.MaterialTheme
import androidx.compose.material.Surface
import androidx.compose.material.Text
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalConfiguration
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.viewinterop.AndroidView
import androidx.core.app.ActivityCompat
import androidx.core.content.ContextCompat
import name.jinleili.gpuimage4.ui.theme.GPUImage4Theme

class MainActivity : ComponentActivity() {
    private var shouldShowCamera: MutableState<Boolean> = mutableStateOf(false)
    private var cameraX: CameraX = CameraX(this, this)

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContent {
            GPUImage4Theme {
                Surface(
                    modifier = Modifier.fillMaxSize(),
                    color = MaterialTheme.colors.background
                ) {
                    if (shouldShowCamera.value) {
//                        AndroidView(
//                            modifier = Modifier.fillMaxSize(),
//                            factory = { cameraX.startCameraPreviewView() }
//                        )
//                    } else {
                        SurfaceCard()
                    }
                }
            }
        }

        requestCameraPermission()
    }

    private val requestPermissionLauncher = registerForActivityResult(
        ActivityResultContracts.RequestPermission()
    ) { isGranted ->
        shouldShowCamera.value = isGranted
        if (isGranted) {
            Log.i("wgpu", "Permission granted")
        } else {
            Log.i("wgpu", "Permission denied")
        }
    }

    // https://www.kiloloco.com/articles/015-camera-jetpack-compose/
    private fun requestCameraPermission() {
        when {
            ContextCompat.checkSelfPermission(
                this,
                Manifest.permission.CAMERA
            ) == PackageManager.PERMISSION_GRANTED -> {
                shouldShowCamera.value = true
                Log.i("wgpu", "Permission previously granted")
            }

            ActivityCompat.shouldShowRequestPermissionRationale(
                this,
                Manifest.permission.CAMERA
            ) -> Log.i("wgpu", "Show camera permissions dialog")

            else -> requestPermissionLauncher.launch(Manifest.permission.CAMERA)
        }
    }
}

var surfaceView: WGPUSurfaceView? = null

@Composable
fun SurfaceCard() {
    val screenWidth = LocalConfiguration.current.screenWidthDp.dp
    Column(modifier = Modifier.fillMaxSize()) {
        Row(
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.Center,
            modifier = Modifier
                .height(44.dp)
                .padding(horizontal = 0.dp, vertical = 7.dp)
                .fillMaxWidth()
        ) {
            Text(text = "GPUImage 4", fontSize = 20.sp, fontWeight = FontWeight.Bold)
        }
        Spacer(modifier = Modifier.height(8.dp))
        AndroidView(
            factory = { ctx ->
                val sv = WGPUSurfaceView(context = ctx)
                surfaceView = sv
                sv
            },
            modifier = Modifier
                .fillMaxWidth()
                .height(screenWidth),
        )
    }

}

@Preview(showBackground = true)
@Composable
fun DefaultPreview() {
    GPUImage4Theme {
        SurfaceCard()
    }
}