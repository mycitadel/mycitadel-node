#ifndef libmycitadel_h
#define libmycitadel_h

/* Generated with cbindgen:0.16.0 */

/* Warning, this file is autogenerated by cbindgen. Don't modify this manually. */

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

#define BECH32_OK 0

#define BECH32_ERR_HRP 1

#define BECH32_ERR_CHECKSUM 2

#define BECH32_ERR_ENCODING 3

#define BECH32_ERR_PAYLOAD 4

#define BECH32_ERR_UNSUPPORTED 5

#define BECH32_ERR_INTERNAL 6

#define BECH32_ERR_NULL 7

#define BECH32_UNKNOWN 0

#define BECH32_URL 1

#define BECH32_BC_ADDRESS 256

#define BECH32_LN_BOLT11 257

#define BECH32_LNPBP_ID 512

#define BECH32_LNPBP_DATA 513

#define BECH32_LNPBP_ZDATA 514

#define BECH32_LNPBP_INVOICE 528

#define BECH32_RGB_SCHEMA_ID 768

#define BECH32_RGB_CONTRACT_ID 769

#define BECH32_RGB_SCHEMA 784

#define BECH32_RGB_GENESIS 785

#define BECH32_RGB_CONSIGNMENT 800

#define BECH32_RGB20_ASSET 800

#define SUCCESS 0

#define ERRNO_IO 1

#define ERRNO_RPC 2

#define ERRNO_NET 3

#define ERRNO_TRANSPORT 4

#define ERRNO_NOTSUPPORTED 5

#define ERRNO_STORAGE 6

#define ERRNO_SERVERFAIL 7

#define ERRNO_EMBEDDEDFAIL 8

#define ERRNO_UNINIT 100

#define ERRNO_CHAIN 101

#define ERRNO_JSON 102

#define ERRNO_BECH32 103

#define ERRNO_PARSE 104

#define ERRNO_NULL 105

typedef enum bip39_mnemonic_type {
        words_12,
        words_15,
        words_18,
        words_21,
        words_24,
} bip39_mnemonic_type;

enum error_t
#ifdef __cplusplus
  : uint16_t
#endif // __cplusplus
 {
        success = 0,
        /**
         * got a null pointer as one of the function arguments
         */
        null_pointer,
        /**
         * result data must be a valid string which does not contain zero bytes
         */
        invalid_result_data,
        /**
         * invalid mnemonic string
         */
        invalid_mnemonic,
        /**
         * invalid UTF-8 string
         */
        invalid_utf8_string,
        /**
         * wrong BIP32 extended public or private key data
         */
        wrong_extended_key,
        /**
         * unable to derive hardened path from a public key
         */
        unable_to_derive_hardened,
        /**
         * invalid derivation path
         */
        invalid_derivation_path,
        /**
         * general BIP32-specific failure
         */
        bip32_failure,
};
#ifndef __cplusplus
typedef uint16_t error_t;
#endif // __cplusplus

typedef struct bech32_info_t {
        int status;
        int category;
        bool bech32m;
        const char *details;
} bech32_info_t;

typedef struct mycitadel_client_t {
        void *opaque;
        const char *message;
        int err_no;
} mycitadel_client_t;

typedef union result_details_t {
        const char *data;
        const char *error;
} result_details_t;

typedef struct string_result_t {
        error_t code;
        union result_details_t details;
} string_result_t;

#ifdef __cplusplus
extern "C" {
#endif // __cplusplus

void lnpbp_bech32_release(struct bech32_info_t info);

struct bech32_info_t lnpbp_bech32_info(const char *bech_str);

void release_string(char *s);

struct mycitadel_client_t *mycitadel_run_embedded(const char *chain,
                                                  const char *data_dir,
                                                  const char *electrum_server);

bool mycitadel_is_ok(struct mycitadel_client_t *client);

bool mycitadel_has_err(struct mycitadel_client_t *client);

const char *mycitadel_contract_list(struct mycitadel_client_t *client);

const char *mycitadel_single_sig_create(struct mycitadel_client_t *client,
                                        const char *name,
                                        const char *keychain,
                                        OuterCategory category);

const char *mycitadel_contract_rename(struct mycitadel_client_t *client,
                                      const char *contract_id,
                                      const char *new_name);

const char *mycitadel_contract_delete(struct mycitadel_client_t *client,
                                      const char *contract_id);

const char *mycitadel_contract_balance(struct mycitadel_client_t *client,
                                       const char *contract_id,
                                       bool rescan,
                                       uint8_t lookup_depth);

const char *mycitadel_address_list(struct mycitadel_client_t *client,
                                   const char *contract_id,
                                   bool rescan,
                                   uint8_t lookup_depth);

const char *mycitadel_address_create(struct mycitadel_client_t *client,
                                     const char *contract_id,
                                     bool mark_used,
                                     bool legacy);

const char *mycitadel_invoice_create(struct mycitadel_client_t *client,
                                     InvoiceType category,
                                     const char *contract_id,
                                     const char *asset_id,
                                     uint64_t amount,
                                     const char *merchant,
                                     const char *purpose,
                                     bool unmark,
                                     bool legacy);

const char *mycitadel_invoice_list(struct mycitadel_client_t *client,
                                   const char *contract_id);

const char *mycitadel_invoice_pay(struct mycitadel_client_t *client,
                                  const char *contract_id,
                                  const char *invoice,
                                  uint64_t fee,
                                  uint64_t giveaway);

const char *mycitadel_invoice_accept(struct mycitadel_client_t *client,
                                     const char *contract_id);

const char *mycitadel_asset_list(struct mycitadel_client_t *client);

const char *mycitadel_asset_import(struct mycitadel_client_t *client,
                                   const char *genesis_b32);

bool is_success(struct string_result_t result);

void result_destroy(struct string_result_t result);

/**
 * Creates a rust-owned mnemonic string. You MUSt always call
 * [`string_destroy`] right after storing the mnemonic string and
 * do not call other methods from this library on that mnemonic. If you need
 * to call [`bip39_master_xpriv`] you MUST read mnemonic again and provide
 * unowned string to the rust.
 */
struct string_result_t bip39_mnemonic_create(const uint8_t *entropy,
                                             enum bip39_mnemonic_type mnemonic_type);

struct string_result_t bip39_master_xpriv(char *seed_phrase,
                                          char *passwd,
                                          bool wipe,
                                          bool testnet);

struct string_result_t bip32_derive_xpriv(char *master,
                                          bool wipe,
                                          const char *derivation);

struct string_result_t bip32_derive_xpub(char *master,
                                         bool wipe,
                                         const char *derivation);

struct string_result_t psbt_sign(const char *_psbt,
                                 const char *_xpriv,
                                 bool _wipe);

#ifdef __cplusplus
} // extern "C"
#endif // __cplusplus

#endif /* libmycitadel_h */
