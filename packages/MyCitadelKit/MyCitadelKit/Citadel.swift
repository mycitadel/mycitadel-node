//
// Created by Maxim Orlovsky on 2/22/21.
//

import Foundation

public struct Citadel {
    internal let client: MyCitadelClient

    public var contracts: [WalletContract] = []
    public var assets: [String: RGB20Asset] = [:]

    internal init(withClient client: MyCitadelClient) {
        self.client = client
        try? self.syncAll()
    }

    public mutating func syncAll() throws {
        let _ = try self.syncContracts()
        let _ = try self.syncAssets()
    }

    public mutating func syncContracts() throws -> [WalletContract] {
        let contractData: [ContractData] = try client.listContracts()
        self.contracts = contractData.map { WalletContract(withClient: client, contractData: $0) }

        for var contract in self.contracts {
            try contract.sync()
        }
        return self.contracts
    }

    public mutating func syncAssets() throws -> [String:RGB20Asset] {
        let assetData: [AssetData] = try client.listAssets()
        let network = client.network
        self.assets = [
            BitcoinNetwork.rgbAssetId: RGB20Asset(withAssetData: AssetData(
                    genesis: network.geneisHash(),
                    id: BitcoinNetwork.rgbAssetId,
                    ticker: network.ticker(),
                    name: network.coinName(),
                    description: nil,
                    fractionalBits: 8,
                    date: network.genesisDate(),
                    knownCirculating: network.issuedSupply(),
                    issueLimit: network.issueLimit()
            ))
        ]
        assetData.forEach { self.assets[$0.id] = RGB20Asset(withAssetData: $0) }
        return self.assets
    }

    public mutating func createSingleSig(named name: String, descriptor descriptorType: DescriptorType, enableRGB hasRGB: Bool) throws {
        let pubkeyChain = descriptorType.createPubkeyChain(network: client.network, rgb: hasRGB, multisig: false, scope: nil)
        let contractData = try self.client.create(singleSig: pubkeyChain, name: name, descriptorType: descriptorType)
        self.contracts.append(WalletContract(withClient: self.client, contractData: contractData))
    }

    public mutating func importAsset(genesisBech32 genesis: String) throws -> RGB20Asset {
        let assetData = try self.client.importAsset(bech32: genesis)
        return RGB20Asset(withAssetData: assetData)
    }
}

public struct WalletContract {
    internal let client: MyCitadelClient

    public let id: String
    public var name: String
    public let chain: BitcoinNetwork
    public let policy: Policy
    private var balances: [String: Balance]

    internal init(withClient client: MyCitadelClient, contractData: ContractData) {
        self.client = client
        self.id = contractData.id
        self.name = contractData.name
        self.chain = contractData.chain
        self.policy = contractData.policy
        self.balances = [
            "rgb1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqg40adx":
            Balance(withAssetId: "rgb1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqg40adx", total: 0, unspent: [])
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

    internal init(withAssetId assetId: String, utxo: UTXO) {
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