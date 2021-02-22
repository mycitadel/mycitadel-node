//
// Created by Maxim Orlovsky on 2/19/21.
//

import Foundation
import Security

private enum SecurityItemNames: String {
    case seed = "MyCitadel.seed"
}

public struct SignerError: Error {
    let details: String
}

extension MyCitadelClient {
    internal func createSeed() throws {
        var query = [kSecClass: kSecClassGenericPassword,
                     kSecAttrAccount: SecurityItemNames.seed,
                     kSecAttrAccessible: kSecAttrAccessibleWhenUnlocked,
                     kSecUseDataProtectionKeychain: true] as [String: Any]
        var status = SecItemCopyMatching(query as CFDictionary, nil)
        guard status == errSecSuccess else {
            return // We already have a seed generated
        }

        let result = bip39_mnemonic_create(nil, BIP39_MNEMONIC_TYPE_WORDS_12);
        if is_success(result) {
            throw SignerError(details: "Unable to generate seed: \(String(cString: result.details.error))")
        }
        let seedData = String(cString: result.details.data);

        query[kSecValueData as String] = seedData
        status = SecItemAdd(query as CFDictionary, nil)
        guard status == errSecSuccess else {
            throw SignerError(details: "Unable to store seed information: \(status.description)")
        }
    }
}