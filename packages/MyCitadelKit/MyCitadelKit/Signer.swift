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
    let localizedDescription: String
}

extension MyCitadelClient {
    public func checkKeychain(attrName: String) throws -> Bool {
        let query = [
            kSecClass: kSecClassGenericPassword,
            kSecAttrAccount: attrName,
            kSecUseDataProtectionKeychain: true,
        ] as [String: Any]
        var foundItem: CFTypeRef?;

        switch SecItemCopyMatching(query as CFDictionary, &foundItem) {
        case errSecSuccess:
            return true
        case errSecItemNotFound:
            return false
        case let status:
            throw SignerError(localizedDescription: "Keychain check failed: \(status.description)")
        }
    }

    public func readKeychain(attrName: String) throws -> String? {
        let query = [
            kSecClass: kSecClassGenericPassword,
            kSecAttrAccount: attrName,
            kSecUseDataProtectionKeychain: true,
            kSecReturnData: true
        ] as [String: Any]
        var foundItem: CFTypeRef?;

        print("Reading item from the keychain: \(attrName)")
        let status = SecItemCopyMatching(query as CFDictionary, &foundItem)
        print("Status code: \(status)")
        switch status {
        case errSecSuccess:
            guard let data = foundItem as? Data,
                  let stringRepr = String(data: data, encoding: .utf8)
            else {
                print("Wrong encoding of \(attrName) in the Apple Keychain")
                throw SignerError(localizedDescription: "Wrong encoding of \(attrName) in the Apple Keychain")
            }
            return stringRepr
        case errSecItemNotFound:
            print("Item not found")
            return nil
        case let status:
            print("Keychain read failed: \(status.description)")
            throw SignerError(localizedDescription: "Keychain read failed: \(status.description)")
        }
    }

    public func writeKeychain(attrName: String, value: String) -> OSStatus {
        let query = [
            kSecClass: kSecClassGenericPassword,
            kSecAttrAccount: attrName,
            kSecAttrAccessible: kSecAttrAccessibleWhenUnlocked,
            kSecUseDataProtectionKeychain: true,
            kSecValueData: value.data(using: String.Encoding.utf8)!
        ] as [String: Any]

        return SecItemAdd(query as CFDictionary, nil)
    }

    internal func createSeed() throws {
        print("Initializing seed")
        if try self.checkKeychain(attrName: SecurityItemNames.seed.rawValue) {
            print("Existing seed found")
            return // We already have a seed phrase
        }

        print("Creating entropy for mnemonic")
        let result = bip39_mnemonic_create(nil, BIP39_MNEMONIC_TYPE_WORDS_12);
        if !is_success(result) {
            throw SignerError(localizedDescription: "Unable to generate seed: \(String(cString: result.details.error))")
        }
        var seedPhrase = String(cString: result.details.data)
        var status = self.writeKeychain(attrName: SecurityItemNames.seed.rawValue, value: seedPhrase)
        if status != errSecSuccess {
            throw SignerError(localizedDescription: "Unable to store seed information: \(status.description)")
        }
        defer {
            seedPhrase.safelyWipe()
        }

        print("Creating master extended private key")
        let xpriv_result = bip39_master_xpriv(UnsafeMutablePointer(mutating: result.details.data), nil, true, self.network != .mainnet)
        if !is_success(xpriv_result) {
            throw SignerError(localizedDescription: String(cString: xpriv_result.details.error))
        }
        var xpriv = String(cString: xpriv_result.details.data)
        status = self.writeKeychain(attrName: SecurityItemNames.masterXpriv.rawValue, value: xpriv)
        if status != errSecSuccess {
            throw SignerError(localizedDescription: "Unable to store master private key information: \(status.description)")
        }
        xpriv.safelyWipe()
        result_destroy(xpriv_result)
    }

    internal func createIdentity(derivation: String) throws -> String {
        print("Creating identity private key")

        guard var master = try self.readKeychain(attrName: SecurityItemNames.masterXpriv.rawValue) else {
            print("Unable to find master extended private key")
            throw SignerError(localizedDescription: "Unable to find master extended private key")
        }
        defer {
            master.safelyWipe()
        }

        print("Seed found, deriving scoped private key from it for \(derivation)")
        let xpriv_result = bip32_scoped_xpriv(master.cPtr(), false, derivation)
        if !is_success(xpriv_result) {
            let failure = String(cString: xpriv_result.details.error);
            result_destroy(xpriv_result)
            print("Derivation failed: \(failure)")
            throw SignerError(localizedDescription: failure)
        }
        var xpriv = String(cString: xpriv_result.details.data)
        defer {
            xpriv.safelyWipe()
            result_destroy(xpriv_result)
        }

        print("Creating identity pubkeychain")
        let pubkeychain_result = bip32_pubkey_chain_create(master.cPtr(), true, derivation)
        defer {
            result_destroy(pubkeychain_result)
        }
        if !is_success(pubkeychain_result) {
            let failure = String(cString: pubkeychain_result.details.error);
            print("Pubkeychain creation failed: \(failure)")
            throw SignerError(localizedDescription: failure)
        }
        let pubkeyChain = String(cString: pubkeychain_result.details.data)
        print("Created pubkeychain: \(pubkeyChain)")

        print("Storing scoped private key")
        let status = self.writeKeychain(attrName: pubkeyChain, value: xpriv)
        if status != errSecSuccess {
            print("Unable to store identity private key information")
            throw SignerError(localizedDescription: "Unable to store identity private key information: \(status.description)")
        }

        return pubkeyChain
    }
}

extension String {
    mutating func safelyWipe() {
        withUnsafeMutableBytes(of: &self) { pointer in
            pointer.copyBytes(from: [UInt8](repeating: 0, count: pointer.count))
        }
    }

    mutating func cPtr() -> UnsafeMutablePointer<Int8>? {
        UnsafeMutablePointer(mutating: (self as NSString).utf8String)
    }
}
