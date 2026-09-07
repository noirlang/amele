//! Amele CLI kabuk otomatik tamamlama (shell autocompletion) script üreticisi.
//! Bash, Zsh ve Fish kabukları için tamamlama betikleri oluşturur.

/// Bash otomatik tamamlama betiğini üretir.
pub fn generate_bash_completion() -> &'static str {
    r#"# Bash completion for amele
_amele_completion() {
    local cur prev words cword
    _init_completion || return

    local commands="linux windows ram disk android ios docker profile case mount hash verify update ui ui-browser help completion"
    local global_flags="--help -h --version -V --quiet -q --no-logo --verbose -v --lang --profile --json"

    if [[ $cword -eq 1 ]]; then
        COMPREPLY=( $(compgen -W "${commands} ${global_flags}" -- "$cur") )
        return 0
    fi

    case "${words[1]}" in
        linux)
            if [[ $cword -eq 2 ]]; then
                COMPREPLY=( $(compgen -W "disk ram --help -h" -- "$cur") )
            elif [[ "${words[2]}" == "disk" ]]; then
                if [[ $cword -eq 3 ]]; then
                    COMPREPLY=( $(compgen -W "--list analyze --agent --ssh --help -h" -- "$cur") )
                elif [[ "${words[3]}" == "analyze" ]]; then
                    COMPREPLY=( $(compgen -f -- "$cur") )
                fi
            elif [[ "${words[2]}" == "ram" ]]; then
                if [[ $cword -eq 3 ]]; then
                    COMPREPLY=( $(compgen -W "--status install --agent --ssh --help -h" -- "$cur") )
                fi
            fi
            ;;
        windows)
            if [[ $cword -eq 2 ]]; then
                COMPREPLY=( $(compgen -W "disk ram --help -h" -- "$cur") )
            elif [[ "${words[2]}" == "disk" ]]; then
                if [[ $cword -eq 3 ]]; then
                    COMPREPLY=( $(compgen -W "--list analyze --agent --ssh --help -h" -- "$cur") )
                elif [[ "${words[3]}" == "analyze" ]]; then
                    COMPREPLY=( $(compgen -f -- "$cur") )
                fi
            elif [[ "${words[2]}" == "ram" ]]; then
                if [[ $cword -eq 3 ]]; then
                    COMPREPLY=( $(compgen -W "--status install --agent --ssh --help -h" -- "$cur") )
                fi
            fi
            ;;
        disk)
            if [[ $cword -eq 2 ]]; then
                COMPREPLY=( $(compgen -W "--list analyze --agent --ssh --help -h" -- "$cur") )
            elif [[ "${words[2]}" == "analyze" ]]; then
                COMPREPLY=( $(compgen -f -- "$cur") )
            fi
            ;;
        ram)
            if [[ $cword -eq 2 ]]; then
                COMPREPLY=( $(compgen -W "--status install --agent --ssh --help -h" -- "$cur") )
            fi
            ;;
        android)
            if [[ $cword -eq 2 ]]; then
                COMPREPLY=( $(compgen -W "status devices profile logical filesystem ram capabilities lemon-preflight remote-connect remote-disconnect case-analysis --help -h" -- "$cur") )
            fi
            ;;
        ios)
            if [[ $cword -eq 2 ]]; then
                COMPREPLY=( $(compgen -W "profile normalize --help -h" -- "$cur") )
            fi
            ;;
        docker)
            if [[ $cword -eq 2 ]]; then
                COMPREPLY=( $(compgen -W "status list logs acquire remote-status remote-list remote-logs remote-acquire --help -h" -- "$cur") )
            fi
            ;;
        case)
            if [[ $cword -eq 2 ]]; then
                COMPREPLY=( $(compgen -W "list create export import verify info show --help -h" -- "$cur") )
            elif [[ "${words[2]}" =~ ^(export|import|verify)$ ]]; then
                COMPREPLY=( $(compgen -f -- "$cur") )
            fi
            ;;
        mount)
            if [[ $cword -eq 2 ]]; then
                COMPREPLY=( $(compgen -W "list cleanup unmount --help -h" -- "$cur") )
                if [[ -z "${COMPREPLY[0]}" ]]; then
                    COMPREPLY=( $(compgen -f -- "$cur") )
                fi
            fi
            ;;
        profile)
            if [[ $cword -eq 2 ]]; then
                COMPREPLY=( $(compgen -W "list create use select logout sync online-sync --help -h" -- "$cur") )
            fi
            ;;
        completion)
            if [[ $cword -eq 2 ]]; then
                COMPREPLY=( $(compgen -W "bash zsh fish" -- "$cur") )
            fi
            ;;
        hash|verify)
            COMPREPLY=( $(compgen -f -- "$cur") )
            ;;
        *)
            COMPREPLY=( $(compgen -W "${global_flags}" -- "$cur") )
            ;;
    esac
}

complete -F _amele_completion amele
"#
}

/// Zsh otomatik tamamlama betiğini üretir.
pub fn generate_zsh_completion() -> &'static str {
    r#"#compdef amele

_amele() {
    local -a commands
    commands=(
        'linux:Linux disk and RAM acquisition/analysis'
        'windows:Windows disk and RAM acquisition/analysis'
        'ram:Live RAM acquisition'
        'disk:Disk listing and raw/AFF4 image acquisition'
        'android:Android physical/logical forensics and ADB'
        'ios:iOS forensic backup and structure analysis'
        'docker:Docker container acquisition and evidence dump'
        'profile:Investigator and analyst profile management'
        'case:Forensic case vault and export/import packages'
        'mount:Mount and unmount forensic disk images read-only'
        'hash:Cryptographic hashing (SHA256, SHA1, MD5)'
        'verify:Verify image hashes against evidence logs'
        'update:Check for software updates'
        'ui:Launch Amele forensic GUI'
        'ui-browser:Open Amele GUI in default web browser'
        'completion:Generate shell autocompletions (bash, zsh, fish)'
        'help:Show help information'
    )

    _arguments -C \
        '(-h --help)'{-h,--help}'[Show help information]' \
        '(-V --version)'{-V,--version}'[Show version information]' \
        '(-q --quiet)'{-q,--quiet}'[Quiet mode, suppress ASCII banner]' \
        '--no-logo[Suppress ASCII banner]' \
        '(-v --verbose)'{-v,--verbose}'[Verbose output]' \
        '--lang[Select language (tr/en)]:language:(tr en)' \
        '--profile[Use specific analyst profile]:profile:' \
        '--json[Output in JSON format]' \
        '1: :->command' \
        '*:: :->args'

    case $state in
        command)
            _describe -t commands 'amele command' commands
            ;;
        args)
            case $line[1] in
                linux|windows)
                    local -a os_cmds
                    os_cmds=(
                        'disk:Disk acquisition and analysis'
                        'ram:Live RAM acquisition'
                    )
                    _describe -t os_cmds 'subcommand' os_cmds
                    ;;
                case)
                    local -a case_cmds
                    case_cmds=(
                        'list:List all forensic cases'
                        'create:Create a new case vault'
                        'export:Export case to .amelecase package'
                        'import:Import .amelecase package'
                        'verify:Verify package integrity'
                        'info:Display case details'
                    )
                    _describe -t case_cmds 'case subcommand' case_cmds
                    ;;
                mount)
                    local -a mount_cmds
                    mount_cmds=(
                        'list:List active mounts'
                        'cleanup:Unmount all active image mounts'
                        'unmount:Unmount forensic image'
                    )
                    _describe -t mount_cmds 'mount subcommand' mount_cmds
                    ;;
                profile)
                    local -a prof_cmds
                    prof_cmds=(
                        'list:List registered analyst profiles'
                        'create:Create a new analyst profile'
                        'use:Switch to profile'
                        'logout:Log out from active profile'
                        'sync:Synchronize with central server'
                    )
                    _describe -t prof_cmds 'profile subcommand' prof_cmds
                    ;;
                completion)
                    local -a shells
                    shells=('bash:Bash completion' 'zsh:Zsh completion' 'fish:Fish completion')
                    _describe -t shells 'shell' shells
                    ;;
            esac
            ;;
    esac
}

_amele "$@"
"#
}

/// Fish otomatik tamamlama betiğini üretir.
pub fn generate_fish_completion() -> &'static str {
    r#"# Fish completion for amele
complete -c amele -f

# Global flags
complete -c amele -s h -l help -d "Show help information"
complete -c amele -s V -l version -d "Show version information"
complete -c amele -s q -l quiet -d "Suppress ASCII logo banner"
complete -c amele -l no-logo -d "Suppress ASCII logo banner"
complete -c amele -s v -l verbose -d "Verbose output"
complete -c amele -l json -d "JSON output"
complete -c amele -l lang -x -a "tr en" -d "CLI language (tr/en)"
complete -c amele -l profile -x -d "Analyst profile"

# Subcommands
complete -c amele -n "__fish_use_subcommand" -a linux -d "Linux disk and RAM acquisition/analysis"
complete -c amele -n "__fish_use_subcommand" -a windows -d "Windows disk and RAM acquisition/analysis"
complete -c amele -n "__fish_use_subcommand" -a ram -d "Live RAM acquisition"
complete -c amele -n "__fish_use_subcommand" -a disk -d "Disk listing and raw/AFF4 image acquisition"
complete -c amele -n "__fish_use_subcommand" -a android -d "Android physical/logical forensics and ADB"
complete -c amele -n "__fish_use_subcommand" -a ios -d "iOS forensic backup and structure analysis"
complete -c amele -n "__fish_use_subcommand" -a docker -d "Docker container acquisition and evidence dump"
complete -c amele -n "__fish_use_subcommand" -a profile -d "Investigator and analyst profile management"
complete -c amele -n "__fish_use_subcommand" -a case -d "Forensic case vault and export/import packages"
complete -c amele -n "__fish_use_subcommand" -a mount -d "Mount and unmount forensic disk images read-only"
complete -c amele -n "__fish_use_subcommand" -a hash -d "Cryptographic hashing (SHA256, SHA1, MD5)"
complete -c amele -n "__fish_use_subcommand" -a verify -d "Verify image hashes against evidence logs"
complete -c amele -n "__fish_use_subcommand" -a update -d "Check for software updates"
complete -c amele -n "__fish_use_subcommand" -a ui -d "Launch Amele forensic GUI"
complete -c amele -n "__fish_use_subcommand" -a ui-browser -d "Open Amele GUI in default web browser"
complete -c amele -n "__fish_use_subcommand" -a completion -d "Generate shell completion scripts"

# Completion subcommands
complete -c amele -n "__fish_seen_subcommand_from completion" -a "bash zsh fish"
"#
}
