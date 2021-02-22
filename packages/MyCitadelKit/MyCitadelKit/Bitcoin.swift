//
// Created by Maxim Orlovsky on 2/2/21.
//

import Foundation

public enum BitcoinNetwork: String, Codable {
    case mainnet = "mainnet"
    case testnet = "testnet"
    case signet = "signet"

    public func derivationIndex() -> UInt32 {
        switch self {
        case .mainnet: return 0
        case .testnet, .signet: return 1
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
}

public struct PubkeyChain {
    static func construct(network: BitcoinNetwork, rgbSupport: Bool, descriptorType: DescriptorType, multisig: Bool, scope: UInt32?) -> String {
        let boundary = UInt32.max & 0x7FFFFFFF
        let id = UInt32.random(in: 0...boundary)
        let scope = UInt32.random(in: 0...boundary);
        return rgbSupport
                ? "m/827166'/\(descriptorType.usesSchnorr() ? "340" : "0")'/\(network.derivationIndex())'/\(id)'/\(scope)'/0/*"
                : descriptorType.usesSchnorr()
                ? "m/\(multisig ? 345 : 344)'/0'/\(scope)/0/*"
                : descriptorType.usesSegWit()
                ? "m/\(multisig ? 84 : 84)'/0'/\(scope)/0/*"
                : "m/\(multisig ? 45 : 44)'/0'/\(scope)/0/*"
    }
}
