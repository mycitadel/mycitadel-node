//
// Created by Maxim Orlovsky on 2/22/21.
//

import Foundation

struct ContractJson: Codable {
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

struct UTXOJson: Codable {
    let height: Int32
    let offset: UInt32
    let txid: String
    let vout: UInt32
    let value: UInt64
    let derivationIndex: UInt32
    let address: String?
}

struct RGB20Json: Codable {
    let genesis: String
    let id: String
    let ticker: String
    let name: String
    let description: String?
    let fractionalBits: UInt8
    let date: String
    let knownCirculating: UInt64
    let issueLimit: UInt64?
}

internal protocol CitadelRPC {
    func create(singleSig derivation: String, name: String, descriptorType: DescriptorType) throws -> ContractJson
    func listContracts() throws -> [ContractJson]
    func balance(walletId: String) throws -> [String: [UTXOJson]]
    func listAssets() throws -> [RGB20Json]
    func importRGB(genesisBech32 genesis: String) throws -> RGB20Json
}
