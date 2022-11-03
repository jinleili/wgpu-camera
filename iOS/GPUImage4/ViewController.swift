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
    var latestCameraTexture: UnsafeMutableRawPointer?
    
    // Record texture unused times
    var texKV: Dictionary<String, Int32> = [:]
    var current_tex_key: String = ""
    var frameIndex = 0

    lazy var displayLink: CADisplayLink = {
        CADisplayLink.init(target: self, selector: #selector(enterFrame))
    }()
    
    override func viewDidLoad() {
        super.viewDidLoad()
        if let bgImg = UIImage(named: "paper") {
            self.view.backgroundColor = UIColor.init(patternImage: bgImg)
        }
        
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
        enter_frame(canvas, current_tex_key)
        
        // 每 60 帧清理一次纹理缓存
        frameIndex += 1
        if frameIndex >= 60 {
            frameIndex = 0
            
            if texKV.count > 2 {
               let s = texKV.sorted { a, b in
                    a.value < b.value
                }
                // 如果一个纹理连续 59 帧以上没被使用，则清除此纹理在 wgpu 中的对应 bind_group
                s.last.map { d in
                    if d.value > 59 {
                        remove_texture(canvas, d.key)
                        texKV.removeValue(forKey: d.key)
                    }
                }
            }
        }

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
        let tex_key = "\(tex_pointer)"
        var isNewTexture = true
        // 记录连续没被使用的次数
        for (k, v) in texKV {
            if k == tex_key {
                texKV.updateValue(0, forKey: k)
                isNewTexture = false
            } else {
                texKV.updateValue(v + 1, forKey: k)
            }
        }
        if isNewTexture {
            texKV.updateValue(0, forKey: tex_key)
        }
        current_tex_key  = tex_key
                
//        if tex_pointer != latestCameraTexture {
//            print("----------- \(tex_pointer)")
//
//            latestCameraTexture = tex_pointer
//        }
        
        if isNewTexture {
//            displayLink.isPaused = true
            set_external_texture(canvas, tex_pointer, tex_key, Int32(self.texture!.width), Int32(self.texture!.height))
//            displayLink.isPaused = false
        }
//        print(texKV.values.sorted(by: { a, b in
//            a < b
//        }))

    }
    
    func cameraSession(_ cameraSession: CameraSession, didUpdateState state: CameraSessionState, error: CameraSessionError?) {
        
        if error == .captureSessionRuntimeError {
            /**
             *  In this app we are going to ignore capture session runtime errors
             */
            cameraSession.start()
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

