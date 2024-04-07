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
            opts="-a -c -e -x -f -g -d -D -b -B -o -i -r -s -t -v -h -V --algorithm --clear_terminal --csv_dir --xlsx_dir --full_path --generate --min_depth --max_depth --min_size --max_size --omit_hidden --input_dir --result_format --sort --time --verbose --help --version"
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
                --csv_dir)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -e)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --xlsx_dir)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -x)
                    COMPREPLY=($(compgen -f "${cur}"))
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
                --min_depth)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -d)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --max_depth)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -D)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --min_size)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -b)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --max_size)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -B)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --input_dir)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -i)
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

if [[ "${BASH_VERSINFO[0]}" -eq 4 && "${BASH_VERSINFO[1]}" -ge 4 || "${BASH_VERSINFO[0]}" -gt 4 ]]; then
    complete -F _find_duplicate_files -o nosort -o bashdefault -o default find_duplicate_files
else
    complete -F _find_duplicate_files -o bashdefault -o default find_duplicate_files
fi
