//
//  RGBHelpers.swift
//  MyCitadelKit
//
//  Created by Maxim Orlovsky on 2/2/21.
//

import Foundation

open class UniversalParser {
    public enum ParsedData {
        case unknown
        case url

        case address(AddressInfo)
        case wifPrivateKey
        case xpub
        case xpriv

        case derivation
        case descriptor
        case miniscript
        case script

        case bolt11Invoice
        case lnpbpId
        case lnpbpData
        case lnpbpZData
        case lnbpInvoice(Invoice)
        case rgbSchemaId
        case rgbContractId
        case rgbSchema
        case rgbGenesis
        case rgbConsignment
        case rgb20Asset(RGB20Asset)

        case outpoint(OutPoint)
        case hash160(Data)
        case genesis(BitcoinNetwork)
        case hex256(Data)

        case transaction
        case psbt

        case bech32Unknown(hrp: String, payload: String, data: Data)
        case base64Unknown(Data)
        case base58Unknown(Data)
        case hexUnknown(Data)
    }

    public struct ParseError: Error {
        public let type: ParseStatus
        public let message: String
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
    public var error: ParseError? {
        isOk ? nil : ParseError(type: parseStatus, message: parseReport)
    }
    public let parseStatus: ParseStatus
    public let parseReport: String
    public let parsedData: ParsedData
    
    public init(_ text: String) {

        if let address = try? UniversalParser.parse(address: text) {
            parsedData = .address(address)
            parseStatus = .ok
            parseReport = "Address parsed successfully"
            return
        }

        do {
            parsedData = try UniversalParser.parse(bech32: text)
            parseStatus = .ok
            parseReport = "Bech32 string parsed successfully"
        } catch let error where error is ParseError {
            let parseError = error as! ParseError
            parsedData = .unknown
            parseStatus = parseError.type
            parseReport = parseError.message
        } catch {
            parsedData = .unknown
            parseStatus = .invalidJSON
            parseReport = "Unable to recognize details from the provided JSON data"
            print("Bech32 parse error \(error.localizedDescription)")
        }

        // TODO: Parse descriptors
        // TODO: Parse derivation strings
        // TODO: Parse outpoint
        // TODO: Parse hex + transaction
        // TODO: Parse Base58 (private keys included)
        // TODO: Parse Base64
    }

    public static func parse(address: String) throws -> AddressInfo {
        let info = lnpbp_address_parse(address)
        defer {
            result_destroy(info)
        }
        if !is_success(info) {
            let errorMessage = String(cString: info.details.error)
            print("Error parsing address: \(errorMessage)")
            throw CitadelError(errorMessage)
        }
        let jsonString = String(cString: info.details.data)
        let jsonData = Data(jsonString.utf8)
        let decoder = JSONDecoder();
        print("Parsing JSON address data: \(jsonString)")
        do {
            return try decoder.decode(AddressInfo.self, from: jsonData)
        } catch {
            print("Error parsing address: \(error.localizedDescription)")
            throw error
        }
    }

    public static func parse(bech32: String) throws -> ParsedData {
        let info = lnpbp_bech32_info(bech32)

        let jsonString = String(cString: info.details)
        let jsonData = Data(jsonString.utf8)
        let decoder = JSONDecoder();
        print("Parsing JSON Bech32 data: \(jsonString)")

        if info.status != 0 {
            throw ParseError(type: ParseStatus(rawValue: info.status)!, message: String(cString: info.details))
        }

        switch info.category {
        case BECH32_RGB20_ASSET:
            let assetData = try decoder.decode(RGB20Json.self, from: jsonData)
            return ParsedData.rgb20Asset(RGB20Asset(withAssetData: assetData, citadelVault: CitadelVault.embedded))
        case BECH32_LNPBP_INVOICE:
            let invoice = try decoder.decode(Invoice.self, from: jsonData)
            return ParsedData.lnbpInvoice(invoice)
        default: return ParsedData.unknown
        }
    }
}
