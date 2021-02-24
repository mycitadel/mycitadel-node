//
// Created by Maxim Orlovsky on 2/22/21.
//

import Foundation

public protocol VaultAPI: ObservableObject {
    var network: BitcoinNetwork { get }
    var blockchainState: BlockchainState { get }
    var mempoolState: MempoolState { get }
    var contracts: [WalletContract] { get }
    var assets: [String: Asset] { get }
    var balances: [Balance] { get }

    func syncAll() throws
    func syncContracts() throws -> [WalletContract]
    func syncAssets() throws -> [String: Asset]
    func createSingleSig(named name: String, descriptor descriptorType: DescriptorType, enableRGB hasRGB: Bool) throws
    func importAsset(fromString assetString: String) throws -> Asset
}

extension CitadelVault: VaultAPI {
    public func syncAll() throws {
        let _ = try self.syncContracts()
        let _ = try self.syncAssets()
    }

    public func syncContracts() throws -> [WalletContract] {
        let contractData: [ContractJson] = try listContracts()
        self.contracts = contractData.map {
            WalletContract(withContractData: $0, citadelVault: self)
        }

        for contract in self.contracts {
            try contract.sync()
        }
        return self.contracts
    }

    public func syncAssets() throws -> [String: Asset] {
        let assetData: [RGB20Json] = try listAssets()
        for assetUpdate in assetData {
            if let asset = assets[assetUpdate.id] as? RGB20Asset {
                asset.knownIssued = assetUpdate.knownCirculating
                asset.maxIssued = assetUpdate.issueLimit ?? UInt64.max
            } else {
                assets[assetUpdate.id] = RGB20Asset(withAssetData: assetUpdate, citadelVault: self)
            }
        }
        // TODO: Update balance
        if !assets.keys.contains(BitcoinNetwork.rgbAssetId) {
            assets[BitcoinNetwork.rgbAssetId] = NativeAsset(withCitadelVault: self)
        }
        return assets
    }

    public func createSingleSig(named name: String, descriptor descriptorType: DescriptorType, enableRGB hasRGB: Bool) throws {
        let pubkeyChain = descriptorType.createPubkeyChain(network: network, rgb: hasRGB, multisig: false, scope: nil)
        let contractData = try self.create(singleSig: pubkeyChain, name: name, descriptorType: descriptorType)
        self.contracts.append(WalletContract(withContractData: contractData, citadelVault: self))
    }

    public func importAsset(fromString assetString: String) throws -> Asset {
        let assetData = try self.importRGB(genesisBech32: assetString)
        return RGB20Asset(withAssetData: assetData, citadelVault: self)
    }
}

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

    public func balance(of assetId: String) -> Double {
        guard let asset = vault.assets[assetId] else { return 0 }
        return asset.amount(fromAtoms: balanceInAtoms(of: assetId))
    }

    public func balanceInAtoms(of assetId: String) -> UInt64 {
        vault.balances.filter { $0.walletId == id && $0.assetId == assetId }.reduce(0) { sum, balance in sum + balance.total }
    }

    public func unspentAllocations(of assetId: String) -> [Allocation] {
        vault.balances.filter { $0.walletId == id }.flatMap { $0.unspentAlocations }
    }

    public func sync() throws {
        let balanceData = try vault.balance(walletId: self.id)
        balanceData.forEach { (assetId, utxoSet) in
            vault.balances.removeAll(where: { $0.walletId == id && $0.assetId == assetId })
            let allocations = utxoSet.map { Allocation(withAssetId: assetId, utxo: $0) }
            let total = allocations.reduce(into: 0) { sum, u in sum += u.value }
            vault.balances.append(Balance(withWalletId: id, assetId: assetId, total: total, unspent: allocations))
        }
    }
}

public struct Balance {
    public let walletId: String
    public let assetId: String
    public let total: UInt64
    public let unspentAlocations: [Allocation]

    internal init(withWalletId walletId: String, assetId: String, total: UInt64, unspent: [Allocation]) {
        self.walletId = walletId
        self.assetId = assetId
        self.total = total
        self.unspentAlocations = unspent
    }
}

public struct Allocation {
    public let assetId: String
    public let txid: String
    public let vout: UInt32
    public let value: UInt64
    public let address: String?

    internal init(withAssetId assetId: String, utxo: UTXOJson) {
        self.assetId = assetId
        self.txid = utxo.txid
        self.vout = utxo.vout
        self.value = utxo.value
        self.address = utxo.address
    }
}

/*
    public let assetTicker: String
    public let assetName: String
    public let fractionDigits: UInt8
 */