//
// Created by Maxim Orlovsky on 2/22/21.
//

import Foundation

public struct MyCitadel {
    internal let client: MyCitadelClient

    public var contracts: [WalletContract] = []
    public var assets: [RGB20Asset] = []

    internal init(withClient client: MyCitadelClient) {
        self.client = client
        self.sync()
    }

    public mutating func sync() {
        let contractData: [ContractData] = (try? client.listContracts()) ?? []
        self.contracts = contractData.map { WalletContract(withClient: client, contractData: $0) }

        for var contract in self.contracts {
            contract.sync()
        }

        let assetData: [AssetData] = (try? client.listAssets()) ?? []
        self.assets = assetData.map(RGB20Asset.init)
    }
}

public struct WalletContract {
    internal let client: MyCitadelClient

    public let id: String
    public var name: String
    public let chain: BitcoinNetwork
    public let policy: String
    private var balances: [String: Balance] = [:]

    internal init(withClient client: MyCitadelClient, contractData: ContractData) {
        self.client = client
        self.id = contractData.id
        self.name = contractData.name
        self.chain = contractData.chain
        self.policy = contractData.policy
    }

    public func balance(of assetId: String) -> UInt64 {
        self.balances[assetId]?.total ?? 0
    }

    public mutating func sync() {
        let balanceData = (try? client.balance(walletId: self.id)) ?? [:]
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