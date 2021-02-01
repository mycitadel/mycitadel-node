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

    init(errNo errNo: Int, message: String) {
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
    
    public func refreshAssets() throws -> [Asset] {
        guard let json = mycitadel_list_assets(client) else {
            guard let err = self.lastError() else {
                throw MyCitadelError("MyCitadelClient API is broken")
            }
            throw err
        }
        let jsonData = Data(String(cString: json).utf8)
        return try JSONDecoder().decode([Asset].self, from: jsonData)
    }
}
