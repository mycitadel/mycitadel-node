//
// Created by Maxim Orlovsky on 3/4/21.
//

import Foundation

public class WalletContract: Identifiable {
    internal let vault: CitadelVault

    public let id: String
    public var name: String
    public let chain: BitcoinNetwork
    public let policy: Policy

    public lazy var operations: [TransferOperation] = (try? syncOperations()) ?? []

    init(withContractData contractData: ContractJson, citadelVault: CitadelVault) {
        vault = citadelVault
        id = contractData.id
        name = contractData.name
        chain = contractData.chain
        policy = contractData.policy
        try? sync()
    }

    public private(set) lazy var descriptorInfo: DescriptorInfo = try! parseDescriptor()

    public var availableAssetIds: [String] {
        Array(allBalances.keys)
    }

    public var availableAssets: [Asset] {
        availableAssetIds.map { vault.assets[$0]! }
    }

    public var outpoints: [OutPoint] {
        balance()?.unspentAllocations.map { $0.outpoint } ?? []
    }

    public var allBalances: [String: Balance] {
        var balances: [String: Balance] = [:]
        vault.balances.filter { $0.walletId == id }.forEach { balances[$0.assetId] = $0 }
        return balances
    }

    public func balance(of assetId: String? = nil) -> Balance? {
        let assetId = assetId ?? vault.network.nativeAssetId()
        guard let asset = vault.assets[assetId] else { return nil }
        let allocations = vault.balances.filter { $0.walletId == id && $0.assetId == assetId }.flatMap { $0.unspentAllocations }
        return Balance(withAsset: asset, walletId: id, unspent: allocations)
    }

    public func balanceInAtoms(of assetId: String) -> UInt64 {
        vault.balances.filter { $0.walletId == id && $0.assetId == assetId }.reduce(0) { sum, balance in sum + balance.totalInAtoms }
    }

    public var hasUtxo: Bool {
        vault.balances.contains(where: { $0.walletId == id && $0.assetId == vault.nativeAsset.id })
    }

    public func unspentAllocations(of assetId: String) -> [Allocation] {
        vault.balances.filter { $0.walletId == id }.flatMap { $0.unspentAllocations }
    }

    public func sync() throws {
        let balanceData = try vault.balance(walletId: id)
        try balanceData.forEach { (assetId, utxoSet) in
            var assetId = assetId
            if assetId == BitcoinNetwork.rgbAssetId {
                assetId = vault.network.nativeAssetId()
            }
            vault.balances.removeAll(where: { $0.walletId == id && $0.assetId == assetId })
            guard let asset = vault.assets[assetId] else {
                throw InternalDataInconsistency.unknownAssetId(assetId)
            }
            let allocations = utxoSet.map { Allocation(withAsset: asset, utxo: $0) }
            let balance = Balance(
                    withAsset: asset,
                    walletId: id,
                    unspent: allocations
            )
            vault.balances.append(balance)
        }
    }

    public func syncOperations() throws -> [TransferOperation] {
        try vault.operations(walletId: id)
    }

    public func nextAddress(legacySegWit legacy: Bool = false) throws -> AddressDerivation {
        try vault.nextAddress(forContractId: id, useLegacySegWit: legacy)
    }

    public func refreshAddresses() throws {
        usedAddresses = (try? vault.usedAddresses(forContractId: id)) ?? usedAddresses
    }

    public private(set) lazy var usedAddresses: [AddressDerivation] = (try? vault.usedAddresses(forContractId: id)) ?? []

    public func addressBitcoins(_ address: String) -> Double {
        addressAllocations(address).bitcoinBalance(network: vault.network)
    }

    public func addressAllocations(_ address: String) -> [Allocation] {
        allBalances.flatMap { (_, balance) in
            balance.unspentAllocations.filter { $0.address == address }
        }
    }

    public func invoice(usingFormat format: InvoiceType, nominatedIn asset: Asset, amount: Double?, useLegacySegWit legacy: Bool = false) throws -> String {
        let assetId = asset.isNative ? nil : asset.id
        let value = amount != nil ? asset.amount(toAtoms: amount!) : nil
        return try vault.invoice(usingFormat: format, receiveTo: id, nominatedIn: assetId, value: value, useLegacySegWit: legacy)
    }

    /*
    public func mark(address: String, used: Bool = true) throws {
        try vault.mark(address: address, used: used)
    }

    public func mark(invoice: String, used: Bool = true) throws {
        try vault.mark(invoice: invoice, used: used)
    }
     */

    public func pay(invoice: String, fee: UInt64, giveaway: UInt64? = nil) throws -> PaymentResult {
        let transfer = try vault.pay(from: id, invoice: invoice, fee: fee, giveaway: giveaway)
        let signedPsbt = try vault.sign(psbt: transfer.psbt)
        let txid = try vault.publish(psbt: signedPsbt)

        return PaymentResult(txid: txid, consignment: transfer.consignment)
    }

    public func accept(consignment: String) throws -> String {
        try vault.accept(consignment: consignment)
    }
}

public struct Balance {
    public let walletId: String
    public let assetId: String
    public let totalInAtoms: UInt64
    public let total: Double
    public let unspentAllocations: [Allocation]

    internal init(withAsset asset: Asset, walletId: String, unspent: [Allocation]) {
        self.walletId = walletId
        assetId = asset.id
        totalInAtoms = unspent.reduce(into: 0) { sum, u in sum += u.valueInAtoms }
        total = asset.amount(fromAtoms: totalInAtoms)
        unspentAllocations = unspent
    }
}

public struct Allocation {
    public let assetId: String
    public let txid: String
    public let vout: UInt16
    public let valueInAtoms: UInt64
    public let amount: Double
    public let address: String?

    public var outpoint: OutPoint {
        OutPoint(txid: txid, vout: vout)
    }

    internal init(withAsset asset: Asset, utxo: UTXOJson) {
        assetId = asset.id
        txid = utxo.txid
        vout = utxo.vout
        valueInAtoms = utxo.value
        amount = asset.amount(fromAtoms: utxo.value)
        address = utxo.address
    }
}

extension Array where Element == Allocation {
    public func bitcoinBalance(network: BitcoinNetwork) -> Double {
        filter { $0.assetId == network.nativeAssetId() }.reduce(into: 0) { (sum: inout Double, allocation: Allocation) in
            sum = sum + allocation.amount
        }
    }

    public func bitcoinBalance(forOutpoint outpoint: OutPoint, network: BitcoinNetwork) -> Double {
        filter { $0.assetId == network.nativeAssetId() && $0.outpoint == outpoint }
                .reduce(into: 0) { (sum: inout Double, allocation: Allocation) in
            sum = sum + allocation.amount
        }
    }

    public func satoshisBalance(network: BitcoinNetwork) -> UInt64 {
        filter { $0.assetId == network.nativeAssetId() }.reduce(into: 0) { (sum: inout UInt64, allocation: Allocation) in
            sum = sum + allocation.valueInAtoms
        }
    }

    public func satoshisBalance(forOutpoint outpoint: OutPoint, network: BitcoinNetwork) -> UInt64 {
        filter { $0.assetId == network.nativeAssetId() && $0.outpoint == outpoint }
                .reduce(into: 0) { (sum: inout UInt64, allocation: Allocation) in
            sum = sum + allocation.valueInAtoms
        }
    }

    public var assetBalances: [String: Double] {
        self.reduce(into: [:]) { (result: inout [String: Double], allocation: Allocation) in
            result[allocation.assetId] = (result[allocation.assetId] ?? 0) + allocation.amount
        }
    }

    public func assetBalances(forOutpoint outpoint: OutPoint) -> [String: Double] {
        filter { $0.outpoint == outpoint }
                .reduce(into: [:]) { (result: inout [String: Double], allocation: Allocation) in
            result[allocation.assetId] = (result[allocation.assetId] ?? 0) + allocation.amount
        }
    }

    public var outpointBalances: [OutPoint: [String: Double]] {
        self.reduce(into: [:]) { (result: inout [OutPoint: [String: Double]], allocation: Allocation) in
            result[allocation.outpoint] = assetBalances(forOutpoint: allocation.outpoint)
        }
    }
}

/*
    public let assetTicker: String
    public let assetName: String
    public let fractionDigits: UInt8
 */

public struct IncomingTransfer: Codable {
    public let giveaway: UInt64
    public let inputDerivationIndexes: [UInt32]

    private enum CodingKeys: String, CodingKey {
        case giveaway, inputDerivationIndexes = "input_derivation_indexes"
    }
}

public struct OutcomingTransfer: Codable {
    public let published: Bool
    public let assetChange: UInt64
    public let bitcoinChange: UInt64
    public let changeOutputs: [UInt16]
    public let giveaway: UInt64?
    public let paidBitcoinFee: UInt64
    public let outputDerivationIndexes: [UInt32]
    public let invoice: String

    private enum CodingKeys: String, CodingKey {
        case published, assetChange = "asset_change", bitcoinChange = "bitcoin_change", changeOutputs = "change_outputs",
             giveaway = "giveaway", paidBitcoinFee = "paid_bitcoin_fee",
             outputDerivationIndexes = "output_derivation_indexes", invoice = "invoice"
    }
}

public enum TransferDirection: Codable {
    case incoming(IncomingTransfer)
    case outcoming(OutcomingTransfer)

    enum CodingKeys: CodingKey {
        case incoming, outcoming
    }

    public init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        if container.contains(.incoming) {
            let value = try container.decode(IncomingTransfer.self, forKey: .incoming)
            self = .incoming(value)
        } else if container.contains(.outcoming) {
            let value = try container.decode(OutcomingTransfer.self, forKey: .outcoming)
            self = .outcoming(value)
        } else {
            throw DecodingError.typeMismatch(
                    IncomingTransfer.self,
                    DecodingError.Context(codingPath: [CodingKeys.incoming], debugDescription: "IncomingTransfer value expected")
            )
        }
    }

    public func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)
        switch self {
        case .incoming(let value):
            try container.encode(value, forKey: .incoming)
        case .outcoming(let value):
            try container.encode(value, forKey: .outcoming)
        }
    }
}

public struct TransferOperation: Codable, Identifiable {
    public var id: String {
        txid
    }

    public let direction: TransferDirection
    public let createdAt: Date
    public let height: Int64
    public let assetId: String?
    public let balanceBefore: UInt64
    public let bitcoinVolume: UInt64
    public let assetVolume: UInt64
    public let txFee: UInt64
    public let txid: String
    public let psbt: String
    public let consignment: String?
    public var notes: String?
    public let value: UInt64
    public let bitcoinValue: UInt64

    public var isOutcoming: Bool {
        if case .outcoming(_) = direction { return true } else { return false }
    }

    public var bitcoinAmount: Double {
        Double(bitcoinValue) / 100_000_000.0
    }

    private enum CodingKeys: String, CodingKey {
        case direction, createdAt = "created_at", height, assetId = "asset_id", balanceBefore = "balance_before",
             bitcoinVolume = "bitcoin_volume", assetVolume = "asset_volume", txFee = "tx_fee",
             txid, psbt, consignment, notes, value = "asset_value", bitcoinValue = "bitcoin_value"
    }
}
