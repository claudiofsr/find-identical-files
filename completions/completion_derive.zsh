#compdef find_duplicate_files

autoload -U is-at-least

_find_duplicate_files() {
    typeset -A opt_args
    typeset -a _arguments_options
    local ret=1

    if is-at-least 5.2; then
        _arguments_options=(-s -S -C)
    else
        _arguments_options=(-s -C)
    fi

    local context curcontext="$curcontext" state line
    _arguments "${_arguments_options[@]}" \
'-a+[Choose the hash algorithm]:ALGORITHM:(ahash blake3 fxhash sha256 sha512)' \
'--algorithm=[Choose the hash algorithm]:ALGORITHM:(ahash blake3 fxhash sha256 sha512)' \
'-g+[If provided, outputs the completion file for given shell]:GENERATOR:(bash elvish fish powershell zsh)' \
'--generate=[If provided, outputs the completion file for given shell]:GENERATOR:(bash elvish fish powershell zsh)' \
'-m+[Set the maximum depth to search for duplicate files]:MAX_DEPTH: ' \
'--max_depth=[Set the maximum depth to search for duplicate files]:MAX_DEPTH: ' \
'-b+[Set a minimum file size to search for duplicate files]:MIN_SIZE: ' \
'--min_size=[Set a minimum file size to search for duplicate files]:MIN_SIZE: ' \
'-p+[Set the path where to look for duplicate files, otherwise use the current directory]:PATH:_files' \
'--path=[Set the path where to look for duplicate files, otherwise use the current directory]:PATH:_files' \
'-r+[Print the result in the chosen format]:RESULT_FORMAT:(json yaml personal)' \
'--result_format=[Print the result in the chosen format]:RESULT_FORMAT:(json yaml personal)' \
'-c[Clear the terminal screen before listing the duplicate files]' \
'--clear_terminal[Clear the terminal screen before listing the duplicate files]' \
'-f[Prints full path of duplicate files, otherwise relative path]' \
'--full_path[Prints full path of duplicate files, otherwise relative path]' \
'-o[Omit hidden files (starts with '\''.'\''), otherwise search all files]' \
'--omit_hidden[Omit hidden files (starts with '\''.'\''), otherwise search all files]' \
'-s[Sort result by file size, otherwise sort by number of duplicate files]' \
'--sort[Sort result by file size, otherwise sort by number of duplicate files]' \
'-t[Show total execution time]' \
'--time[Show total execution time]' \
'-v[Show intermediate runtime messages]' \
'--verbose[Show intermediate runtime messages]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
'-V[Print version]' \
'--version[Print version]' \
&& ret=0
}

(( $+functions[_find_duplicate_files_commands] )) ||
_find_duplicate_files_commands() {
    local commands; commands=()
    _describe -t commands 'find_duplicate_files commands' commands "$@"
}

if [ "$funcstack[1]" = "_find_duplicate_files" ]; then
    _find_duplicate_files "$@"
else
    compdef _find_duplicate_files find_duplicate_files
fi
