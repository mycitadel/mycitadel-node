//
//  CAPI.swift
//  MyCitadelKit
//
//  Created by Maxim Orlovsky on 1/31/21.
//

import os
import Foundation
import Combine

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

    private var rpcClient: UnsafeMutablePointer<mycitadel_client_t>!

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

    public func lastError() -> CitadelError? {
        if mycitadel_has_err(rpcClient) {
            return CitadelError(errNo: Int(rpcClient.pointee.err_no), message: String(cString: rpcClient.pointee.message))
        } else {
            return nil
        }
    }

    private func processResponseToString(_ response: UnsafePointer<Int8>?) throws -> String {
        guard let response = response else {
            guard let err = self.lastError() else {
                throw CitadelError("MyCitadel C API is broken")
            }
            throw err
        }
        var string = String(cString: response)
        // TODO: Remove this debug printing
        print(string)
        string.reserveCapacity(string.count * 2)
        release_string(UnsafeMutablePointer(mutating: response))
        return string
    }

    private func processResponse(_ response: UnsafePointer<Int8>?) throws -> Data {
        try Data(processResponseToString(response).utf8)
    }
}

extension CitadelVault: CitadelRPC {
    internal func create(singleSig derivation: String, name: String, descriptorType: DescriptorType) throws -> ContractJson {
        try self.createSeed()
        let pubkeyChain = try self.createScopedChain(derivation: derivation)
        let response = mycitadel_single_sig_create(rpcClient, name, pubkeyChain, descriptorType.cDescriptorType());
        return try JSONDecoder().decode(ContractJson.self, from: self.processResponse(response))
    }

    internal func listContracts() throws -> [ContractJson] {
        print("Listing contracts")
        let response = mycitadel_contract_list(rpcClient)
        return try JSONDecoder().decode([ContractJson].self, from: self.processResponse(response))
    }

    internal func balance(walletId: String) throws -> [String: [UTXOJson]] {
        print("Requesting balance for \(walletId)")
        let response = mycitadel_contract_balance(rpcClient, walletId, true, 20)
        return try JSONDecoder().decode([String: [UTXOJson]].self, from: self.processResponse(response))
    }

    internal func listAssets() throws -> [RGB20Json] {
        print("Listing assets")
        let response = mycitadel_asset_list(rpcClient);
        return try JSONDecoder().decode([RGB20Json].self, from: self.processResponse(response))
    }

    internal func importRGB(genesisBech32 genesis: String) throws -> RGB20Json {
        let response = mycitadel_asset_import(rpcClient, genesis);
        return try JSONDecoder().decode(RGB20Json.self, from: self.processResponse(response))
    }

    public func address(forContractId contractId: String, useLegacySegWit legacy: Bool = false) throws -> AddressDerivation {
        let response = mycitadel_address_create(rpcClient, contractId, false, legacy)
        return try JSONDecoder().decode(AddressDerivation.self, from: self.processResponse(response))
    }

    internal func invoice(usingFormat format: InvoiceType, receiveTo contractId: String, nominatedIn assetId: String?, value: UInt64?, useLegacySegWit legacy: Bool = false) throws -> String {
        let invoice = mycitadel_invoice_create(rpcClient, format.cType(), contractId, assetId ?? nil, value ?? 0, nil, nil, false, legacy)
        return try processResponseToString(invoice)
    }

    /*
    public func mark(addressUnused address: String) throws {
        mycitadel_mark
    }

    public func mark(invoiceUnused invoice: String) throws {
        try vault.mark(invoice: invoice, used: used)
    }
     */

    internal func pay(from contractId: String, invoice: String, fee: UInt64, giveaway: UInt64? = nil) throws -> Transfer {
        let transfer = mycitadel_invoice_pay(rpcClient, contractId, invoice, fee, giveaway ?? 0)
        if !transfer.success {
            guard let err = self.lastError() else {
                throw CitadelError("MyCitadel C API is broken")
            }
            throw err
        }
        let psbt = String(cString: transfer.psbt_base64)
        release_string(UnsafeMutablePointer(mutating: transfer.psbt_base64))

        let consignment: String?
        if transfer.consignment_bech32 != nil {
            consignment = String(cString: transfer.consignment_bech32)
            release_string(UnsafeMutablePointer(mutating: transfer.psbt_base64))
        } else {
            consignment = nil
        }

        return Transfer(psbt: psbt, consignment: consignment)
    }

    internal func publish(psbt: String) throws -> String {
        let txid = mycitadel_psbt_publish(rpcClient, psbt)
        return try processResponseToString(txid)

    }
}
