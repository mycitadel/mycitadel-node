
using namespace System.Management.Automation
using namespace System.Management.Automation.Language

Register-ArgumentCompleter -Native -CommandName 'mycitadel' -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)

    $commandElements = $commandAst.CommandElements
    $command = @(
        'mycitadel'
        for ($i = 1; $i -lt $commandElements.Count; $i++) {
            $element = $commandElements[$i]
            if ($element -isnot [StringConstantExpressionAst] -or
                $element.StringConstantType -ne [StringConstantType]::BareWord -or
                $element.Value.StartsWith('-')) {
                break
        }
        $element.Value
    }) -join ';'

    $completions = @(switch ($command) {
        'mycitadel' {
            [CompletionResult]::new('-T', 'T', [CompletionResultType]::ParameterName, 'Use Tor')
            [CompletionResult]::new('--tor-proxy', 'tor-proxy', [CompletionResultType]::ParameterName, 'Use Tor')
            [CompletionResult]::new('-x', 'x', [CompletionResultType]::ParameterName, 'ZMQ socket name/address for MyCitadel node RPC interface')
            [CompletionResult]::new('--rpc-endpoint', 'rpc-endpoint', [CompletionResultType]::ParameterName, 'ZMQ socket name/address for MyCitadel node RPC interface')
            [CompletionResult]::new('-n', 'n', [CompletionResultType]::ParameterName, 'Blockchain to use')
            [CompletionResult]::new('--chain', 'chain', [CompletionResultType]::ParameterName, 'Blockchain to use')
            [CompletionResult]::new('-d', 'd', [CompletionResultType]::ParameterName, 'Data directory path')
            [CompletionResult]::new('--data-dir', 'data-dir', [CompletionResultType]::ParameterName, 'Data directory path')
            [CompletionResult]::new('--electrum-server', 'electrum-server', [CompletionResultType]::ParameterName, 'Electrum server connection string')
            [CompletionResult]::new('--rgb20-endpoint', 'rgb20-endpoint', [CompletionResultType]::ParameterName, 'RGB node connection string')
            [CompletionResult]::new('-c', 'c', [CompletionResultType]::ParameterName, 'Path to the configuration file')
            [CompletionResult]::new('--config', 'config', [CompletionResultType]::ParameterName, 'Path to the configuration file')
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Print help information')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help information')
            [CompletionResult]::new('-V', 'V', [CompletionResultType]::ParameterName, 'Print version information')
            [CompletionResult]::new('--version', 'version', [CompletionResultType]::ParameterName, 'Print version information')
            [CompletionResult]::new('-v', 'v', [CompletionResultType]::ParameterName, 'Set verbosity level')
            [CompletionResult]::new('--verbose', 'verbose', [CompletionResultType]::ParameterName, 'Set verbosity level')
            [CompletionResult]::new('--init', 'init', [CompletionResultType]::ParameterName, 'Initializes config file with the default values')
            [CompletionResult]::new('--rgb-embedded', 'rgb-embedded', [CompletionResultType]::ParameterName, 'rgb-embedded')
            [CompletionResult]::new('wallet', 'wallet', [CompletionResultType]::ParameterValue, 'Wallet management commands')
            [CompletionResult]::new('address', 'address', [CompletionResultType]::ParameterValue, 'Address-related commands')
            [CompletionResult]::new('asset', 'asset', [CompletionResultType]::ParameterValue, 'Asset management commands')
            [CompletionResult]::new('invoice', 'invoice', [CompletionResultType]::ParameterValue, 'Invoice-related commands')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'mycitadel;wallet' {
            [CompletionResult]::new('-n', 'n', [CompletionResultType]::ParameterName, 'Blockchain to use')
            [CompletionResult]::new('--chain', 'chain', [CompletionResultType]::ParameterName, 'Blockchain to use')
            [CompletionResult]::new('-d', 'd', [CompletionResultType]::ParameterName, 'Data directory path')
            [CompletionResult]::new('--data-dir', 'data-dir', [CompletionResultType]::ParameterName, 'Data directory path')
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Print help information')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help information')
            [CompletionResult]::new('-V', 'V', [CompletionResultType]::ParameterName, 'Print version information')
            [CompletionResult]::new('--version', 'version', [CompletionResultType]::ParameterName, 'Print version information')
            [CompletionResult]::new('-v', 'v', [CompletionResultType]::ParameterName, 'Set verbosity level')
            [CompletionResult]::new('--verbose', 'verbose', [CompletionResultType]::ParameterName, 'Set verbosity level')
            [CompletionResult]::new('list', 'list', [CompletionResultType]::ParameterValue, 'Lists existing wallets')
            [CompletionResult]::new('create', 'create', [CompletionResultType]::ParameterValue, 'Creates wallet with a given name and descriptor parameters')
            [CompletionResult]::new('rename', 'rename', [CompletionResultType]::ParameterValue, 'Change a name of a wallet')
            [CompletionResult]::new('delete', 'delete', [CompletionResultType]::ParameterValue, 'Delete existing wallet contract')
            [CompletionResult]::new('balance', 'balance', [CompletionResultType]::ParameterValue, 'Returns detailed wallet balance information')
            [CompletionResult]::new('sign', 'sign', [CompletionResultType]::ParameterValue, 'Signs given PSBT with keys controlled by a wallet master extended keys')
            [CompletionResult]::new('publish', 'publish', [CompletionResultType]::ParameterValue, 'Finalizes fully-signed PSBT and publishes transaction to bitcoin network, updating PSBT data stored in wallet `wallet_id`')
            break
        }
        'mycitadel;wallet;list' {
            [CompletionResult]::new('-f', 'f', [CompletionResultType]::ParameterName, 'How the wallet list should be formatted')
            [CompletionResult]::new('--format', 'format', [CompletionResultType]::ParameterName, 'How the wallet list should be formatted')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help information')
            [CompletionResult]::new('--version', 'version', [CompletionResultType]::ParameterName, 'Print version information')
            break
        }
        'mycitadel;wallet;create' {
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help information')
            [CompletionResult]::new('--version', 'version', [CompletionResultType]::ParameterName, 'Print version information')
            [CompletionResult]::new('single-sig', 'single-sig', [CompletionResultType]::ParameterValue, 'Creates current single-sig wallet account')
            break
        }
        'mycitadel;wallet;create;single-sig' {
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help information')
            [CompletionResult]::new('--version', 'version', [CompletionResultType]::ParameterName, 'Print version information')
            [CompletionResult]::new('--bare', 'bare', [CompletionResultType]::ParameterName, 'Creates old "bare" wallets, where public key is kept in the explicit form within bitcoin transaction P2PK output')
            [CompletionResult]::new('--legacy', 'legacy', [CompletionResultType]::ParameterName, 'Whether create a pre-SegWit wallet (P2PKH) rather than SegWit (P2WPKH). If you''d like to use legacy SegWit-style addresses (P2WPKH-in-P2SH), do not use this flag, create normal SegWit wallet instead and specify `--legacy` option when requesting new address')
            [CompletionResult]::new('--segwit', 'segwit', [CompletionResultType]::ParameterName, 'Recommended SegWit wallet with P2WKH and P2WPKH-in-P2SH outputs')
            [CompletionResult]::new('--taproot', 'taproot', [CompletionResultType]::ParameterName, 'Reserved for the future taproot P2TR outputs')
            break
        }
        'mycitadel;wallet;rename' {
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help information')
            [CompletionResult]::new('--version', 'version', [CompletionResultType]::ParameterName, 'Print version information')
            break
        }
        'mycitadel;wallet;delete' {
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help information')
            [CompletionResult]::new('--version', 'version', [CompletionResultType]::ParameterName, 'Print version information')
            break
        }
        'mycitadel;wallet;balance' {
            [CompletionResult]::new('--lookup-depth', 'lookup-depth', [CompletionResultType]::ParameterName, 'How many addresses should be scanned at least after the final address with no transactions is reached. Defaults to 20')
            [CompletionResult]::new('-f', 'f', [CompletionResultType]::ParameterName, 'How the command output should be formatted')
            [CompletionResult]::new('--format', 'format', [CompletionResultType]::ParameterName, 'How the command output should be formatted')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help information')
            [CompletionResult]::new('--version', 'version', [CompletionResultType]::ParameterName, 'Print version information')
            [CompletionResult]::new('-r', 'r', [CompletionResultType]::ParameterName, 'Whether to re-scan addresses space with Electrum server')
            [CompletionResult]::new('--rescan', 'rescan', [CompletionResultType]::ParameterName, 'Whether to re-scan addresses space with Electrum server')
            break
        }
        'mycitadel;wallet;sign' {
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help information')
            [CompletionResult]::new('--version', 'version', [CompletionResultType]::ParameterName, 'Print version information')
            break
        }
        'mycitadel;wallet;publish' {
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help information')
            [CompletionResult]::new('--version', 'version', [CompletionResultType]::ParameterName, 'Print version information')
            break
        }
        'mycitadel;address' {
            [CompletionResult]::new('-n', 'n', [CompletionResultType]::ParameterName, 'Blockchain to use')
            [CompletionResult]::new('--chain', 'chain', [CompletionResultType]::ParameterName, 'Blockchain to use')
            [CompletionResult]::new('-d', 'd', [CompletionResultType]::ParameterName, 'Data directory path')
            [CompletionResult]::new('--data-dir', 'data-dir', [CompletionResultType]::ParameterName, 'Data directory path')
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Print help information')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help information')
            [CompletionResult]::new('-V', 'V', [CompletionResultType]::ParameterName, 'Print version information')
            [CompletionResult]::new('--version', 'version', [CompletionResultType]::ParameterName, 'Print version information')
            [CompletionResult]::new('-v', 'v', [CompletionResultType]::ParameterName, 'Set verbosity level')
            [CompletionResult]::new('--verbose', 'verbose', [CompletionResultType]::ParameterName, 'Set verbosity level')
            [CompletionResult]::new('list-used', 'list-used', [CompletionResultType]::ParameterValue, 'Print address list')
            [CompletionResult]::new('create', 'create', [CompletionResultType]::ParameterValue, 'create')
            [CompletionResult]::new('mark-used', 'mark-used', [CompletionResultType]::ParameterValue, 'mark-used')
            [CompletionResult]::new('pay', 'pay', [CompletionResultType]::ParameterValue, 'pay')
            break
        }
        'mycitadel;address;list-used' {
            [CompletionResult]::new('--lookup-depth', 'lookup-depth', [CompletionResultType]::ParameterName, 'How many addresses should be scanned at least after the final address with no transactions is reached. Defaults to 20')
            [CompletionResult]::new('-f', 'f', [CompletionResultType]::ParameterName, 'How the command output should be formatted')
            [CompletionResult]::new('--format', 'format', [CompletionResultType]::ParameterName, 'How the command output should be formatted')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help information')
            [CompletionResult]::new('--version', 'version', [CompletionResultType]::ParameterName, 'Print version information')
            [CompletionResult]::new('-r', 'r', [CompletionResultType]::ParameterName, 'Whether to re-scan addresses space with Electrum server')
            [CompletionResult]::new('--rescan', 'rescan', [CompletionResultType]::ParameterName, 'Whether to re-scan addresses space with Electrum server')
            break
        }
        'mycitadel;address;create' {
            [CompletionResult]::new('-i', 'i', [CompletionResultType]::ParameterName, 'Create address at custom index number')
            [CompletionResult]::new('--index', 'index', [CompletionResultType]::ParameterName, 'Create address at custom index number')
            [CompletionResult]::new('-f', 'f', [CompletionResultType]::ParameterName, 'How the asset list output should be formatted')
            [CompletionResult]::new('--format', 'format', [CompletionResultType]::ParameterName, 'How the asset list output should be formatted')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help information')
            [CompletionResult]::new('--version', 'version', [CompletionResultType]::ParameterName, 'Print version information')
            [CompletionResult]::new('-u', 'u', [CompletionResultType]::ParameterName, 'Whether to mark address as used')
            [CompletionResult]::new('--unmark', 'unmark', [CompletionResultType]::ParameterName, 'Whether to mark address as used')
            [CompletionResult]::new('--legacy', 'legacy', [CompletionResultType]::ParameterName, 'Use SegWit legacy address format (applicable only to a SegWit wallets)')
            break
        }
        'mycitadel;address;mark-used' {
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help information')
            [CompletionResult]::new('--version', 'version', [CompletionResultType]::ParameterName, 'Print version information')
            [CompletionResult]::new('--legacy', 'legacy', [CompletionResultType]::ParameterName, 'Use SegWit legacy address format (applicable only to a SegWit wallets)')
            [CompletionResult]::new('-u', 'u', [CompletionResultType]::ParameterName, 'Remove use mark (inverses the command)')
            [CompletionResult]::new('--unmark', 'unmark', [CompletionResultType]::ParameterName, 'Remove use mark (inverses the command)')
            break
        }
        'mycitadel;address;pay' {
            [CompletionResult]::new('-o', 'o', [CompletionResultType]::ParameterName, 'File name to output PSBT. If no name is given PSBT data are output to STDOUT')
            [CompletionResult]::new('--output', 'output', [CompletionResultType]::ParameterName, 'File name to output PSBT. If no name is given PSBT data are output to STDOUT')
            [CompletionResult]::new('-f', 'f', [CompletionResultType]::ParameterName, 'PSBT format to use for the output; if no file is specified defaults to Base64 output; otherwise defaults to binary')
            [CompletionResult]::new('--format', 'format', [CompletionResultType]::ParameterName, 'PSBT format to use for the output; if no file is specified defaults to Base64 output; otherwise defaults to binary')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help information')
            [CompletionResult]::new('--version', 'version', [CompletionResultType]::ParameterName, 'Print version information')
            break
        }
        'mycitadel;asset' {
            [CompletionResult]::new('-n', 'n', [CompletionResultType]::ParameterName, 'Blockchain to use')
            [CompletionResult]::new('--chain', 'chain', [CompletionResultType]::ParameterName, 'Blockchain to use')
            [CompletionResult]::new('-d', 'd', [CompletionResultType]::ParameterName, 'Data directory path')
            [CompletionResult]::new('--data-dir', 'data-dir', [CompletionResultType]::ParameterName, 'Data directory path')
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Print help information')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help information')
            [CompletionResult]::new('-V', 'V', [CompletionResultType]::ParameterName, 'Print version information')
            [CompletionResult]::new('--version', 'version', [CompletionResultType]::ParameterName, 'Print version information')
            [CompletionResult]::new('-v', 'v', [CompletionResultType]::ParameterName, 'Set verbosity level')
            [CompletionResult]::new('--verbose', 'verbose', [CompletionResultType]::ParameterName, 'Set verbosity level')
            [CompletionResult]::new('list', 'list', [CompletionResultType]::ParameterValue, 'Lists known assets')
            [CompletionResult]::new('import', 'import', [CompletionResultType]::ParameterValue, 'Import asset genesis data')
            break
        }
        'mycitadel;asset;list' {
            [CompletionResult]::new('-f', 'f', [CompletionResultType]::ParameterName, 'How the asset list output should be formatted')
            [CompletionResult]::new('--format', 'format', [CompletionResultType]::ParameterName, 'How the asset list output should be formatted')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help information')
            [CompletionResult]::new('--version', 'version', [CompletionResultType]::ParameterName, 'Print version information')
            break
        }
        'mycitadel;asset;import' {
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help information')
            [CompletionResult]::new('--version', 'version', [CompletionResultType]::ParameterName, 'Print version information')
            break
        }
        'mycitadel;invoice' {
            [CompletionResult]::new('-n', 'n', [CompletionResultType]::ParameterName, 'Blockchain to use')
            [CompletionResult]::new('--chain', 'chain', [CompletionResultType]::ParameterName, 'Blockchain to use')
            [CompletionResult]::new('-d', 'd', [CompletionResultType]::ParameterName, 'Data directory path')
            [CompletionResult]::new('--data-dir', 'data-dir', [CompletionResultType]::ParameterName, 'Data directory path')
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Print help information')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help information')
            [CompletionResult]::new('-V', 'V', [CompletionResultType]::ParameterName, 'Print version information')
            [CompletionResult]::new('--version', 'version', [CompletionResultType]::ParameterName, 'Print version information')
            [CompletionResult]::new('-v', 'v', [CompletionResultType]::ParameterName, 'Set verbosity level')
            [CompletionResult]::new('--verbose', 'verbose', [CompletionResultType]::ParameterName, 'Set verbosity level')
            [CompletionResult]::new('create', 'create', [CompletionResultType]::ParameterValue, 'Create new invoice')
            [CompletionResult]::new('list', 'list', [CompletionResultType]::ParameterValue, 'List all issued invoices')
            [CompletionResult]::new('info', 'info', [CompletionResultType]::ParameterValue, 'Parse invoice and print out its detailed information')
            [CompletionResult]::new('pay', 'pay', [CompletionResultType]::ParameterValue, 'Pay an invoice')
            [CompletionResult]::new('accept', 'accept', [CompletionResultType]::ParameterValue, 'Accept payment for the invoice. Required only for on-chain RGB payments; Bitcoin & Lightning-network payments (including RGB lightning) are accepted automatically and does not require calling this method')
            break
        }
        'mycitadel;invoice;create' {
            [CompletionResult]::new('-a', 'a', [CompletionResultType]::ParameterName, 'Asset in which the payment is requested; defaults to bitcoin on the currently used blockchain (mainnet, liqud, testnet etc)')
            [CompletionResult]::new('--asset', 'asset', [CompletionResultType]::ParameterName, 'Asset in which the payment is requested; defaults to bitcoin on the currently used blockchain (mainnet, liqud, testnet etc)')
            [CompletionResult]::new('-m', 'm', [CompletionResultType]::ParameterName, 'Optional details about the merchant providing the invoice')
            [CompletionResult]::new('--merchant', 'merchant', [CompletionResultType]::ParameterName, 'Optional details about the merchant providing the invoice')
            [CompletionResult]::new('-p', 'p', [CompletionResultType]::ParameterName, 'Information about the invoice')
            [CompletionResult]::new('--purpose', 'purpose', [CompletionResultType]::ParameterName, 'Information about the invoice')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help information')
            [CompletionResult]::new('--version', 'version', [CompletionResultType]::ParameterName, 'Print version information')
            [CompletionResult]::new('-u', 'u', [CompletionResultType]::ParameterName, 'Whether to mark address as used')
            [CompletionResult]::new('--unmark', 'unmark', [CompletionResultType]::ParameterName, 'Whether to mark address as used')
            [CompletionResult]::new('--legacy', 'legacy', [CompletionResultType]::ParameterName, 'Use SegWit legacy address format (applicable only to a SegWit wallets)')
            [CompletionResult]::new('--descriptor', 'descriptor', [CompletionResultType]::ParameterName, 'Create descriptor-based invoice (not compatible with instant wallet accounts)')
            [CompletionResult]::new('--psbt', 'psbt', [CompletionResultType]::ParameterName, 'Create a PSBT-based invoice (not compatible with instant wallet accounts)')
            break
        }
        'mycitadel;invoice;list' {
            [CompletionResult]::new('-f', 'f', [CompletionResultType]::ParameterName, 'How invoice list should be formatted')
            [CompletionResult]::new('--format', 'format', [CompletionResultType]::ParameterName, 'How invoice list should be formatted')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help information')
            [CompletionResult]::new('--version', 'version', [CompletionResultType]::ParameterName, 'Print version information')
            break
        }
        'mycitadel;invoice;info' {
            [CompletionResult]::new('-f', 'f', [CompletionResultType]::ParameterName, 'Format to use for the invoice representation')
            [CompletionResult]::new('--format', 'format', [CompletionResultType]::ParameterName, 'Format to use for the invoice representation')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help information')
            [CompletionResult]::new('--version', 'version', [CompletionResultType]::ParameterName, 'Print version information')
            break
        }
        'mycitadel;invoice;pay' {
            [CompletionResult]::new('-a', 'a', [CompletionResultType]::ParameterName, 'Force payment with the specified amount (always in satoshis). Required for invoices that does not provide amount field. For other types of invoices, if provided, overrides the amount found in the invoice')
            [CompletionResult]::new('--amount', 'amount', [CompletionResultType]::ParameterName, 'Force payment with the specified amount (always in satoshis). Required for invoices that does not provide amount field. For other types of invoices, if provided, overrides the amount found in the invoice')
            [CompletionResult]::new('-o', 'o', [CompletionResultType]::ParameterName, 'File name to output PSBT. If no name is given PSBT data are output to STDOUT')
            [CompletionResult]::new('--output', 'output', [CompletionResultType]::ParameterName, 'File name to output PSBT. If no name is given PSBT data are output to STDOUT')
            [CompletionResult]::new('-c', 'c', [CompletionResultType]::ParameterName, 'File name to output consignment. If no name is given, consignment data are output to STDOUT in Bech32 format')
            [CompletionResult]::new('--consignment', 'consignment', [CompletionResultType]::ParameterName, 'File name to output consignment. If no name is given, consignment data are output to STDOUT in Bech32 format')
            [CompletionResult]::new('-f', 'f', [CompletionResultType]::ParameterName, 'PSBT format to use for the output; if no file is specified defaults to Base64 output; otherwise defaults to binary')
            [CompletionResult]::new('--format', 'format', [CompletionResultType]::ParameterName, 'PSBT format to use for the output; if no file is specified defaults to Base64 output; otherwise defaults to binary')
            [CompletionResult]::new('-g', 'g', [CompletionResultType]::ParameterName, 'How much satoshis to give away with RGB payment; required and allowed only when paying descriptor-based RGB invoices')
            [CompletionResult]::new('--giveaway', 'giveaway', [CompletionResultType]::ParameterName, 'How much satoshis to give away with RGB payment; required and allowed only when paying descriptor-based RGB invoices')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help information')
            [CompletionResult]::new('--version', 'version', [CompletionResultType]::ParameterName, 'Print version information')
            break
        }
        'mycitadel;invoice;accept' {
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help information')
            [CompletionResult]::new('--version', 'version', [CompletionResultType]::ParameterName, 'Print version information')
            [CompletionResult]::new('-f', 'f', [CompletionResultType]::ParameterName, 'Whether parameter given by consignment is a file name or a Bech32 string')
            [CompletionResult]::new('--file', 'file', [CompletionResultType]::ParameterName, 'Whether parameter given by consignment is a file name or a Bech32 string')
            break
        }
        'mycitadel;help' {
            [CompletionResult]::new('-n', 'n', [CompletionResultType]::ParameterName, 'Blockchain to use')
            [CompletionResult]::new('--chain', 'chain', [CompletionResultType]::ParameterName, 'Blockchain to use')
            [CompletionResult]::new('-d', 'd', [CompletionResultType]::ParameterName, 'Data directory path')
            [CompletionResult]::new('--data-dir', 'data-dir', [CompletionResultType]::ParameterName, 'Data directory path')
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Print help information')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help information')
            [CompletionResult]::new('-V', 'V', [CompletionResultType]::ParameterName, 'Print version information')
            [CompletionResult]::new('--version', 'version', [CompletionResultType]::ParameterName, 'Print version information')
            [CompletionResult]::new('-v', 'v', [CompletionResultType]::ParameterName, 'Set verbosity level')
            [CompletionResult]::new('--verbose', 'verbose', [CompletionResultType]::ParameterName, 'Set verbosity level')
            break
        }
    })

    $completions.Where{ $_.CompletionText -like "$wordToComplete*" } |
        Sort-Object -Property ListItemText
}
