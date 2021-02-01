//
//  MyCitadelKit.swift
//  MyCitadelKit
//
//  Created by Maxim Orlovsky on 1/31/21.
//

import Foundation

public enum Network: String {
    case Mainnet = "mainnet"
    case Testnet = "testnet"
    case Signet = "signet"
}

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

public struct Asset: Decodable {
    public let id: String
    public let ticker: String
    public let name: String
    public let description: String?
    public let fractionalBits: Int8
}

open class MyCitadelClient {
    let network: Network
    let dataDir: String
    private var client: UnsafeMutablePointer<mycitadel_client_t>!
    
    private init(network: Network = .Signet, electrumServer: String = "pandora.network:60001") {
        self.network = network
        self.dataDir = FileManager.default.urls(for: .documentDirectory, in: .userDomainMask).first!.path

        self.client = mycitadel_run_embedded(network.rawValue, self.dataDir, electrumServer)
    }
    
    private static var _shared: MyCitadelClient? = nil
    public static var shared: MyCitadelClient? {
        get { Self._shared }
    }
    
    public static func run(network: Network = .Signet, electrumServer: String = "pandora.network:60001") throws {
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
        return Data(String(cString: json).utf8)
    }
    
    public func refreshAssets() throws -> [Asset] {
        let response = mycitadel_list_assets(client);
        return try JSONDecoder().decode([Asset].self, from: self.processResponse(response))
    }
    
    public func importAsset(bech32 genesis: String) throws -> Asset {
        let response = mycitadel_import_asset(client, genesis);
        return try JSONDecoder().decode(Asset.self, from: self.processResponse(response))
    }
}
