//
// Created by Maxim Orlovsky on 3/4/21.
//

import Foundation

public struct DescriptorInfo: Codable {
    public let fullType: String
    public let innerType: String
    public let outerType: String
    public let contentType: String
    public let category: String
    public let descrType: String
    public let addrType: String?
    public let descriptor: String
    public let policy: String
    public let sigsRequired: UInt?
    public let isNestable: Bool
    public let isSorted: Bool
    public let keys: [PubkeyChainInfo]
    public let keyspaceSize: UInt
    public let checksum: String?
}

extension DescriptorInfo {
    public var isRGBEnabled: Bool {
        keys.allSatisfy(\.isRGBEnabled)
    }
}

public struct XpubRefInfo: Codable {
    public let fingerprint: String
    public let identifier: String?
    public let xpubkey: String?
}

public struct XpubInfo: Codable {
    public let fingerprint: String
    public let identifier: String
    public let xpubkey: String
}

public struct PubkeyChainInfo: Codable {
    public let fullKey: String
    public let seedBased: Bool
    public let master: XpubRefInfo?
    public let sourcePath: [BranchStep]
    public let branch: XpubInfo
    public let revocationSeal: OutPoint?
    public let terminalPath: [TerminalStep]
    public let keyspaceSize: UInt
}

extension PubkeyChainInfo {
    public var identityKey: XpubRefInfo? {
        sourcePath.last(where: { $0.xpubRef != nil })?.xpubRef
    }

    public var isRGBEnabled: Bool {
        sourcePath.first?.indexString == "827166h"
    }

    public var bip32Derivation: String {
        "\(seedBased ? "m=" : "")\(master != nil ? "[\(master!.fingerprint)]" : "")/\(sourcePathString)/\(terminalPathString)"
    }

    public var sourcePathString: String {
        sourcePath.map { "\($0.indexString)" }.joined(separator: "/")
    }
    public var terminalPathString: String {
        terminalPath.map { "\($0.indexString)" }.joined(separator: "/")
    }
}

public enum BranchStep: Codable {
    case hardened(UInt32, XpubRefInfo?)
    case unhardened(UInt32)

    public var xpubRef: XpubRefInfo? {
        switch self {
        case .hardened(_, let xpubref):
            return xpubref
        case .unhardened(_):
            return nil
        }
    }

    public var indexString: String {
        switch self {
        case .hardened(let index, _):
            return "\(index)h"
        case .unhardened(let index):
            return "\(index)"
        }
    }

    enum CodingKeys: CodingKey {
        case hardened, unhardened
    }

    enum NestedKeys: CodingKey {
        case index, xpubRef
    }

    enum CodingError: Error {
        case unknownValue
    }

    public init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        if let container = try? container.nestedContainer(keyedBy: NestedKeys.self, forKey: .hardened) {
            let step = try container.decode(UInt32.self, forKey: .index)
            let xpubRef = try container.decode(XpubRefInfo?.self, forKey: .xpubRef)
            self = .hardened(step, xpubRef)
        } else {
            let step = try decoder.singleValueContainer().decode(String.self)
            guard let index = UInt32(step) else { throw CodingError.unknownValue }
            self = .unhardened(index)
        }
    }

    public func encode(to encoder: Encoder) throws {
        switch self {
        case .hardened(let index, let xpubRef):
            var container = encoder.container(keyedBy: NestedKeys.self);
            try container.encode(index, forKey: .index)
            try container.encode(xpubRef, forKey: .xpubRef)
        case .unhardened(let index):
            var container = encoder.singleValueContainer();
            try container.encode("\(index)")
        }
    }
}

public enum TerminalStep: Codable {
    case unhardened(UInt32)
    case range(UInt32, UInt32)
    case wildcard

    enum CodingError: Error {
        case unknownValue
    }

    public var indexString: String {
        switch self {
        case .unhardened(let index):
            return "\(index)"
        case .range(let from, let upto):
            return "\(from)-\(upto)"
        case .wildcard:
            return "*"
        }
    }

    public init(from decoder: Decoder) throws {
        let step = try decoder.singleValueContainer().decode(String.self)

        guard let last = step.last else { throw CodingError.unknownValue }
        if last == "*" {
            self = .wildcard
        } else if step.contains("-") {
            let components = step.split(separator: "-")
            if components.count != 2 { throw CodingError.unknownValue }
            guard let from = UInt32(components.first!),
                  let upto = UInt32(components.last!) else { throw CodingError.unknownValue }
            self = .range(from, upto)
        } else {
            guard let index = UInt32(step) else { throw CodingError.unknownValue }
            self = .unhardened(index)
        }
    }

    public func encode(to encoder: Encoder) throws {
        var container = encoder.singleValueContainer();
        switch self {
        case .wildcard:
            try container.encode("*")
        case .range(let from, let to):
            try container.encode("\(from)-\(to)")
        case .unhardened(let index):
            try container.encode("\(index)")
        }
    }
}
