//
//  ViewController+CollectionView.swift
//  WGPUCamera
//
//  Created by 李金磊 on 2022/10/31.
//

import UIKit

let nativeScale = Float(UIScreen.main.nativeScale)
struct Filter {
    public var name: String
    public var min: Float
    public var max: Float
    
    init(name: String, min: Float, max: Float) {
        self.name = name
        self.min = min
        self.max = max
    }
}

let filters = [
    Filter.init(name: "Original", min: 0.0, max: 0.0),
    Filter.init(name: "ASCII Art", min: 4.0 * nativeScale, max: 12.0 * nativeScale) ,
    Filter.init(name: "Cross Hatch", min: 8.0 * nativeScale, max: 20.0 * nativeScale),
    Filter.init(name: "Edge Detection", min: 0.05, max: 0.33)
]

extension ViewController: UICollectionViewDelegate, UICollectionViewDataSource {
    func collectionView(_ collectionView: UICollectionView, numberOfItemsInSection section: Int) -> Int {
        filters.count
    }
    
    func collectionView(_ collectionView: UICollectionView, cellForItemAt indexPath: IndexPath) -> UICollectionViewCell {
        let cell = collectionView.dequeueReusableCell(withReuseIdentifier: "cell", for: indexPath) as! FilterCVCell
        let filter = filters[indexPath.row]
        cell.set_name(name: filter.name)
        return cell
    }
    
    func collectionView(_ collectionView: UICollectionView, didSelectItemAt indexPath: IndexPath) {
        guard let canvas = self.wgpuCanvas else {
            return
        }
        // call wgpu
        let filter = filters[indexPath.row]
        slider.minimumValue = filter.min
        slider.maximumValue = filter.max
        slider.value = filter.min
        minLb.text = "\(filter.min)"
        maxLb.text = "\(filter.max)"

        set_filter(canvas, filter_type(UInt32(indexPath.row)), 0, filter.min)
    }
    
    @IBAction func sliderValueChanged() {
        guard let canvas = self.wgpuCanvas else {
            return
        }
        change_filter_param(canvas, slider.value)
    }
    
}
