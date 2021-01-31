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

public struct MyCitadelError {
    let errNo: Int
    let message: String

    /*
    init(_ err: mycitadel_error_t) {
        self.errNo = Int(err.err_no)
        self.message = String(cString: err.message)
    }
 */
    
    init(_ msg: String) {
        self.errNo = 1000
        self.message = msg
    }
}

open class MyCitadelClient {
    let network: Network
    let dataDir: String
    private var client: UnsafeMutablePointer<mycitadel_client_t>
    
    public init(network: Network = .Signet, electrumServer: String = "pandora.network:60001") {
        self.network = network
        self.dataDir = FileManager.default.urls(for: .documentDirectory, in: .userDomainMask).first!.path

        self.client = mycitadel_run_embedded(network.rawValue, self.dataDir, electrumServer)
    }
}
