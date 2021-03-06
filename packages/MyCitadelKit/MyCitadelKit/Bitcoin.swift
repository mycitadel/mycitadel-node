//
// Created by Maxim Orlovsky on 2/2/21.
//

import Foundation

public enum BitcoinNetwork: String, Codable {
    static let rgbAssetId = "rgb1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqg40adx"

    case mainnet = "mainnet"
    case testnet = "testnet"
    case signet = "signet"

    public func derivationIndex() -> UInt32 {
        switch self {
        case .mainnet: return 0
        case .testnet, .signet: return 1
        }
    }

    public func ticker() -> String {
        switch self {
        case .mainnet:
            return "BTC"
        case .testnet:
            return "tBTC"
        case .signet:
            return "sBTC"
        }
    }

    public func coinName() -> String {
        switch self {
        case .mainnet:
            return "Bitcoin"
        case .testnet:
            return "Bitcoin (testnet)"
        case .signet:
            return "Bitcoin (signet)"
        }
    }

    public func issueLimit() -> UInt64? {
        switch self {
        case .mainnet:
            return 21_000_000_0000_0000
        case .testnet:
            return nil
        case .signet:
            return nil
        }
    }

    // TODO: keep these values up to date
    public func issuedSupply() -> UInt64 {
        switch self {
        case .mainnet:
            return 18_636_414_0000_0000
        case .testnet:
            return 20_963_086_0000_0000
        case .signet:
            return 26265 * 50 * 1_0000_0000
        }
    }

    public func genesisTimestamp() -> Int64 {
        switch self {
        case .mainnet:
            return 1231006505
        case .testnet:
            return 1296688602
        case .signet:
            return 1598918400
        }
    }

    public func genesisDate() -> Date {
        Date(timeIntervalSince1970: TimeInterval(genesisTimestamp()))
    }

    public func geneisHash() -> String {
        switch self {
        case .mainnet:
            return "000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f"
        case .testnet:
            return "000000000933ea01ad0ee984209779baaec3ced90fa3f408719526f8d77f4943"
        case .signet:
            return "00000008819873e925422c1ff0f99f7cc9bbb232af63a077a480a3633bee1ef6"
        }
    }

    public func nativeAssetId() -> String {
        geneisHash()
    }
}

public struct BlockchainState {
    public let updatedAt: Date
    public let height: UInt32
    public let supply: UInt64
    public let lastBlockHash: String
    public let lastBlockTime: Int64
    public let lastBlockReward: UInt64
    public let knownBurned: UInt64

    public init() {
        updatedAt = Date(timeIntervalSince1970: 0)
        height = 0
        supply = 0
        lastBlockHash = ""
        lastBlockTime = 0
        lastBlockReward = 0
        knownBurned = 0
    }
}

public struct MempoolState {
    public let updatedAt: Date
    public let txCount: UInt64
    public let totalFee: UInt64

    public init() {
        updatedAt = Date(timeIntervalSince1970: 0)
        txCount = 0
        totalFee = 0
    }
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

    public func usesSchnorr() -> Bool {
        self == .taproot
    }

    public func usesSegWit() -> Bool {
        self == .segwit
    }

    public func createPubkeyChain(network: BitcoinNetwork, rgb: Bool, multisig: Bool, scope: UInt32?) -> String {
        let boundary = UInt32.max & 0x7FFFFFFF
        let id = UInt32.random(in: 0...boundary)
        let scope = UInt32.random(in: 0...boundary);
        return rgb
                ? "m/827166'/\(usesSchnorr() ? "340" : "0")'/\(network.derivationIndex())'/\(id)'/\(scope)'/0/*"
                : usesSchnorr()
                ? "m/\(multisig ? 345 : 344)'/0'/\(scope)/0/*"
                : usesSegWit()
                ? "m/\(multisig ? 84 : 84)'/0'/\(scope)/0/*"
                : "m/\(multisig ? 45 : 44)'/0'/\(scope)/0/*"
    }
}

public enum WitnessVersion: Equatable {
    case none
    case segwit
    case taproot
    case future(UInt8)
}

public enum AddressNetwork: Codable, Equatable {
    case mainnet
    case testnet
    case regtest

    enum CodingError: Error {
        case unknownValue
    }

    public init(from decoder: Decoder) throws {
        guard let value = try? decoder.singleValueContainer().decode(String.self)
                else { throw CodingError.unknownValue }
        switch value {
        case "mainnet": self = .mainnet
        case "testnet": self = .testnet
        case "regtest": self = .regtest
        default: throw CodingError.unknownValue
        }
    }

    public func encode(to encoder: Encoder) throws {
        var container = encoder.singleValueContainer();
        switch self {
        case .mainnet: try container.encode("mainnet")
        case .testnet: try container.encode("testnet")
        case .regtest: try container.encode("regtest")
        }
    }
}

public enum AddressFormat: RawRepresentable, Codable, Equatable {
    case P2PKH
    case P2SH
    case P2WPKH
    case P2WSH
    case P2TR
    case future(UInt8)

    public typealias RawValue = String

    public init?(rawValue: String) {
        var rawValue = rawValue
        if rawValue.hasPrefix("P2W") {
            rawValue.removeFirst(3)
            guard let ver = UInt8(rawValue) else { return nil }
            self = .future(ver)
            return
        }
        switch rawValue {
        case "P2PKH": self = .P2PKH
        case "P2SH": self = .P2SH
        case "P2WPKH": self = .P2WPKH
        case "P2WSH": self = .P2WSH
        case "P2TR": self = .P2TR
        default: return nil
        }
    }

    public var rawValue: String {
        switch self {
        case .P2PKH: return "P2PKH"
        case .P2SH: return "P2SH"
        case .P2WPKH: return "P2WPKH"
        case .P2WSH: return "P2WSH"
        case .P2TR: return "P2TR"
        case .future(let ver): return "P2W\(ver)"
        }
    }

    enum CodingError: Error {
        case unknownValue
    }

    public init(from decoder: Decoder) throws {
        guard let value = try? decoder.singleValueContainer().decode(String.self)
                else { throw CodingError.unknownValue }
        guard let raw = AddressFormat(rawValue: value)
                else { throw CodingError.unknownValue }
        self = raw
    }

    public func encode(to encoder: Encoder) throws {
        var container = encoder.singleValueContainer();
        try container.encode(rawValue)
    }
}

public extension AddressFormat {
    var witnessVersion: WitnessVersion {
        switch self {
        case .P2PKH, .P2SH: return .none
        case .P2WPKH, .P2WSH: return .segwit
        case .P2TR: return .taproot
        case .future(let ver): return .future(ver)
        }
    }
}

public struct AddressInfo: Codable, Identifiable, Equatable {
    public var id: String { address }

    public let address: String
    public let network: AddressNetwork
    public let payload: String
    public let value: UInt64?
    public let label: String?
    public let message: String?
    public let format: AddressFormat

    public var witnessVer: WitnessVersion {
        format.witnessVersion
    }
}

public struct AddressDerivation: Codable, Identifiable, Equatable {
    public var id: String { address }
    public let address: String
    public let derivation: [UInt32]

    public var path: String {
        derivation.map { "\($0)" }.joined(separator: "/")
    }
}

public struct OutPoint: Codable, Identifiable, Hashable, Equatable {
    public var id: String {
        "\(txid):\(vout)"
    }
    public let txid: String
    public let vout: UInt16
}

public struct TweakedOutpoint: Codable, Identifiable, Equatable {
    public var id: String {
        outpoint.id
    }
    public let outpoint: OutPoint
    public let script: String
    public let tweak: String
    public let pubkey: String
    public let derivationIndex: UInt32
}
