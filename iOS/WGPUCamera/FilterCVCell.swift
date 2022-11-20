//
//  FilterCVCell.swift
//  WGPUCamera
//
//  Created by 李金磊 on 2022/10/31.
//

import UIKit

class FilterCVCell: UICollectionViewCell {
    lazy var nameLb: UILabel = {
        let lb = UILabel()
        lb.textColor = UIColor.black
        lb.textAlignment = .center
        lb.adjustsFontSizeToFitWidth = true
        lb.frame = self.contentView.frame
        self.contentView.addSubview(lb)
        self.contentView.layer.borderColor = UIColor.lightGray.cgColor
        self.contentView.layer.borderWidth = 0.5
        self.contentView.layer.cornerRadius = 8
        return lb
    }()
    
    func set_name(name: String) {
        self.nameLb.text = name
    }
}
