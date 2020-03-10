#!/usr/bin/env bash

{ # this ensures the entire script is downloaded #

cmd_has() {
  type "$1" > /dev/null 2>&1
}

safe_cli_install_dir() {
  if [ -n "$SAFE_CLI_DIR" ]; then
    printf %s "${SAFE_CLI_DIR}"
  else
    printf %s "${HOME}/.safe/cli"
  fi
}

safe_cli_latest_version() {
  echo "0.9.0"
}

safe_cli_download() {
  if cmd_has "curl"; then
    curl --compressed -q "$@"
  elif cmd_has "wget"; then
    # Emulate curl with wget
    ARGS=$(echo "$*" | command sed -e 's/--progress-bar /--progress=bar /' \
                            -e 's/-L //' \
                            -e 's/--compressed //' \
                            -e 's/-I /--server-response /' \
                            -e 's/-s /-q /' \
                            -e 's/-o /-O /' \
                            -e 's/-C - /-c /')
    # shellcheck disable=SC2086
    eval wget $ARGS
  fi
}

safe_cli_try_profile() {
  if [ -z "${1-}" ] || [ ! -f "${1}" ]; then
    return 1
  fi
  echo "${1}"
}

#
# Detect profile file if not specified as environment variable
# (eg: PROFILE=~/.myprofile)
# The echo'ed path is guaranteed to be an existing file
# Otherwise, an empty string is returned
#
safe_cli_detect_profile() {
  if [ "${PROFILE-}" = '/dev/null' ]; then
    # the user has specifically requested NOT to have SAFE CLI set in their profile
    return
  fi

  if [ -n "${PROFILE}" ] && [ -f "${PROFILE}" ]; then
    echo "${PROFILE}"
    return
  fi

  detected_profile=''

  if [ -n "${BASH_VERSION-}" ]; then
    if [ -f "$HOME/.bashrc" ]; then
      detected_profile="$HOME/.bashrc"
    elif [ -f "$HOME/.bash_profile" ]; then
      detected_profile="$HOME/.bash_profile"
    fi
  elif [ -n "${ZSH_VERSION-}" ]; then
    detected_profile="$HOME/.zshrc"
  fi

  if [ -z "$detected_profile" ]; then
    for each_profile in ".profile" ".bashrc" ".bash_profile" ".zshrc"
    do
      if detected_profile="$(safe_cli_try_profile "${HOME}/${each_profile}")"; then
        break
      fi
    done
  fi

  if [ -n "$detected_profile" ]; then
    echo "$detected_profile"
  fi
}

safe_cli_profile_is_bash_or_zsh() {
  test_profile="${1-}"
  case "${test_profile-}" in
    *"/.bashrc" | *"/.bash_profile" | *"/.zshrc")
      return
    ;;
    *)
      return 1
    ;;
  esac
}

safe_cli_install() {
  platform="unknown-linux-gnu"
  safe_cli_exec="safe"
  uname_output=$(uname -a)
  case $uname_output in
      Linux*)
          ;;
      Darwin*)
          platform="apple-darwin"
          ;;
      MSYS_NT* | MINGW*)
          platform="pc-windows-msvc"
          safe_cli_exec="safe.exe"
          ;;
      *)
          echo "Platform not supported by the SAFE CLI installation script."
          exit 1
  esac

  cli_package="safe-cli-$(safe_cli_latest_version)-x86_64-$platform.tar.gz"
  cli_package_url="https://safe-api.s3.eu-west-2.amazonaws.com/$cli_package"
  tmp_dir=$(mktemp -d)
  tmp_dir_package=$tmp_dir/$cli_package

  echo "=> Downloading SAFE CLI package from '$cli_package_url'..."
  safe_cli_download "$cli_package_url" -o "$tmp_dir_package"

  install_dir="$(safe_cli_install_dir)"
  echo "=> Unpacking SAFE CLI to '$install_dir'..."
  mkdir -p "$install_dir"
  tar -xzf $tmp_dir_package -C $install_dir

  case $uname_output in
      Linux* | Darwin*)
          safe_cli_profile="$(safe_cli_detect_profile)"
          safe_cli_in_path_str="\\nexport PATH=\$PATH:$install_dir"

          if [ -z "${safe_cli_profile-}" ] ; then
            local tried_profile
            if [ -n "${PROFILE}" ]; then
              tried_profile="${safe_cli_profile} (as defined in \$PROFILE), "
            fi
            echo "=> Shell profile not found. Tried ${tried_profile-}~/.bashrc, ~/.bash_profile, ~/.zshrc, and ~/.profile"
            echo "=> Create one of them and run this script again"
            echo "   OR"
            echo "=> Append the following lines to the correct file yourself:"
            command printf "${safe_cli_in_path_str}"
            echo
          else
            echo "=> Adding statement to '$safe_cli_profile' profile to have SAFE CLI binary path in the \$PATH"
            if ! command grep -qc "$install_dir" "$safe_cli_profile"; then
              command printf "${safe_cli_in_path_str}" >> "$safe_cli_profile"
              echo "=> Statement appended to '$safe_cli_profile' profile"
              echo "=> Close and reopen your terminal to start using SAFE CLI"
            else
              echo "=> Profile '${safe_cli_profile}' already contains a statement to set SAFE CLI in the \$PATH"
            fi
          fi
          ;;
      MSYS_NT* | MINGW*)
          if ! command grep -qc "$install_dir" <<< $PATH; then
            echo "=> Adding SAFE CLI binary path to the PATH in the system for all users"
            setx PATH "$PATH:$install_dir" -m
            echo "=> Close and reopen your terminal to start using SAFE CLI"
          else
            echo "=> SAFE CLI binary path was already set in the PATH"
          fi
          ;;
  esac

  safe_cli_reset
}

#
# Unsets the various functions defined
# during the execution of the install script
#
safe_cli_reset() {
  unset -f safe_cli_try_profile safe_cli_download \
    safe_cli_latest_version safe_cli_install_dir \
    cmd_has safe_cli_detect_profile \
    safe_cli_profile_is_bash_or_zsh safe_cli_install
}

safe_cli_install

} # this ensures the entire script is downloaded #
