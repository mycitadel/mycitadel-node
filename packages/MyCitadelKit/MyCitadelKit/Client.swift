//
//  Client.swift
//  MyCitadelKit
//
//  Created by Maxim Orlovsky on 1/31/21.
//

import os
import Foundation

public struct MyCitadelError: Error {
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

extension MyCitadelError: CustomStringConvertible {
    public var description: String {
        self.message
    }
}

extension MyCitadelError: LocalizedError {
    public var errorDescription: String? {
        self.message
    }
}

open class MyCitadelClient {
    private var client: UnsafeMutablePointer<mycitadel_client_t>!

    public let network: BitcoinNetwork
    let dataDir: String

    private var _data: Citadel? = nil
    public var citadel: Citadel {
        if _data == nil {
            _data = Citadel(withClient: self)
        }
        return _data!
    }

    private init(network: BitcoinNetwork = .testnet, electrumServer: String = "pandora.network:60001") {
        self.network = network
        self.dataDir = FileManager.default.urls(for: .documentDirectory, in: .userDomainMask).first!.appendingPathComponent(network.rawValue).path

        self.client = mycitadel_run_embedded(network.rawValue, self.dataDir, electrumServer)
    }
    
    private static var _shared: MyCitadelClient? = nil
    public static var shared: MyCitadelClient! {
        get { Self._shared }
    }
    
    public static func connect(network: BitcoinNetwork = .testnet, electrumServer: String = "pandora.network:60001") throws {
        if Self._shared == nil {
            Self._shared = MyCitadelClient(network: network, electrumServer: electrumServer)
        } else {
            throw MyCitadelError("MyCitadelClient is already running")
        }
    }
    
    public func lastError() -> MyCitadelError? {
        if mycitadel_has_err(client) {
            return MyCitadelError(errNo: Int(client.pointee.err_no), message: String(cString: client.pointee.message))
        } else {
            return nil
        }
    }
    
    private func processResponse(_ response: UnsafePointer<Int8>?) throws -> Data {
        guard let json = response else {
            guard let err = self.lastError() else {
                throw MyCitadelError("MyCitadelClient API is broken")
            }
            throw err
        }
        // TODO: Remove debugging print here
        print(String(cString: json))
        let data = Data(String(cString: json).utf8)
        release_string(UnsafeMutablePointer(mutating: response))
        return data
    }

    internal func create(singleSig derivation: String, name: String, descriptorType: DescriptorType) throws -> ContractData {
        try self.createSeed()
        let pubkeyChain = try self.createIdentity(derivation: derivation)
        let response = mycitadel_single_sig_create(client, name, pubkeyChain, descriptorType.cDescriptorType());
        return try JSONDecoder().decode(ContractData.self, from: self.processResponse(response))
    }

    internal func listContracts() throws -> [ContractData] {
        let response = mycitadel_contract_list(client)
        return try JSONDecoder().decode([ContractData].self, from: self.processResponse(response))
    }

    internal func balance(walletId: String) throws -> [String: [UTXO]] {
        let response = mycitadel_contract_balance(client, walletId, true, 20)
        return try JSONDecoder().decode([String: [UTXO]].self, from: self.processResponse(response))
    }

    internal func listAssets() throws -> [AssetData] {
        let response = mycitadel_asset_list(client);
        return try JSONDecoder().decode([AssetData].self, from: self.processResponse(response))
    }

    internal func importAsset(bech32 genesis: String) throws -> AssetData {
        let response = mycitadel_asset_import(client, genesis);
        return try JSONDecoder().decode(AssetData.self, from: self.processResponse(response))
    }
}
