#!/usr/bin/env bash

{ # this ensures the entire script is downloaded #

cmd_has() {
  type "$1" > /dev/null 2>&1
}

sn_cli_install_dir() {
  if [ -n "$SN_CLI_DIR" ]; then
    printf %s "${SN_CLI_DIR}"
  else
    printf %s "${HOME}/.safe/cli"
  fi
}

sn_cli_latest_version() {
  curl -s "https://api.github.com/repos/maidsafe/sn_api/releases/latest" | grep '"tag_name":' | sed -E 's/.*"v([^"]+)".*/\1/'
}

sn_cli_download() {
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

sn_cli_try_profile() {
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
sn_cli_detect_profile() {
  if [ "${PROFILE-}" = '/dev/null' ]; then
    # the user has specifically requested NOT to have Safe CLI set in their profile
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
      if detected_profile="$(sn_cli_try_profile "${HOME}/${each_profile}")"; then
        break
      fi
    done
  fi

  if [ -n "$detected_profile" ]; then
    echo "$detected_profile"
  fi
}

sn_cli_profile_is_bash_or_zsh() {
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

sn_cli_install() {
  platform="unknown-linux-musl"
  sn_cli_exec="safe"
  uname_output=$(uname -a)
  case $uname_output in
      Linux*)
          ;;
      Darwin*)
          platform="apple-darwin"
          ;;
      MSYS_NT* | MINGW*)
          platform="pc-windows-msvc"
          sn_cli_exec="safe.exe"
          ;;
      *)
          echo "Platform not supported by the Safe CLI installation script."
          exit 1
  esac

  cli_package="sn_cli-$(sn_cli_latest_version)-x86_64-$platform.tar.gz"
  cli_package_url="https://sn-api.s3.eu-west-2.amazonaws.com/$cli_package"
  tmp_dir=$(mktemp -d)
  tmp_dir_package=$tmp_dir/$cli_package

  echo "=> Downloading Safe CLI package from '$cli_package_url'..."
  sn_cli_download "$cli_package_url" -o "$tmp_dir_package"

  install_dir="$(sn_cli_install_dir)"
  echo "=> Unpacking Safe CLI to '$install_dir'..."
  mkdir -p "$install_dir"
  tar -xzf $tmp_dir_package -C $install_dir

  case $uname_output in
      Linux* | Darwin*)
          sn_cli_profile="$(sn_cli_detect_profile)"
          sn_cli_in_path_str="\\nexport PATH=\$PATH:$install_dir"

          if [ -z "${sn_cli_profile-}" ] ; then
            local tried_profile
            if [ -n "${PROFILE}" ]; then
              tried_profile="${sn_cli_profile} (as defined in \$PROFILE), "
            fi
            echo "=> Shell profile not found. Tried ${tried_profile-}~/.bashrc, ~/.bash_profile, ~/.zshrc, and ~/.profile"
            echo "=> Create one of them and run this script again"
            echo "   OR"
            echo "=> Append the following lines to the correct file yourself:"
            command printf "${sn_cli_in_path_str}"
            echo
          else
            echo "=> Adding statement to '$sn_cli_profile' profile to have Safe CLI binary path in the \$PATH"
            if ! command grep -qc "$install_dir" "$sn_cli_profile"; then
              command printf "${sn_cli_in_path_str}" >> "$sn_cli_profile"
              echo "=> Statement appended to '$sn_cli_profile' profile"
              echo "=> Close and reopen your terminal to start using Safe CLI"
            else
              echo "=> Profile '${sn_cli_profile}' already contains a statement to set Safe CLI in the \$PATH"
            fi
          fi
          ;;
      MSYS_NT* | MINGW*)
          if ! command grep -qc "$install_dir" <<< $PATH; then
            echo "=> Adding Safe CLI binary path to the PATH in the system for all users"
            setx PATH "$PATH:$install_dir" -m
            echo "=> Close and reopen your terminal to start using Safe CLI"
          else
            echo "=> Safe CLI binary path was already set in the PATH"
          fi
          ;;
  esac

  sn_cli_reset
}

#
# Unsets the various functions defined
# during the execution of the install script
#
sn_cli_reset() {
  unset -f sn_cli_try_profile sn_cli_download \
    sn_cli_latest_version sn_cli_install_dir \
    cmd_has sn_cli_detect_profile \
    sn_cli_profile_is_bash_or_zsh sn_cli_install
}

sn_cli_install

} # this ensures the entire script is downloaded #
