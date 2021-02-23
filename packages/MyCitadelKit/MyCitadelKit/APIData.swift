//
// Created by Maxim Orlovsky on 2/22/21.
//

import Foundation

struct ContractData: Codable {
    let id: String
    let name: String
    let chain: BitcoinNetwork
    let policy: Policy
}

public enum Policy {
    case current(String)
}

extension Policy: Codable {
    enum CodingKeys: CodingKey {
        case current
    }

    public init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        if container.contains(.current) {
            let value = try container.decode(String.self, forKey: .current)
            self = .current(value)
        } else {
            throw DecodingError.typeMismatch(String.self, DecodingError.Context(codingPath: [CodingKeys.current], debugDescription: "string value expected"))
        }
    }

    public func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)
        switch self {
        case .current(let value):
            try container.encode(value, forKey: .current)
        }
    }
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
