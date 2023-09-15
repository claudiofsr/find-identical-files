_find_duplicate_files() {
    local i cur prev opts cmd
    COMPREPLY=()
    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD-1]}"
    cmd=""
    opts=""

    for i in ${COMP_WORDS[@]}
    do
        case "${cmd},${i}" in
            ",$1")
                cmd="find_duplicate_files"
                ;;
            *)
                ;;
        esac
    done

    case "${cmd}" in
        find_duplicate_files)
            opts="-a -c -f -g -m -o -p -r -s -t -h -V --algorithm --clear_terminal --full_path --generate --max_depth --omit_hidden --path --result_format --sort --time --help --version"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 1 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --algorithm)
                    COMPREPLY=($(compgen -W "ahash blake3 fxhash sha256 sha512" -- "${cur}"))
                    return 0
                    ;;
                -a)
                    COMPREPLY=($(compgen -W "ahash blake3 fxhash sha256 sha512" -- "${cur}"))
                    return 0
                    ;;
                --generate)
                    COMPREPLY=($(compgen -W "bash elvish fish powershell zsh" -- "${cur}"))
                    return 0
                    ;;
                -g)
                    COMPREPLY=($(compgen -W "bash elvish fish powershell zsh" -- "${cur}"))
                    return 0
                    ;;
                --max_depth)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -m)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --path)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -p)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --result_format)
                    COMPREPLY=($(compgen -W "json yaml personal" -- "${cur}"))
                    return 0
                    ;;
                -r)
                    COMPREPLY=($(compgen -W "json yaml personal" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
    esac
}

complete -F _find_duplicate_files -o nosort -o bashdefault -o default find_duplicate_files
