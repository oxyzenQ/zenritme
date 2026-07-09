# =============================================================================
# Zenritme — Fish completion
# =============================================================================
# Static completion file. No runtime generation required.
# Install to: ${PREFIX}/share/fish/vendor_completions.d/zenritme.fish
# =============================================================================

# Disable file completions globally for zenritme
complete -c zenritme -f

# Modes
complete -c zenritme -n '__fish_is_first_arg' -l timer-up -d 'Count-up timer'
complete -c zenritme -n '__fish_is_first_arg' -l timer-down -d 'Count-down timer'
complete -c zenritme -n '__fish_is_first_arg' -l stopwatch -d 'Stopwatch mode'
complete -c zenritme -n '__fish_is_first_arg' -l pomodoro -d 'Pomodoro ritual'

# Info commands
complete -c zenritme -n '__fish_is_first_arg' -l help -d 'Show usage information'
complete -c zenritme -n '__fish_is_first_arg' -s V -l version -d 'Show version information'
complete -c zenritme -n '__fish_is_first_arg' -l sound-test -d 'Preview notification sounds'
complete -c zenritme -n '__fish_is_first_arg' -l check-update -d 'Check for newer release (read-only)'
complete -c zenritme -n '__fish_is_first_arg' -l check-updated -d 'Alias for --check-update'

# Options
complete -c zenritme -l theme -d 'Color theme' -xa 'void ember aura forest tron tron-green tron-cyan tron-orange tron-red tron-yellow mono'
complete -c zenritme -l view -d 'Display view mode' -xa 'minimal orbit cinematic tron'
complete -c zenritme -l sound-profile -d 'Sound profile' -xa 'calm silent'
complete -c zenritme -l mute -d 'Suppress all notification sounds'
complete -c zenritme -l focus -d 'Focus session length'
complete -c zenritme -l break -d 'Short break length'
complete -c zenritme -l long-break -d 'Long break length'
complete -c zenritme -l cycles -d 'Focus sessions per round'