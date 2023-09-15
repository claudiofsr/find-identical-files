#!/usr/bin/env bash

# find_duplicate_files --generate=bash > completions/completion_derive.bash
# find_duplicate_files --generate=elvish > completions/completion_derive.elvish
# find_duplicate_files --generate=fish > completions/completion_derive.fish
# find_duplicate_files --generate=powershell > completions/completion_derive.powershell
# find_duplicate_files --generate=zsh > completions/completion_derive.zsh

shells="bash elvish fish powershell zsh"

for shell in $shells; do
 find_duplicate_files --generate=$shell > completions/completion_derive.$shell
done
