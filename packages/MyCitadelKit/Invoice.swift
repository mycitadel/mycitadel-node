//
// Created by Maxim Orlovsky on 3/3/21.
//

import Foundation

public struct InvoiceDetails: Identifiable, Codable {
    public var id: String {
        commitment
    }
    public let commitment: String
    public let source: URL
}

public struct Invoice: Codable {
    public var amountString: String = "any"
    public var assetId: String? = nil
    public let beneficiary: String
    public var merchant: String? = nil
    public var purpose: String? = nil
    public var details: InvoiceDetails? = nil

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
        assetId != nil ? CitadelVault.embedded.assets[assetId!] : CitadelVault.embedded.nativeAsset
    }

    public init(beneficiary: String) {
        self.beneficiary = beneficiary
    }

    private enum CodingKeys: String, CodingKey {
        case amountString = "amount", assetId = "asset", beneficiary, merchant, purpose, details
    }
}

public struct PaymentResult {
    public let txid: String
    public let consignment: String?
}

public enum InvoiceType {
    case addressUtxo
    case descriptor
    case psbt

    public func cType() -> invoice_type {
        switch self {
        case .addressUtxo: return INVOICE_TYPE_ADDRESS_UTXO
        case .descriptor: return INVOICE_TYPE_DESCRIPTOR
        case .psbt: return INVOICE_TYPE_PSBT
        }
    }
}
