//
// Created by Maxim Orlovsky on 2/2/21.
//

import Foundation

public enum BitcoinNetwork: String {
    case Mainnet = "mainnet"
    case Testnet = "testnet"
    case Signet = "signet"
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
