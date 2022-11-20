//
//  MetalView.swift
//
//  Created by Jinlei Li on 2022/10/20.
//

import UIKit
import Foundation

class MetalView: UIView {
    override class var layerClass: AnyClass {
        return CAMetalLayer.self
    }
    
    override func awakeFromNib() {
        super.awakeFromNib()
        configLayer()
    }
    
    private func configLayer() {
        guard let layer = self.layer as? CAMetalLayer else {
            return
        }
        layer.pixelFormat = .bgra8Unorm_srgb
        layer.presentsWithTransaction = false
        layer.framebufferOnly = true
        layer.backgroundColor = UIColor.clear.cgColor

        // nativeScale is real physical pixel scale
        // https://tomisacat.xyz/tech/2017/06/17/scale-nativescale-contentsscale.html
        self.contentScaleFactor = UIScreen.main.nativeScale
    }
}

