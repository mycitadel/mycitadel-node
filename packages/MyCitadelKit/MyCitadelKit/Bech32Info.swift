//
//  RGBHelpers.swift
//  MyCitadelKit
//
//  Created by Maxim Orlovsky on 2/2/21.
//

import Foundation

open class Bech32Info {
    public enum Details {
        case unknown
        case url
        case bcAddress
        case bolt11Invoice
        case lnpbpId
        case lnpbpData
        case lnpbpZData
        case lnbpInvoice
        case rgbSchemaId
        case rgbContractId
        case rgbSchema
        case rgbGenesis
        case rgbConsignment
        case rgb20Asset(RGB20Asset)

        public func name() -> String {
            switch self {
            case .unknown:
                return "Unknown"
            case .url:
                return "URL"
            case .bcAddress:
                return "Bitcoin address"
            case .bolt11Invoice:
                return "LN BOLT11 invoice"
            case .lnpbpId:
                return "LNPBP-39 id"
            case .lnpbpData:
                return "LNPBP-39 data"
            case .lnpbpZData:
                return "LNPBP-39 compressed data"
            case .lnbpInvoice:
                return "LNPBP-38 invoice"
            case .rgbSchemaId:
                return "RGB Schema Id"
            case .rgbContractId:
                return "RGB Contract Id"
            case .rgbSchema:
                return "RGB Schema"
            case .rgbGenesis:
                return "RGB Genesis"
            case .rgbConsignment:
                return "RGB Consignment"
            case .rgb20Asset(_):
                return "RGB20 Asset"
            }
        }
    }
    
    public enum ParseStatus: Int32 {
        case ok = 0
        case hrpErr = 1
        case checksumErr = 2
        case encodingErr = 3
        case payloadErr = 4
        case unsupportedErr = 5
        case internalErr = 6
        case invalidJSON = 0xFFFF
    }
    
    public var isOk: Bool {
        parseStatus == .ok
    }
    public var isBech32m: Bool
    public let parseStatus: ParseStatus
    public let parseReport: String
    public let details: Details
    
    public init(_ bech32: String) {
        let info = lnpbp_bech32_info(bech32)

        self.isBech32m = info.bech32m

        let jsonData = Data(String(cString: info.details).utf8)
        let decoder = JSONDecoder();

        do {
            switch info.category {
            case BECH32_RGB20_ASSET:
                self.details = Details.rgb20Asset(try decoder.decode(RGB20Asset.self, from: jsonData))
            default: self.details = Details.unknown
            }

            self.parseStatus = ParseStatus(rawValue: info.status)!
            self.parseReport = info.status == 0 ? "Bech32 parsed successfully" : String(cString: info.details)
        } catch {
            self.details = .unknown
            self.parseStatus = .invalidJSON
            self.parseReport = "Unable to recognize details from the provided JSON data"
        }
    }
}
