#tested on Linux Mint 19.1
#examples/documentation:
#1: - complete -F _dkms dkms 
#   - type _dkms
#2: - https://linux.die.net/man/1/bash: 'Programmable Completion'-chapter
#5: - https://debian-administration.org/article/317/An_introduction_to_bash_completion_part_2

_safe_get_subcmds()
{
  #cmd=$1
  #subcmd=$2 (if given)

  #if subcmds stays empty, then the caller of this function know to not call this fu. again
  #no local variable (see fu _safe() below)
  subcmds=""
  case $1 in
    auth)
      case $2 in
        clear|help)
          :
          ;;
        *)
          subcmds="clear help"
          ;;
      esac;
      ;;
    cat)
      #-i|--info autocomplete -> only once?
      comp_opts+=" safe:// -i --info"
      ;;
    container)
      case $2 in
        add|create)
          #--link safe://... --name <container> #not sure if container=safe://...
          comp_opts+=" --link"
          ;;
        edit)
          #--key <key>
          comp_opts+=" --key"
          ;;
        help)
          :
          ;;
        *)
          subcmds="add create edit help"
          :
          ;;
      esac
      ;;
    files)
      case $2 in
        help)
          :
          ;;
        put)
          #<location: local>
          comp_file="-f"
          #<dest: safe://...>
          comp_opts+=" safe:// -r --recursive"
          ;;
        sync)
          #<location: local>
          comp_file="-f"
          #<target: safe://...>
          comp_opts+=" safe:// -r --recursive"
          ;;
        *)
          subcmds="help put sync"
          ;;
      esac
      ;;
    keys)
      case $2 in
        balance)
          #--keyurl <keyurl> --sk <secret>"
          comp_opts+="--keyurl --sk"
          ;;
        
        create)
          #--pay-with <pay_with> --pk <pk> --preload <preload>"
          comp_opts+="-w --pay-with --pk --preload"
          ;;
        help)
          :
          ;;
        *)
          subcmds="balance create help"    
          ;;
      esac
      ;;
    nrs)
      case $2 in
        add)
          #-l|--link safe://... <name: do nothing>
          comp_opts+=" -l --link safe:// -t --direct --default"
          ;;
        create)
          #-l|--link safe://... <name: do nothing>
          comp_opts+=" -l --link safe:// -t --direct"
          ;;
        help)
          :
          ;;
        remove)
          #<name: do nothing>
          :
          ;;
        *)
          subcmds="add create help remove"
          ;;
      esac
      ;;
    safe-id)
      case $2 in
        create|update)
          #--email <email> --name <name> --surname <surname> --wallet <wallet> --website <website>
          comp_opts+=" --email --name --surname --wallet --website"
          ;;
        help)
          ;;
        *)
          subcmds="create help update"
          ;;
      esac
      ;;
    wallet)
      case $2 in
        #<target: safe://...>
        balance)
          comp_opts+=" safe://"
          ;;
        create)
          #--keyurl <keyurl> --name <name> -w|--pay-with <pay_with> --preload <preload> --sk <secret_key>
          comp_opts+=" --no-balance --test-coins --keyurl --name -w --pay-with --preload --sk"
          ;;
        help)
          :
          ;;
        insert)
          #<target: safe://...> --keyurl <keyurl> --name <name> -w|--pay-with <pay_with> --sk <secret_key>
          comp_opts+=" safe:// --default --keyurl --name -w --pay-with --sk"
          ;;
        transfer)
          #<amount> <to: safe://...> <from: safe://...>
          comp_opts+=" safe://"
          ;;
        *)
          subcmds="balance create help insert transfer"
          ;;
      esac
      ;;
    #There shouldn't be any other option be possible
    help|keypair|update)
    #*)
      :
      ;;
  esac;
}
_safe()
{
  local cur sregex
  cur="${COMP_WORDS[COMP_CWORD]}"
  #necessary for ':' in safe://<tab> -> google this and COMP_WORDBREAK for more details
  _get_comp_words_by_ref -n : cur
  COMPREPLY=()

  sregex="^safe://.*/"

  if [[ "$cur" =~ $sregex ]] ; then
    #    0123456789...   89..: offset 7
    #    prefix
    #           suffix
    #cur=safe://spath/ssubpath/...
    local suffix=${cur:7}
    local spath=${suffix%%/*}

    local xor_urls=($(safe cat safe://$spath/ 2> /dev/null|grep "^| \/"|awk '{print substr($2,2)}'|sort -u))
    #full_xor_urls=$(safe cat $cur 2> /dev/null|grep "^| \/"|awk '{print substr($2,2)}'|sort -u)
    local full_xor_urls=()
    for el in "${xor_urls[@]}"; do
      full_xor_urls+=("safe://$spath/${el}")
    done

    #for some reason expanding of array in compgen doesn't work, after flattening it works
    local full_xor_urls_flat=${full_xor_urls[@]}

    local tmp_compreply=($(compgen -W "$full_xor_urls_flat" -- "$cur"))

    local _cr_el cur_len=${#cur}
    #make assiocative compreply:
    # - to get rid of duplicates, without needing sort
    # - automatically local
    declare -A as_compreply=()
    for cr_el in ${tmp_compreply[@]} ; do
      #get rid of first part so we can get rid of everything after correct '/'-char
      _cr_el=${cr_el:$cur_len}

      #get rid of everything after first '/'-char, if present
      if [[ $_cr_el =~ "/" ]] ; then
        _cr_el=${_cr_el/\/*/\/}
      fi

      #prepend first part again
      _cr_el=${cr_el::$cur_len}$_cr_el

      #used assiociative array->overwrite already existing entries
      as_compreply["$_cr_el"]=1
    done
    COMPREPLY=("${!as_compreply[@]}")

    #if COMPREPLY returns 1 element and last char is '/' (->dir): append no nospace
    [[ ${#COMPREPLY[@]} == 1 ]] && [[ "${COMPREPLY[0]: -1}" == '/' ]] && compopt -o nospace

    #necessary for ':' in safe://<tab> -> google this and COMP_WORDBREAK for more details
    __ltrim_colon_completions "$cur"
  else
    local cmds flags long_flags opts long_opts comp_opts _cmd cmd subcmds subcmd cur i comp_file
    #local prev
    
    cmds="auth cat container files help keypair keys nrs safe-id update wallet"

    #flags can come everywhere
    flags="-n -h -V"
    long_flags="--dry-run --help --json --version"
    #default options and flags can come everywhere
    opts="-o"
    long_opts="--output --xorurl"

    cmd=""
    subcmds=""

    for ((i=1; i < COMP_CWORD; i++ ))
    do
      if [ -z "$cmd" ]; then
        for _cmd in $cmds; do
          if [[ ${COMP_WORDS[i]} == $_cmd ]] ; then
            cmd=$_cmd
            #sets subcmds sys var
            _safe_get_subcmds $cmd
          fi
        done
      elif [ -n "$subcmds" ] ; then
        _safe_get_subcmds $cmd "${COMP_WORDS[i]}"
      fi
    done;
    SREGEX="^safe://.*/"

    if [ -z "$cmd" ] ; then
      comp_opts+=" "$cmds
      comp_opts+=$flags
      comp_opts+=" "$long_flags
      comp_opts+=" "$opts
      comp_opts+=" "$long_opts
    fi
    if [ -n "$subcmds" ] ; then
      comp_opts+=" "$subcmds
    fi

    COMPREPLY=($(compgen $comp_file -W "$comp_opts" -- $cur))
    #necessary for ':' in safe://<tab> -> google this and COMP_WORDBREAK for more details
    __ltrim_colon_completions "$cur"
    #We don't want a space appended after safe://
    #sometimes return is not safe:// but // (':' causes split into 'safe', ':' and '//'...)
    [[ "$COMPREPLY" == *// ]] && compopt -o nospace
  fi
}

complete -F _safe safe
