//
// Created by Maxim Orlovsky on 2/22/21.
//

import Foundation

struct ContractData: Codable {
    let id: String
    let name: String
    let chain: BitcoinNetwork
    let policy: String
}

struct UTXO: Codable {
    let height: Int32
    let offset: UInt32
    let txid: String
    let vout: UInt32
    let value: UInt64
    let derivationIndex: UInt32
    let address: String?
}

public struct AssetData: Codable {
    public let genesis: String
    public let id: String
    public let ticker: String
    public let name: String
    public let description: String?
    public let fractionalBits: UInt8
    public let date: String
    public let knownCirculating: UInt64
    public let issueLimit: UInt64
}
