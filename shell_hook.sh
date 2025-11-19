# Aethr shell hook (append to ~/.bashrc or ~/.zshrc)
export AETHR_CMD_LOG="${HOME}/.aethr/commands.log"
aethr_log_command() {
  local exit_code=$?
  if [ $exit_code -eq 0 ]; then
    local cmd
    cmd="$(history 1 | sed 's/^ *[0-9]* *//')"
    mkdir -p "$(dirname "$AETHR_CMD_LOG")"
    printf '%s\t%s\n' "$(date +%s)" "$cmd" >> "$AETHR_CMD_LOG"
  fi
}
if [ -n "$BASH_VERSION" ]; then
  PROMPT_COMMAND='aethr_log_command; '"${PROMPT_COMMAND-}"
fi
if [ -n "$ZSH_VERSION" ]; then
  autoload -Uz add-zsh-hook
  add-zsh-hook precmd aethr_log_command
fi
