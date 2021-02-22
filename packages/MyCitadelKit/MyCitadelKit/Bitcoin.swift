//
// Created by Maxim Orlovsky on 2/2/21.
//

import Foundation

public enum BitcoinNetwork: String, Codable {
    case mainnet = "mainnet"
    case testnet = "testnet"
    case signet = "signet"
}

public enum DescriptorType {
    case bare
    case hashed
    case segwit
    case taproot

    public func cDescriptorType() -> descriptor_type {
        switch self {
        case .bare: return DESCRIPTOR_TYPE_BARE
        case .hashed: return DESCRIPTOR_TYPE_HASHED
        case .segwit: return DESCRIPTOR_TYPE_SEGWIT
        case .taproot: return DESCRIPTOR_TYPE_TAPROOT
        }
    }
}

public struct Contract {
    let id: String
    let balance: [UTXO]
    let assetBalances: [String: [UTXO]]
}

public struct UTXO {
    let height: Int32
    let offset: UInt32
    let vout: UInt32
    let value: UInt64

    let derivationIndex: UInt32
    let address: String
}

let contracts: [Contract] = []

func some() {
    var balance: UInt64 = 0
    let assetId = ""
    for contract in contracts {
        if let assetBalance = contract.assetBalances[assetId] {
            balance += assetBalance.reduce(into: 0) { $0 += $1.value }
        }
    }
}
