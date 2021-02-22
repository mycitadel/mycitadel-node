//
// Created by Maxim Orlovsky on 2/19/21.
//

import Foundation
import Security

private enum SecurityItemNames: String {
    case seed = "MyCitadel.seed"
    case masterXpriv = "MyCitadel.masterXpriv"
}

public struct SignerError: Error {
    let details: String
}

extension MyCitadelClient {
    private func checkKeychain(attrName: String) throws -> Bool {
        let query = [
            kSecClass: kSecClassGenericPassword,
            kSecAttrAccount: attrName,
            kSecAttrAccessible: kSecAttrAccessibleWhenUnlocked,
            kSecUseDataProtectionKeychain: true,
        ] as [String: Any]
        var foundItem: CFTypeRef?;

        switch SecItemCopyMatching(query as CFDictionary, &foundItem) {
        case errSecSuccess:
            guard let data = foundItem as? Data,
                  let _ = String(data: data, encoding: .utf8)
                    else { throw SignerError(details: "Wrong encoding of \(attrName) in the Apple Keychain") }
            return true
        case errSecItemNotFound: return false
        case let status: throw SignerError(details: "Keychain read failed: \(status.description)")
        }
    }

    private func readKeychain(attrName: String) throws -> String? {
        let query = [
            kSecClass: kSecClassGenericPassword,
            kSecAttrAccount: attrName,
            kSecAttrAccessible: kSecAttrAccessibleWhenUnlocked,
            kSecUseDataProtectionKeychain: true,
            kSecReturnData: true
        ] as [String: Any]
        var foundItem: CFTypeRef?;

        switch SecItemCopyMatching(query as CFDictionary, &foundItem) {
        case errSecSuccess:
            guard let data = foundItem as? Data,
                  let stringRepr = String(data: data, encoding: .utf8)
                    else { throw SignerError(details: "Wrong encoding of \(attrName) in the Apple Keychain") }
            return stringRepr
        case errSecItemNotFound: return nil
        case let status: throw SignerError(details: "Keychain read failed: \(status.description)")
        }
    }

    private func writeKeychain(attrName: String, data: String) throws {
        let query = [
            kSecClass: kSecClassGenericPassword,
            kSecAttrAccount: attrName,
            kSecAttrAccessible: kSecAttrAccessibleWhenUnlocked,
            kSecUseDataProtectionKeychain: true,
            kSecValueData: data
        ] as [String: Any]

        let status = SecItemAdd(query as CFDictionary, nil)
        if status == errSecSuccess {
            throw SignerError(details: "Unable to store seed information: \(status.description)")
        }
    }

    internal func createSeed() throws {
        if try self.checkKeychain(attrName: SecurityItemNames.seed.rawValue) {
            return // We already have a seed phrase
        }

        let result = bip39_mnemonic_create(nil, BIP39_MNEMONIC_TYPE_WORDS_12);
        if is_success(result) {
            throw SignerError(details: "Unable to generate seed: \(String(cString: result.details.error))")
        }
        let seedPhrase = String(cString: result.details.data);
        try self.writeKeychain(attrName: SecurityItemNames.seed.rawValue, data: seedPhrase)
        result_destroy(result)

        let xpriv_result = bip39_master_xpriv(UnsafeMutablePointer<Int8>(mutating: (seedPhrase as NSString).utf8String), nil, true, self.network != .mainnet)
        if !is_success(xpriv_result) {
            throw SignerError(details: String(cString: xpriv_result.details.error))
        }
        try self.writeKeychain(attrName: SecurityItemNames.masterXpriv.rawValue, data: String(cString: xpriv_result.details.data))
        result_destroy(xpriv_result)
    }

    internal func createIdentity(pubkeyChain: String) throws -> String {
        guard let master = try self.readKeychain(attrName: SecurityItemNames.masterXpriv.rawValue) else {
            throw SignerError(details: "Unable to generate identity master extended private key")
        }

        let xpriv_result = bip32_derive_xpriv(UnsafeMutablePointer<Int8>(mutating: (master as NSString).utf8String), true, pubkeyChain)
        if !is_success(xpriv_result) {
            throw SignerError(details: String(cString: xpriv_result.details.error))
        }
        var xpriv = String(cString: xpriv_result.details.data)
        try self.writeKeychain(attrName: pubkeyChain, data: xpriv)
        result_destroy(xpriv_result)

        withUnsafeMutableBytes(of: &xpriv) { pointer in
            pointer.copyBytes(from: [UInt8](repeating: 0, count: pointer.count))
        }

        let xpub_result = bip32_derive_xpub(UnsafeMutablePointer(mutating: (xpriv as NSString).utf8String), true, "m")
        let xpub = String(cString: xpub_result.details.data)
        result_destroy(xpub_result)
        return xpub
    }
}
