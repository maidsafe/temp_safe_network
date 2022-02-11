#!/usr/bin/env bash

{ # this ensures the entire script is downloaded #

cmd_has() {
  type "$1" > /dev/null 2>&1
}

sn_cli_install_dir() {
  user_id=$(id -u)
  if [ $user_id -eq 0 ]; then
    # If we're running as sudo, install to /usr/local/bin. It's almost always
    # on PATH, even in a minimal setup.
    echo "/usr/local/bin"
  else
    echo "${HOME}/.safe/cli"
  fi
}

get_download_url_for_latest_version() {
  local platform="$1"
  local download_url=$(curl -s "https://api.github.com/repos/maidsafe/safe_network/releases/latest" \
    | grep "sn_cli.*-$platform.tar.gz" \
    | grep "browser_download_url" \
    | awk -F ':' '{ print $3 }' \
    | sed 's/"//')
  echo "https:$download_url"
}

sn_cli_download() {
  if cmd_has "curl"; then
    curl -L --compressed -q "$@"
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
  arch="x86_64"
  platform="unknown-linux-musl"
  sn_cli_exec="safe"
  uname_output=$(uname -a)
  case $uname_output in
    *aarch64*)
      arch="aarch64"
      ;;
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

  if [ -z $safe_version ]; then
    cli_package_url=$(get_download_url_for_latest_version "$arch-$platform")
    archive_file_name=$(awk -F '/' '{ print $9 }' <<< $cli_package_url)
  else
    cli_package_url="https://sn-cli.s3.eu-west-2.amazonaws.com/sn_cli-$safe_version-$arch-$platform.tar.gz"
    archive_file_name=$(awk -F '/' '{ print $3 }' <<< $cli_package_url)
  fi
  tmp_dir=$(mktemp -d)
  tmp_archive_path=$tmp_dir/$archive_file_name

  echo "=> Downloading Safe CLI package from '$cli_package_url'..."
  echo "=> Saving to '$tmp_archive_path'..."
  sn_cli_download "$cli_package_url" -o "$tmp_archive_path"

  install_dir="$(sn_cli_install_dir)"
  echo "=> Unpacking Safe CLI to '$install_dir'..."
  mkdir -p "$install_dir"
  tar -xzf $tmp_archive_path -C $install_dir

  # Set this *just in case* the release process has been modified and the binary in the archive
  # hasn't been set to executable (though ideally it should be and that bug should be fixed).
  chmod +x "$install_dir/safe"

  user_id=$(id -u)
  if [ $user_id -eq 0 ]; then
    # If we're running as sudo we're installing to a system-wide location,
    # so we don't need to modify the ~/.bashrc or whatever.
    return
  fi

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

usage() {
  printf "Usage: $0 [-v=<version> or --version=<version>]\n\n"
  printf "To install a specific version of safe, you can optionally supply a version number.\n"
  printf "You should supply it without the 'v' prefix.\n\n"
  printf "Example: ./install.sh --version=0.33.0\n\n"
  printf "If no version number is supplied, the latest version will be installed.\n"
  exit 1
}

# It would have been nice to wrap the argument parsing in a little function, but
# unfortunately it seems to be the case that you can't consume "$@" inside a
# function.

# The reason for doing this manually and not using getopts or getopt, is because
# macOS doesn't have a getopts install and the version of getopt it has by
# default is not the GNU one, so the behaviour will be different on macOS and
# Linux. For that reason, we just parse them 'manually'.
safe_version=""
for arg in "$@"; do
  case $arg in
    -v=*|--version=*)
      safe_version="${arg#*=}"
      shift
      ;;
    *)
      printf "The %s argument is not supported\n\n" "$arg"
      usage
      ;;
  esac
done

sn_cli_install

} # this ensures the entire script is downloaded #
