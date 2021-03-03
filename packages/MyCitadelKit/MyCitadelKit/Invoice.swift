//
// Created by Maxim Orlovsky on 3/3/21.
//

import Foundation

public struct Invoice: Codable {
    public var amountString: String = "any"
    public var assetId: String? = nil
    public let beneficiary: String
    public var merchant: String? = nil

    public var amountInAtoms: UInt64? {
        if amountString == "any" {
            return nil
        } else {
            return UInt64(amountString)
        }
    }
    public var amount: Double? {
        if let amountInAtoms = amountInAtoms {
            return asset?.amount(fromAtoms: amountInAtoms)
        } else {
            return nil
        }
    }
    public var asset: Asset? {
        CitadelVault.embedded.assets[assetId ?? CitadelVault.embedded.network.nativeAssetId()]
    }

    public init(beneficiary: String) {
        self.beneficiary = beneficiary
    }

    private enum CodingKeys: String, CodingKey {
        case amountString = "amount", assetId = "asset", beneficiary, merchant
    }
}
