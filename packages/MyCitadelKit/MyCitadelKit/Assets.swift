//
// Created by Maxim Orlovsky on 2/2/21.
//

import Foundation
import Combine

public enum AssetCategory {
    case currency
    case stablecoin
    case token
    case nft
}

public enum SupplyMetric {
    case knownIssued
    case knownBurned
    case maxIssued
    case maxUnknown
    case knownReplaced
}

public protocol Asset {
    var isNative: Bool { get }
    var network: BitcoinNetwork { get }
    var category: AssetCategory { get }

    var genesis: String { get }
    var id: String { get }
    var ticker: String { get }
    var name: String { get }
    var ricardianContract: String? { get }
    var genesisTimetamp: Int64 { get }
    var genesisDate: Date { get }
    var decimalPrecision: UInt8 { get }

    var isSecondaryIssuePossible: Bool { get }
    var countIssues: UInt16 { get }
    var latestIssue: Date { get }
    var isIssueLimitReached: Bool { get }
    func percentageIssued(includingUnknown: Bool) -> Double?

    var isRenominationPossible: Bool { get }
    var isProofOfBurnPossible: Bool { get }
    var isReplacementPossible: Bool { get }

    func supply(metricInAtoms: SupplyMetric) -> UInt64?
    func supply(metric: SupplyMetric) -> Double?

    var balance: Balance { get }
    var hasBalance: Bool { get }

    func decimalFraction() -> UInt64

    func amount(fromAtoms: UInt64) -> Double
    func amount(toAtoms: Double) -> UInt64

    var authenticity: AssetAuthenticity { get }
}

public extension Asset {
    var genesisDate: Date {
        Date(timeIntervalSince1970: TimeInterval(genesisTimetamp))
    }

    func amount(fromAtoms atoms: UInt64) -> Double {
        Double(atoms) / Double(decimalFraction())
    }

    func amount(toAtoms amount: Double) -> UInt64 {
       UInt64((amount * Double(decimalFraction())).rounded(.toNearestOrAwayFromZero))
    }

    func supply(metric: SupplyMetric) -> Double? {
        guard let sum = supply(metricInAtoms: metric) else { return nil }
        return amount(fromAtoms: sum)
    }

    var hasBalance: Bool {
        balance.total > 0
    }

    func decimalFraction() -> UInt64 {
        [1, 10, 100,
         1_000, 10_000, 100_000,
         1_000_000, 10_000_000, 100_000_000,
         1_000_000_000, 10_000_000_000, 100_000_000_000,
         1_000_000_000_000, 10_000_000_000_000, 100_000_000_000_000,
         1_000_000_000_000_000, 10_000_000_000_000_000, 100_000_000_000_000_000,
         1_000_000_000_000_000_000, 10_000_000_000_000_000_000][Int(decimalPrecision)]
    }

    func percentageIssued(includingUnknown: Bool) -> Double? {
        guard var sum = supply(metricInAtoms: .knownIssued) else { return nil }
        if includingUnknown {
            guard let unknown = supply(metricInAtoms: .maxUnknown) else { return nil }
            sum += unknown
        }
        guard let max = supply(metric: .maxIssued) else { return nil }
        return amount(fromAtoms: sum) / max * 100.0
    }
}

public class NativeAsset: Asset, ObservableObject {
    internal let vault: CitadelVault

    public let isNative: Bool = true
    public var network: BitcoinNetwork {
        vault.network
    }
    public let category: AssetCategory = .currency
    public var genesis: String {
        network.geneisHash()
    }
    public var id: String {
        network.geneisHash()
    }
    public var ticker: String {
        network.ticker()
    }
    public var name: String {
        network.coinName()
    }
    public let ricardianContract: String? = nil
    public var genesisTimetamp: Int64 {
        network.genesisTimestamp()
    }
    public let decimalPrecision: UInt8 = 8
    public let isSecondaryIssuePossible: Bool = true
    public var countIssues: UInt16 {
        UInt16(vault.blockchainState.height)
    }
    public var latestIssue: Date {
        vault.blockchainState.updatedAt
    }

    public let isIssueLimitReached: Bool = false
    public let isRenominationPossible: Bool = false
    public let isProofOfBurnPossible: Bool = true
    public let isReplacementPossible: Bool = false

    public func supply(metricInAtoms: SupplyMetric) -> UInt64? {
        switch metricInAtoms {
        case .knownIssued:
            return vault.blockchainState.supply
        case .knownBurned:
            return nil // vault.blockchainState.knownBurned
        case .maxIssued:
            return network.issueLimit() ?? UInt64.max
        case .maxUnknown:
            return vault.blockchainState.lastBlockReward
        case .knownReplaced:
            return 0
        }
    }
    public var balance: Balance {
        let allocations = vault.balances.filter { $0.assetId == network.nativeAssetId() }.flatMap { $0.unspentAllocations }
        return Balance(withAsset: self, walletId: "", unspent: allocations)
    }

    public var authenticity: AssetAuthenticity {
        var issuer: Issuer
        if network == .signet {
            issuer = Issuer(name: "Signet federation", details: nil)
        } else {
            issuer = Issuer(name: "Proof of Work", details: nil)
        }
        return AssetAuthenticity(issuer: issuer, status: .publicTruth, url: nil, signature: nil)
    }

    public init(withCitadelVault citadelVault: CitadelVault) {
        vault = citadelVault
    }
}

public class RGB20Asset: Asset, ObservableObject {
    internal let vault: CitadelVault

    public let isNative: Bool = false
    public let network: BitcoinNetwork
    public let category: AssetCategory = .token
    public let genesis: String
    public let id: String
    public let ticker: String
    public let name: String
    public let ricardianContract: String?
    public let genesisTimetamp: Int64
    public let decimalPrecision: UInt8
    public let isSecondaryIssuePossible: Bool
    @Published
    public internal(set) var countIssues: UInt16
    @Published
    public internal(set) var latestIssue: Date
    @Published
    public internal(set) var isIssueLimitReached: Bool
    public let isRenominationPossible: Bool
    public let isProofOfBurnPossible: Bool
    public let isReplacementPossible: Bool
    public let authenticity: AssetAuthenticity

    @Published
    public internal(set) var knownIssued: UInt64?
    @Published
    public internal(set) var knownBurned: UInt64?
    @Published
    public internal(set) var maxIssued: UInt64
    @Published
    public internal(set) var maxUnknown: UInt64?
    @Published
    public internal(set) var knownReplaced: UInt64?

    public func supply(metricInAtoms: SupplyMetric) -> UInt64? {
        switch metricInAtoms {
        case .knownIssued:
            return knownIssued
        case .knownBurned:
            return knownBurned
        case .maxIssued:
            return maxIssued
        case .maxUnknown:
            return maxUnknown
        case .knownReplaced:
            return knownReplaced
        }
    }

    public var balance: Balance {
        let allocations = vault.balances.filter { $0.assetId == id }.flatMap { $0.unspentAllocations }
        return Balance(withAsset: self, walletId: "", unspent: allocations)
    }

    init(withAssetData asset: RGB20Json, citadelVault: CitadelVault) {
        vault = citadelVault

        network = citadelVault.network
        genesis = asset.genesis
        id = asset.id
        ticker = asset.ticker
        name = asset.name
        ricardianContract = asset.description
        decimalPrecision = asset.fractionalBits
        genesisTimetamp = 0 // asset.date
        knownIssued = asset.knownCirculating
        maxIssued = asset.issueLimit ?? UInt64.max

        // TODO: Fill in the data below basing on the asset genesis
        knownBurned = nil
        maxUnknown = nil
        knownReplaced = nil

        isSecondaryIssuePossible = false
        isRenominationPossible = false
        isReplacementPossible = false
        isProofOfBurnPossible = false

        countIssues = 1
        latestIssue = Date(timeIntervalSince1970: 0)
        isIssueLimitReached = false

        authenticity = AssetAuthenticity(issuer: nil, status: .unverified, url: nil, signature: nil)
    }
}

public struct Issuer {
    public let name: String
    public let details: String?
}

public struct AssetAuthenticity {
    public let issuer: Issuer?
    public let status: VerificationStatus
    public let url: String?
    public let signature: String?
}

public enum VerificationStatus {
    case publicTruth
    case verified
    case unverified

    public func isVerified() -> Bool {
        self != .unverified
    }
}
