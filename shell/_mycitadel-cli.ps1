
using namespace System.Management.Automation
using namespace System.Management.Automation.Language

Register-ArgumentCompleter -Native -CommandName 'mycitadel-cli' -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)

    $commandElements = $commandAst.CommandElements
    $command = @(
        'mycitadel-cli'
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
        'mycitadel-cli' {
            [CompletionResult]::new('-T', 'T', [CompletionResultType]::ParameterName, 'Use Tor')
            [CompletionResult]::new('--tor-proxy', 'tor-proxy', [CompletionResultType]::ParameterName, 'Use Tor')
            [CompletionResult]::new('-x', 'x', [CompletionResultType]::ParameterName, 'ZMQ socket name/address for MyCitadel node RPC interface')
            [CompletionResult]::new('--rpc-endpoint', 'rpc-endpoint', [CompletionResultType]::ParameterName, 'ZMQ socket name/address for MyCitadel node RPC interface')
            [CompletionResult]::new('-c', 'c', [CompletionResultType]::ParameterName, 'Path to the configuration file')
            [CompletionResult]::new('--config', 'config', [CompletionResultType]::ParameterName, 'Path to the configuration file')
            [CompletionResult]::new('-v', 'v', [CompletionResultType]::ParameterName, 'Set verbosity level')
            [CompletionResult]::new('--verbose', 'verbose', [CompletionResultType]::ParameterName, 'Set verbosity level')
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Prints help information')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Prints help information')
            [CompletionResult]::new('-V', 'V', [CompletionResultType]::ParameterName, 'Prints version information')
            [CompletionResult]::new('--version', 'version', [CompletionResultType]::ParameterName, 'Prints version information')
            [CompletionResult]::new('wallet', 'wallet', [CompletionResultType]::ParameterValue, 'Wallet management commands')
            [CompletionResult]::new('address', 'address', [CompletionResultType]::ParameterValue, 'Address-related commands')
            [CompletionResult]::new('asset', 'asset', [CompletionResultType]::ParameterValue, 'Asset management commands')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Prints this message or the help of the given subcommand(s)')
            break
        }
        'mycitadel-cli;wallet' {
            [CompletionResult]::new('-v', 'v', [CompletionResultType]::ParameterName, 'Set verbosity level')
            [CompletionResult]::new('--verbose', 'verbose', [CompletionResultType]::ParameterName, 'Set verbosity level')
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Prints help information')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Prints help information')
            [CompletionResult]::new('-V', 'V', [CompletionResultType]::ParameterName, 'Prints version information')
            [CompletionResult]::new('--version', 'version', [CompletionResultType]::ParameterName, 'Prints version information')
            [CompletionResult]::new('list', 'list', [CompletionResultType]::ParameterValue, 'Lists existing wallets')
            [CompletionResult]::new('create', 'create', [CompletionResultType]::ParameterValue, 'Creates wallet with a given name and descriptor parameters')
            [CompletionResult]::new('rename', 'rename', [CompletionResultType]::ParameterValue, 'Change a name of a wallet')
            [CompletionResult]::new('delete', 'delete', [CompletionResultType]::ParameterValue, 'Delete existing wallet contract')
            [CompletionResult]::new('balance', 'balance', [CompletionResultType]::ParameterValue, 'Returns detailed wallet balance information')
            break
        }
        'mycitadel-cli;wallet;list' {
            [CompletionResult]::new('-f', 'f', [CompletionResultType]::ParameterName, 'How the wallet list should be formatted')
            [CompletionResult]::new('--format', 'format', [CompletionResultType]::ParameterName, 'How the wallet list should be formatted')
            [CompletionResult]::new('-v', 'v', [CompletionResultType]::ParameterName, 'Set verbosity level')
            [CompletionResult]::new('--verbose', 'verbose', [CompletionResultType]::ParameterName, 'Set verbosity level')
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Prints help information')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Prints help information')
            [CompletionResult]::new('-V', 'V', [CompletionResultType]::ParameterName, 'Prints version information')
            [CompletionResult]::new('--version', 'version', [CompletionResultType]::ParameterName, 'Prints version information')
            break
        }
        'mycitadel-cli;wallet;create' {
            [CompletionResult]::new('--bare', 'bare', [CompletionResultType]::ParameterName, 'Creates old "bare" wallets, where public key is kept in the explicit form within bitcoin transaction P2PK output')
            [CompletionResult]::new('--legacy', 'legacy', [CompletionResultType]::ParameterName, 'Whether create a pre-SegWit wallet (P2PKH) rather than SegWit (P2WPKH). If you''d like to use legacy SegWit-style addresses (P2WPKH-in-P2SH), do not use this flag, create normal SegWit wallet instead and specify `--legacy` option when requesting new address')
            [CompletionResult]::new('--segwit', 'segwit', [CompletionResultType]::ParameterName, 'Recommended SegWit wallet with P2WKH and P2WPKH-in-P2SH outputs')
            [CompletionResult]::new('--taproot', 'taproot', [CompletionResultType]::ParameterName, 'Reserved for the future taproot P2TR outputs')
            [CompletionResult]::new('-v', 'v', [CompletionResultType]::ParameterName, 'Set verbosity level')
            [CompletionResult]::new('--verbose', 'verbose', [CompletionResultType]::ParameterName, 'Set verbosity level')
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Prints help information')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Prints help information')
            [CompletionResult]::new('-V', 'V', [CompletionResultType]::ParameterName, 'Prints version information')
            [CompletionResult]::new('--version', 'version', [CompletionResultType]::ParameterName, 'Prints version information')
            [CompletionResult]::new('single-sig', 'single-sig', [CompletionResultType]::ParameterValue, 'Creates current single-sig wallet account')
            break
        }
        'mycitadel-cli;wallet;create;single-sig' {
            [CompletionResult]::new('--bare', 'bare', [CompletionResultType]::ParameterName, 'Creates old "bare" wallets, where public key is kept in the explicit form within bitcoin transaction P2PK output')
            [CompletionResult]::new('--legacy', 'legacy', [CompletionResultType]::ParameterName, 'Whether create a pre-SegWit wallet (P2PKH) rather than SegWit (P2WPKH). If you''d like to use legacy SegWit-style addresses (P2WPKH-in-P2SH), do not use this flag, create normal SegWit wallet instead and specify `--legacy` option when requesting new address')
            [CompletionResult]::new('--segwit', 'segwit', [CompletionResultType]::ParameterName, 'Recommended SegWit wallet with P2WKH and P2WPKH-in-P2SH outputs')
            [CompletionResult]::new('--taproot', 'taproot', [CompletionResultType]::ParameterName, 'Reserved for the future taproot P2TR outputs')
            [CompletionResult]::new('-v', 'v', [CompletionResultType]::ParameterName, 'Set verbosity level')
            [CompletionResult]::new('--verbose', 'verbose', [CompletionResultType]::ParameterName, 'Set verbosity level')
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Prints help information')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Prints help information')
            [CompletionResult]::new('-V', 'V', [CompletionResultType]::ParameterName, 'Prints version information')
            [CompletionResult]::new('--version', 'version', [CompletionResultType]::ParameterName, 'Prints version information')
            break
        }
        'mycitadel-cli;wallet;rename' {
            [CompletionResult]::new('-v', 'v', [CompletionResultType]::ParameterName, 'Set verbosity level')
            [CompletionResult]::new('--verbose', 'verbose', [CompletionResultType]::ParameterName, 'Set verbosity level')
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Prints help information')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Prints help information')
            [CompletionResult]::new('-V', 'V', [CompletionResultType]::ParameterName, 'Prints version information')
            [CompletionResult]::new('--version', 'version', [CompletionResultType]::ParameterName, 'Prints version information')
            break
        }
        'mycitadel-cli;wallet;delete' {
            [CompletionResult]::new('-v', 'v', [CompletionResultType]::ParameterName, 'Set verbosity level')
            [CompletionResult]::new('--verbose', 'verbose', [CompletionResultType]::ParameterName, 'Set verbosity level')
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Prints help information')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Prints help information')
            [CompletionResult]::new('-V', 'V', [CompletionResultType]::ParameterName, 'Prints version information')
            [CompletionResult]::new('--version', 'version', [CompletionResultType]::ParameterName, 'Prints version information')
            break
        }
        'mycitadel-cli;wallet;balance' {
            [CompletionResult]::new('-r', 'r', [CompletionResultType]::ParameterName, 'Whether to re-scan addresses space with Electrum server')
            [CompletionResult]::new('--rescan', 'rescan', [CompletionResultType]::ParameterName, 'Whether to re-scan addresses space with Electrum server')
            [CompletionResult]::new('--lookup-depth', 'lookup-depth', [CompletionResultType]::ParameterName, 'How many addresses should be scanned at least after the final address with no transactions is reached')
            [CompletionResult]::new('-f', 'f', [CompletionResultType]::ParameterName, 'How the command output should be formatted')
            [CompletionResult]::new('--format', 'format', [CompletionResultType]::ParameterName, 'How the command output should be formatted')
            [CompletionResult]::new('-v', 'v', [CompletionResultType]::ParameterName, 'Set verbosity level')
            [CompletionResult]::new('--verbose', 'verbose', [CompletionResultType]::ParameterName, 'Set verbosity level')
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Prints help information')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Prints help information')
            [CompletionResult]::new('-V', 'V', [CompletionResultType]::ParameterName, 'Prints version information')
            [CompletionResult]::new('--version', 'version', [CompletionResultType]::ParameterName, 'Prints version information')
            break
        }
        'mycitadel-cli;address' {
            [CompletionResult]::new('-v', 'v', [CompletionResultType]::ParameterName, 'Set verbosity level')
            [CompletionResult]::new('--verbose', 'verbose', [CompletionResultType]::ParameterName, 'Set verbosity level')
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Prints help information')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Prints help information')
            [CompletionResult]::new('-V', 'V', [CompletionResultType]::ParameterName, 'Prints version information')
            [CompletionResult]::new('--version', 'version', [CompletionResultType]::ParameterName, 'Prints version information')
            [CompletionResult]::new('list-used', 'list-used', [CompletionResultType]::ParameterValue, 'Print address list')
            [CompletionResult]::new('create', 'create', [CompletionResultType]::ParameterValue, 'create')
            [CompletionResult]::new('mark-used', 'mark-used', [CompletionResultType]::ParameterValue, 'mark-used')
            break
        }
        'mycitadel-cli;address;list-used' {
            [CompletionResult]::new('-r', 'r', [CompletionResultType]::ParameterName, 'Whether to re-scan addresses space with Electrum server')
            [CompletionResult]::new('--rescan', 'rescan', [CompletionResultType]::ParameterName, 'Whether to re-scan addresses space with Electrum server')
            [CompletionResult]::new('--lookup-depth', 'lookup-depth', [CompletionResultType]::ParameterName, 'How many addresses should be scanned at least after the final address with no transactions is reached')
            [CompletionResult]::new('-f', 'f', [CompletionResultType]::ParameterName, 'How the command output should be formatted')
            [CompletionResult]::new('--format', 'format', [CompletionResultType]::ParameterName, 'How the command output should be formatted')
            [CompletionResult]::new('-l', 'l', [CompletionResultType]::ParameterName, 'Limit the number of addresses printed')
            [CompletionResult]::new('--limit', 'limit', [CompletionResultType]::ParameterName, 'Limit the number of addresses printed')
            [CompletionResult]::new('-f', 'f', [CompletionResultType]::ParameterName, 'How the command output should be formatted')
            [CompletionResult]::new('--format', 'format', [CompletionResultType]::ParameterName, 'How the command output should be formatted')
            [CompletionResult]::new('-v', 'v', [CompletionResultType]::ParameterName, 'Set verbosity level')
            [CompletionResult]::new('--verbose', 'verbose', [CompletionResultType]::ParameterName, 'Set verbosity level')
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Prints help information')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Prints help information')
            [CompletionResult]::new('-V', 'V', [CompletionResultType]::ParameterName, 'Prints version information')
            [CompletionResult]::new('--version', 'version', [CompletionResultType]::ParameterName, 'Prints version information')
            break
        }
        'mycitadel-cli;address;create' {
            [CompletionResult]::new('-i', 'i', [CompletionResultType]::ParameterName, 'Create address at custom index number')
            [CompletionResult]::new('--index', 'index', [CompletionResultType]::ParameterName, 'Create address at custom index number')
            [CompletionResult]::new('-n', 'n', [CompletionResultType]::ParameterName, 'Number of addresses to create')
            [CompletionResult]::new('--no', 'no', [CompletionResultType]::ParameterName, 'Number of addresses to create')
            [CompletionResult]::new('-u', 'u', [CompletionResultType]::ParameterName, 'Whether to mark address as used')
            [CompletionResult]::new('--unmarked', 'unmarked', [CompletionResultType]::ParameterName, 'Whether to mark address as used')
            [CompletionResult]::new('--legacy', 'legacy', [CompletionResultType]::ParameterName, 'Use SegWit legacy address format (applicable only to a SegWit wallets)')
            [CompletionResult]::new('-v', 'v', [CompletionResultType]::ParameterName, 'Set verbosity level')
            [CompletionResult]::new('--verbose', 'verbose', [CompletionResultType]::ParameterName, 'Set verbosity level')
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Prints help information')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Prints help information')
            [CompletionResult]::new('-V', 'V', [CompletionResultType]::ParameterName, 'Prints version information')
            [CompletionResult]::new('--version', 'version', [CompletionResultType]::ParameterName, 'Prints version information')
            break
        }
        'mycitadel-cli;address;mark-used' {
            [CompletionResult]::new('--legacy', 'legacy', [CompletionResultType]::ParameterName, 'Use SegWit legacy address format (applicable only to a SegWit wallets)')
            [CompletionResult]::new('-u', 'u', [CompletionResultType]::ParameterName, 'Remove use mark (inverses the command)')
            [CompletionResult]::new('--unmark', 'unmark', [CompletionResultType]::ParameterName, 'Remove use mark (inverses the command)')
            [CompletionResult]::new('-v', 'v', [CompletionResultType]::ParameterName, 'Set verbosity level')
            [CompletionResult]::new('--verbose', 'verbose', [CompletionResultType]::ParameterName, 'Set verbosity level')
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Prints help information')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Prints help information')
            [CompletionResult]::new('-V', 'V', [CompletionResultType]::ParameterName, 'Prints version information')
            [CompletionResult]::new('--version', 'version', [CompletionResultType]::ParameterName, 'Prints version information')
            break
        }
        'mycitadel-cli;asset' {
            [CompletionResult]::new('-v', 'v', [CompletionResultType]::ParameterName, 'Set verbosity level')
            [CompletionResult]::new('--verbose', 'verbose', [CompletionResultType]::ParameterName, 'Set verbosity level')
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Prints help information')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Prints help information')
            [CompletionResult]::new('-V', 'V', [CompletionResultType]::ParameterName, 'Prints version information')
            [CompletionResult]::new('--version', 'version', [CompletionResultType]::ParameterName, 'Prints version information')
            [CompletionResult]::new('list', 'list', [CompletionResultType]::ParameterValue, 'Lists known assets')
            [CompletionResult]::new('import', 'import', [CompletionResultType]::ParameterValue, 'Import asset genesis data')
            break
        }
        'mycitadel-cli;asset;list' {
            [CompletionResult]::new('-f', 'f', [CompletionResultType]::ParameterName, 'How the asset list output should be formatted')
            [CompletionResult]::new('--format', 'format', [CompletionResultType]::ParameterName, 'How the asset list output should be formatted')
            [CompletionResult]::new('-v', 'v', [CompletionResultType]::ParameterName, 'Set verbosity level')
            [CompletionResult]::new('--verbose', 'verbose', [CompletionResultType]::ParameterName, 'Set verbosity level')
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Prints help information')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Prints help information')
            [CompletionResult]::new('-V', 'V', [CompletionResultType]::ParameterName, 'Prints version information')
            [CompletionResult]::new('--version', 'version', [CompletionResultType]::ParameterName, 'Prints version information')
            break
        }
        'mycitadel-cli;asset;import' {
            [CompletionResult]::new('-v', 'v', [CompletionResultType]::ParameterName, 'Set verbosity level')
            [CompletionResult]::new('--verbose', 'verbose', [CompletionResultType]::ParameterName, 'Set verbosity level')
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Prints help information')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Prints help information')
            [CompletionResult]::new('-V', 'V', [CompletionResultType]::ParameterName, 'Prints version information')
            [CompletionResult]::new('--version', 'version', [CompletionResultType]::ParameterName, 'Prints version information')
            break
        }
        'mycitadel-cli;help' {
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Prints help information')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Prints help information')
            [CompletionResult]::new('-V', 'V', [CompletionResultType]::ParameterName, 'Prints version information')
            [CompletionResult]::new('--version', 'version', [CompletionResultType]::ParameterName, 'Prints version information')
            break
        }
    })

    $completions.Where{ $_.CompletionText -like "$wordToComplete*" } |
        Sort-Object -Property ListItemText
}
