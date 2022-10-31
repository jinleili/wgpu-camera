//
//  ViewController.swift
//  GPUImage4
//
//  Created by Jinlei Li on 2022/10/20.
//

import UIKit

class ViewController: UIViewController {
    @IBOutlet var metalV: MetalView!
    @IBOutlet var cv: UICollectionView!
    
    @IBOutlet var slider: UISlider!
    @IBOutlet var minLb: UILabel!
    @IBOutlet var maxLb: UILabel!

    var wgpuCanvas: OpaquePointer?
    
    var session: CameraSession?
    var texture: MTLTexture?
    var textureIndex = 0
    var latestCameraTexture: UnsafeMutableRawPointer?

    lazy var displayLink: CADisplayLink = {
        CADisplayLink.init(target: self, selector: #selector(enterFrame))
    }()
    
    override func viewDidLoad() {
        super.viewDidLoad()
        session = CameraSession(delegate: self)
        cv.dataSource = self
        cv.delegate = self
        cv.register(FilterCVCell.self, forCellWithReuseIdentifier: "cell")
       let layout =  cv.collectionViewLayout as! UICollectionViewFlowLayout
        let itemSize = CGSize(width: 110, height: 44)
        layout.itemSize = itemSize
        layout.estimatedItemSize = itemSize
        layout.scrollDirection = .horizontal

        slider.addTarget(self, action: #selector(sliderValueChanged), for: .valueChanged)
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
        // call wgpu
        enter_frame(canvas)
    }
}

// MARK: - MetalCameraSessionDelegate
extension ViewController: CameraSessionDelegate {
    func cameraSession(_ session: CameraSession, didReceiveFrameAsTextures textures: [MTLTexture], withTimestamp timestamp: Double) {
        guard let canvas = self.wgpuCanvas else {
            return
        }
        self.texture = textures[0]
        let tex_pointer = Unmanaged.passRetained( self.texture!).toOpaque()
        if tex_pointer != latestCameraTexture {
            print("----------- \(tex_pointer)")

            latestCameraTexture = tex_pointer
        }
        
        if textureIndex == 0 {
            textureIndex += 1

            displayLink.isPaused = true
            set_external_texture(canvas, tex_pointer, Int32(self.texture!.width), Int32(self.texture!.height))
            displayLink.isPaused = false
        }

    }
    
    func cameraSession(_ cameraSession: CameraSession, didUpdateState state: CameraSessionState, error: CameraSessionError?) {
        
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

