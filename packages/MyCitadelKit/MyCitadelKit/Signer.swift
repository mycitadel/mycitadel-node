//
// Created by Maxim Orlovsky on 2/19/21.
//

import Foundation
import Security

private enum SecurityItemNames: String {
    case seed = "Citadel.seed"
    case masterXpriv = "Citadel.master"
}

public struct SignerError: Error {
    let localizedDescription: String
}

protocol KeychainStorage {
    func checkKeychain(attrName: String) throws -> Bool
    func readKeychain(attrName: String) throws -> String?
    func writeKeychain(attrName: String, value: String) throws
}

protocol SignerAPI {
    func createSeed() throws
    func createScopedChain(derivation: String) throws -> String
}

extension CitadelVault: KeychainStorage {
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
            let errorDetails: String
            if let details = SecCopyErrorMessageString(status, nil) {
                errorDetails = "keychain lookup for item `\(attrName)` has failed: \(details)"
            } else {
                errorDetails = "keychain lookup for item `\(attrName)` has failed with OSStatus=\(status)"
            }
            print(errorDetails)
            throw SignerError(localizedDescription: errorDetails)
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

        print("Reading item \(attrName) from the default keychain")
        let status = SecItemCopyMatching(query as CFDictionary, &foundItem)
        switch status {
        case errSecSuccess:
            guard let data = foundItem as? Data,
                  let stringRepr = String(data: data, encoding: .utf8)
            else {
                let errorDetails = "wrong encoding of \(attrName) in the default keychain"
                print(errorDetails)
                throw SignerError(localizedDescription: errorDetails)
            }
            return stringRepr
        case errSecItemNotFound:
            print("Item \(attrName) is not found in the default keychain")
            return nil
        case let status:
            let errorDetails: String
            if let details = SecCopyErrorMessageString(status, nil) {
                errorDetails = "keychain read failure for `\(attrName)` item: \(details)"
            } else {
                errorDetails = "keychain read failure for `\(attrName)` item with OSStatus=\(status)"
            }
            print(errorDetails)
            throw SignerError(localizedDescription: errorDetails)
        }
    }

    public func writeKeychain(attrName: String, value: String) throws {
        let query = [
            kSecClass: kSecClassGenericPassword,
            kSecAttrAccount: attrName,
            kSecAttrAccessible: kSecAttrAccessibleWhenUnlocked,
            kSecUseDataProtectionKeychain: true,
            kSecValueData: value.data(using: String.Encoding.utf8)!
        ] as [String: Any]

        let status = SecItemAdd(query as CFDictionary, nil)
        if status != errSecSuccess {
            let errorDetails: String
            if let details = SecCopyErrorMessageString(status, nil) {
                errorDetails = "error writing item \(attrName) to the keychain: \(details)"
            } else {
                errorDetails = "error writing item \(attrName) to the keychain with OSStatus=\(status)"
            }
            print(errorDetails)
            throw SignerError(localizedDescription: errorDetails)
        }
    }
}

extension CitadelVault: SignerAPI {
    internal func createSeed() throws {
        print("Initializing seed")
        if try self.checkKeychain(attrName: SecurityItemNames.seed.rawValue) {
            print("Existing seed found")
            return // We already have a seed phrase
        }

        print("Creating entropy for mnemonic")
        let result = bip39_mnemonic_create(nil, BIP39_MNEMONIC_TYPE_WORDS_12);
        if !is_success(result) {
            let errorDetails = "Unable to generate seed: \(String(cString: result.details.error))"
            print(errorDetails)
            throw SignerError(localizedDescription: errorDetails)
        }
        var seedPhrase = String(cString: result.details.data)
        do {
            try self.writeKeychain(attrName: SecurityItemNames.seed.rawValue, value: seedPhrase)
        } catch {
            let errorDetails = "Unable to store seed information. Caused by: \(error.localizedDescription)"
            print(errorDetails)
            throw SignerError(localizedDescription: errorDetails)
        }
        defer {
            seedPhrase.safelyWipe()
        }

        print("Creating master extended private key")
        let xpriv_result = bip39_master_xpriv(UnsafeMutablePointer(mutating: result.details.data), nil, true, self.network != .mainnet)
        if !is_success(xpriv_result) {
            let errorDetails = "Unable to generate master extended private key from seed data. Error by seed generator: \(String(cString: xpriv_result.details.error))"
            print(errorDetails)
            throw SignerError(localizedDescription: errorDetails)
        }
        var xpriv = String(cString: xpriv_result.details.data)
        do {
            try self.writeKeychain(attrName: SecurityItemNames.masterXpriv.rawValue, value: xpriv)
        } catch {
            let errorDetails = "Unable to store master private key information. Caused by: \(error.localizedDescription)";
            print(errorDetails)
            throw SignerError(localizedDescription: errorDetails)
        }
        xpriv.safelyWipe()
        result_destroy(xpriv_result)
    }

    internal func createScopedChain(derivation: String) throws -> String {
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
            let errorDetails = String(cString: xpriv_result.details.error);
            result_destroy(xpriv_result)
            print("Derivation failed: \(errorDetails)")
            throw SignerError(localizedDescription: errorDetails)
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
        do {
            try self.writeKeychain(attrName: pubkeyChain, value: xpriv)
        } catch {
            let errorDetails = "Unable to store identity private key information. Caused by: \(error.localizedDescription)";
            print(errorDetails)
            throw SignerError(localizedDescription: errorDetails)
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
