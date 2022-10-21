//
//  MetalCameraCaptureDevice.swift
//  GPUImage4
//
//  Created by Jinlei Li on 2022/10/21.
//

import AVFoundation

internal class CameraCaptureDevice {
   
    internal func device(for mediaType: AVMediaType, with position: AVCaptureDevice.Position) -> AVCaptureDevice? {
        return AVCaptureDevice.devices(for: mediaType).first { $0.position == position }
    }

    internal func requestAccess(for mediaType: AVMediaType, completionHandler handler: @escaping ((Bool) -> Void)) {
        AVCaptureDevice.requestAccess(for: mediaType, completionHandler: handler)
    }
}
