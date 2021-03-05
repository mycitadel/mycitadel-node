//
// Created by Maxim Orlovsky on 3/4/21.
//

import Foundation


public class WalletContract {
    internal let vault: CitadelVault

    public let id: String
    public var name: String
    public let chain: BitcoinNetwork
    public let policy: Policy

    init(withContractData contractData: ContractJson, citadelVault: CitadelVault) {
        self.vault = citadelVault
        self.id = contractData.id
        self.name = contractData.name
        self.chain = contractData.chain
        self.policy = contractData.policy
        try? self.sync()
    }

    public lazy var descriptorInfo: DescriptorInfo = try! parseDescriptor()

    public var availableAssetIds: [String] {
        Array(allBalances.keys)
    }

    public var availableAssets: [Asset] {
        availableAssetIds.map { vault.assets[$0]! }
    }

    public var allBalances: [String: Balance] {
        var balances: [String: Balance] = [:]
        vault.balances.filter { $0.walletId == id }.forEach { balances[$0.assetId] = $0 }
        return balances
    }

    public func balance(of assetId: String?) -> Balance? {
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
        let balanceData = try vault.balance(walletId: self.id)
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

    public func address(useLegacySegWit legacy: Bool = false) throws -> AddressDerivation {
        return try vault.address(forContractId: self.id, useLegacySegWit: legacy)
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
        self.assetId = asset.id
        self.totalInAtoms = unspent.reduce(into: 0) { sum, u in sum += u.valueInAtoms }
        self.total = asset.amount(fromAtoms: self.totalInAtoms)
        self.unspentAllocations = unspent
    }
}

public struct Allocation {
    public let assetId: String
    public let txid: String
    public let vout: UInt32
    public let valueInAtoms: UInt64
    public let amount: Double
    public let address: String?

    internal init(withAsset asset: Asset, utxo: UTXOJson) {
        self.assetId = asset.id
        self.txid = utxo.txid
        self.vout = utxo.vout
        self.valueInAtoms = utxo.value
        self.amount = asset.amount(fromAtoms: utxo.value)
        self.address = utxo.address
    }
}

/*
    public let assetTicker: String
    public let assetName: String
    public let fractionDigits: UInt8
 */