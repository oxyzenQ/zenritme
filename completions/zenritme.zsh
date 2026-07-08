#compdef zenritme
# =============================================================================
# Zenritme — Zsh completion
# =============================================================================
# Static completion file. No runtime generation required.
# Install to: ${PREFIX}/share/zsh/site-functions/_zenritme
# =============================================================================

_zenritme() {
    local -a modes=(
        '--timer-up[Count-up timer]'
        '--timer-down[Count-down timer]:duration:'
        '--stopwatch[Stopwatch mode]'
        '--pomodoro[Pomodoro ritual]'
    )

    local -a info=(
        '--help[Show usage information]'
        '-V[Show version information]'
        '--version[Show version information]'
        '--sound-test[Preview notification sounds]'
        '--check-update[Check for newer release (read-only)]'
        '--check-updated[Alias for --check-update]'
    )

    local -a opts=(
        '--theme[Color theme]:theme:(void ember aura forest tron tron-green tron-cyan tron-orange tron-red tron-yellow mono)'
        '--view[Display view mode]:view:(minimal orbit cinematic)'
        '--sound-profile[Sound profile]:profile:(calm silent)'
        '--mute[Suppress all notification sounds]'
        '--focus[Focus session length]:duration:'
        '--break[Short break length]:duration:'
        '--long-break[Long break length]:duration:'
        '--cycles[Focus sessions per round]:number:'
    )

    _arguments -s \
        "${modes[@]}" \
        "${info[@]}" \
        "${opts[@]}"
}

_zenritme "$@"