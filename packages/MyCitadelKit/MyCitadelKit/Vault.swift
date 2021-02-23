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
        self.contracts = contractData.map { WalletContract(withClient: self, contractData: $0) }

        for var contract in self.contracts {
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
        self.contracts.append(WalletContract(withClient: self, contractData: contractData))
    }

    public func importAsset(fromString assetString: String) throws -> Asset {
        let assetData = try self.importRGB(genesisBech32: assetString)
        return RGB20Asset(withAssetData: assetData, citadelVault: self)
    }
}

public struct WalletContract {
    internal let client: CitadelVault

    public let id: String
    public var name: String
    public let chain: BitcoinNetwork
    public let policy: Policy
    private var balances: [String: Balance]

    internal init(withClient client: CitadelVault, contractData: ContractJson) {
        self.client = client
        self.id = contractData.id
        self.name = contractData.name
        self.chain = contractData.chain
        self.policy = contractData.policy
        self.balances = [
            BitcoinNetwork.rgbAssetId:
            Balance(withAssetId: BitcoinNetwork.rgbAssetId, total: 0, unspent: [])
        ]
        if let balances = try? client.balance(walletId: id) {
            for (assetId, utxoSet) in balances {
                self.balances[assetId] = Balance(
                    withAssetId: assetId,
                    total: utxoSet.reduce(0) { $0 + $1.value },
                    unspent: utxoSet.map { UnspentCoins(withAssetId: assetId, utxo: $0) }
                )
            }
        }
    }

    public func balance(of assetId: String) -> UInt64 {
        self.balances[assetId]?.total ?? 0
    }

    public mutating func sync() throws {
        let balanceData = try client.balance(walletId: self.id)
        balanceData.forEach { (assetId, utxoSet) in
            let unspent = utxoSet.map { UnspentCoins(withAssetId: assetId, utxo: $0) }
            let total = unspent.reduce(into: 0) { sum, u in sum += u.value }
            self.balances[assetId] = Balance(withAssetId: assetId, total: total, unspent: unspent)
        }
    }
}

public struct Balance {
    public let assetId: String
    public let total: UInt64
    public let unspent: [UnspentCoins]

    internal init(withAssetId assetId: String, total: UInt64, unspent: [UnspentCoins]) {
        self.assetId = assetId
        self.total = total
        self.unspent = unspent
    }
}

public struct UnspentCoins {
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