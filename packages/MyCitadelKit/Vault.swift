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

public struct CitadelError: Error {
    let errNo: Int
    let message: String

    init(errNo: Int, message: String) {
        self.errNo = errNo
        self.message = message
    }

    init(_ message: String) {
        self.errNo = 1000
        self.message = message
    }
}

extension CitadelError: CustomStringConvertible {
    public var description: String {
        self.message
    }
}

extension CitadelError: LocalizedError {
    public var errorDescription: String? {
        self.message
    }
}

open class CitadelVault {
    private static var node: CitadelVault? = nil
    public static var embedded: CitadelVault! {
        CitadelVault.node
    }

    public static func runEmbeddedNode(
            connectingNetwork network: BitcoinNetwork,
            rgbNode: String? = nil,
            lnpNode: String? = nil,
            electrumServer: String = "pandora.network:60001"
    ) throws {
        try Self.node = CitadelVault(withEmbeddedNodeConnectingNetwork: network,
                rgbNode: rgbNode, lnpNode: lnpNode, electrumServer: electrumServer)
    }

    internal private(set) var rpcClient: UnsafeMutablePointer<mycitadel_client_t>!

    let dataDir: String
    public let network: BitcoinNetwork
    public var nativeAsset: NativeAsset {
        assets[network.nativeAssetId()] as! NativeAsset
    }
    @Published
    public var blockchainState = BlockchainState()
    @Published
    public var mempoolState = MempoolState()
    @Published
    public var contracts: [WalletContract] = []
    @Published
    public var assets: [String: Asset] = [:]
    @Published
    public var balances: [Balance] = []

    public init(
            connectingCitadelNode citadelNode: String,
            electrumServer: String = "pandora.network:60001",
            onNetwork network: BitcoinNetwork
    ) {
        // TODO: Implement connected mode
        fatalError("Connected mode is not yet implemented")
    }

    public init(
            withEmbeddedNodeConnectingNetwork network: BitcoinNetwork,
            rgbNode: String? = nil,
            lnpNode: String? = nil,
            electrumServer: String = "pandora.network:60001"
    ) throws {
        self.network = network
        dataDir = FileManager.default.urls(for: .documentDirectory, in: .userDomainMask).first!.appendingPathComponent(network.rawValue).path
        rpcClient = mycitadel_run_embedded(network.rawValue, self.dataDir, electrumServer)
        assets[network.nativeAssetId()] = NativeAsset(withCitadelVault: self)
    }

    deinit {
        // TODO: Teardown client
        // mycitadel_shutdown(self.client)
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
