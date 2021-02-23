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

    public func genesisDate() -> String {
        switch self {
        case .mainnet:
            return "2009-01-03 19:15:00"
        case .testnet:
            return "2020-09-01 00:00:00"
        case .signet:
            return "2011-02-03 00:16:42"
        }
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
        return self == .taproot
    }

    public func usesSegWit() -> Bool {
        return self == .segwit
    }

    public func createPubkeyChain(network: BitcoinNetwork, rgb: Bool, multisig: Bool, scope: UInt32?) -> String {
        let boundary = UInt32.max & 0x7FFFFFFF
        let id = UInt32.random(in: 0...boundary)
        let scope = UInt32.random(in: 0...boundary);
        return rgb
                ? "m/827166'/\(self.usesSchnorr() ? "340" : "0")'/\(network.derivationIndex())'/\(id)'/\(scope)'/0/*"
                : self.usesSchnorr()
                ? "m/\(multisig ? 345 : 344)'/0'/\(scope)/0/*"
                : self.usesSegWit()
                ? "m/\(multisig ? 84 : 84)'/0'/\(scope)/0/*"
                : "m/\(multisig ? 45 : 44)'/0'/\(scope)/0/*"
    }
}

public enum WitnessVersion {
    case none
    case segwit
    case taproot
}
