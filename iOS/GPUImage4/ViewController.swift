//
//  ViewController.swift
//  GPUImage4
//
//  Created by Jinlei Li on 2022/10/20.
//

import UIKit

class ViewController: UIViewController {
    @IBOutlet var metalV: MetalView!
    var wgpuCanvas: OpaquePointer?
    
    var session: CameraSession?
    var texture: MTLTexture?
    var textureIndex = 0
    
    lazy var displayLink: CADisplayLink = {
        CADisplayLink.init(target: self, selector: #selector(enterFrame))
    }()
    
    override func viewDidLoad() {
        super.viewDidLoad()
        session = CameraSession(delegate: self)

        self.displayLink.add(to: .current, forMode: .default)
        self.displayLink.isPaused = true
    }

    override func viewDidAppear(_ animated: Bool) {
        super.viewDidAppear(animated)
        self.view.backgroundColor = .white
        if wgpuCanvas == nil {
            let viewPointer = Unmanaged.passUnretained(self.metalV).toOpaque()
            let metalLayer = Unmanaged.passUnretained(self.metalV.layer).toOpaque()
            let maximumFrames = UIScreen.main.maximumFramesPerSecond
            
            let viewObj = ios_view_obj(view: viewPointer, metal_layer: metalLayer,maximum_frames: Int32(maximumFrames), callback_to_swift: callback_to_swift)
            wgpuCanvas = create_wgpu_canvas(viewObj)
        }
        self.displayLink.isPaused = false
        session?.start()
    }
    
    override func viewWillDisappear(_ animated: Bool) {
        super.viewWillDisappear(animated)
        displayLink.isPaused = true
    }
    
    override func viewDidDisappear(_ animated: Bool) {
        super.viewDidDisappear(animated)
        session?.stop()
    }
    
    @objc func enterFrame() {
        guard let canvas = self.wgpuCanvas else {
            return
        }
        // call rust
        enter_frame(canvas)
    }
}

// MARK: - MetalCameraSessionDelegate
extension ViewController: CameraSessionDelegate {
    func metalCameraSession(_ session: CameraSession, didReceiveFrameAsTextures textures: [MTLTexture], withTimestamp timestamp: Double) {
        guard let canvas = self.wgpuCanvas else {
            return
        }

        if textureIndex == 0 {
            textureIndex += 1

            self.texture = textures[0]
            let tex_pointer = Unmanaged.passUnretained( self.texture!).toOpaque()
            let external_tex = external_texture_obj(width: Int32(self.texture!.width), height: Int32(self.texture!.height), raw: tex_pointer )

            print("\(external_tex)")

//            set_external_texture(canvas, external_tex)
            set_external_texture2(canvas, tex_pointer, Int32(self.texture!.width), Int32(self.texture!.height))
        }
       
    }
    
    func metalCameraSession(_ cameraSession: CameraSession, didUpdateState state: CameraSessionState, error: MetalCameraSessionError?) {
        
        if error == .captureSessionRuntimeError {
            /**
             *  In this app we are going to ignore capture session runtime errors
             */
            cameraSession.start()
        }
        
        DispatchQueue.main.async {
            self.title = "Metal camera: \(state)"
        }
        
        NSLog("Session changed state to \(state) with error: \(error?.localizedDescription ?? "None").")
    }
}


func callback_to_swift(arg: Int32) {
    DispatchQueue.main.async {
        switch arg {
        case 0:
            print("wgpu canvas created!")
            break
        case 1:
            print("canvas enter frame")
            break
            
        default:
            break
        }
    }
    
}

