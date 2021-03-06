//
// Created by Maxim Orlovsky on 2/22/21.
//

import Foundation

public enum InternalDataInconsistency {
    case unknownAssetId(String)
}

extension InternalDataInconsistency: Error {
    public var localizedDescription: String {
        switch self {
        case .unknownAssetId(let id):
            return "Unknown asset with id \(id)"
        }
    }
}

public protocol VaultAPI: ObservableObject {
    var network: BitcoinNetwork { get }
    var blockchainState: BlockchainState { get }
    var mempoolState: MempoolState { get }
    var contracts: [WalletContract] { get }
    var assets: [String: Asset] { get }
    var balances: [Balance] { get }

    func contract(id: String) -> WalletContract?

    func syncAll() throws
    func syncContracts() throws -> [WalletContract]
    func syncAssets() throws -> [String: Asset]
    func createSingleSig(named name: String, descriptor descriptorType: DescriptorType, enableRGB hasRGB: Bool) throws
    func importAsset(fromString assetString: String) throws -> Asset
}

extension CitadelVault: VaultAPI {
    public func contract(id: String) -> WalletContract? {
        contracts.first(where: { $0.id == id })
    }

    public func syncAll() throws {
        let _ = try syncAssets()
        let _ = try syncContracts()
    }

    public func syncContracts() throws -> [WalletContract] {
        let contractData: [ContractJson] = try listContracts()
        contracts = contractData.map {
            WalletContract(withContractData: $0, citadelVault: self)
        }

        for contract in contracts {
            try contract.sync()
        }
        print("Contracts synced: \(contracts)")
        return contracts
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
        if !assets.keys.contains(network.nativeAssetId()) {
            assets[network.nativeAssetId()] = NativeAsset(withCitadelVault: self)
        }
        print("Assets synced: \(assets)")
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
