# =============================================================================
# Zenritme — Bash completion
# =============================================================================
# Static completion file. No runtime generation required.
# Install to: ${PREFIX}/share/bash-completion/completions/zenritme
# =============================================================================

_zenritme() {
    local cur prev words cword
    _init_completion -s || return

    local modes="--timer-up --timer-down --stopwatch --pomodoro"
    local info="--help -V --version --sound-test --check-update --check-updated"
    local opts="--theme --view --sound-profile --mute --focus --break --long-break --cycles"

    case "${prev}" in
        --theme)
            COMPREPLY=($(compgen -W "void ember aura forest tron tron-green tron-cyan tron-orange tron-red tron-yellow mono" -- "${cur}"))
            return ;;
        --view)
            COMPREPLY=($(compgen -W "minimal orbit cinematic tron" -- "${cur}"))
            return ;;
        --sound-profile)
            COMPREPLY=($(compgen -W "calm silent" -- "${cur}"))
            return ;;
        --timer-down|--focus|--break|--long-break)
            COMPREPLY=()
            return ;;
        --cycles)
            COMPREPLY=()
            return ;;
    esac

    if [[ "${cur}" == -* ]]; then
        COMPREPLY=($(compgen -W "${modes} ${info} ${opts}" -- "${cur}"))
    fi
}

complete -F _zenritme zenritme