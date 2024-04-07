complete -c find_duplicate_files -s a -l algorithm -d 'Choose the hash algorithm' -r -f -a "{ahash	'',blake3	'',fxhash	'',sha256	'',sha512	''}"
complete -c find_duplicate_files -s e -l csv_dir -d 'Set the output directory for the CSV file (fdf.csv)' -r -F
complete -c find_duplicate_files -s x -l xlsx_dir -d 'Set the output directory for the XLSX file (fdf.xlsx)' -r -F
complete -c find_duplicate_files -s g -l generate -d 'If provided, outputs the completion file for given shell' -r -f -a "{bash	'',elvish	'',fish	'',powershell	'',zsh	''}"
complete -c find_duplicate_files -s d -l min_depth -d 'Set the minimum depth to search for duplicate files' -r
complete -c find_duplicate_files -s D -l max_depth -d 'Set the maximum depth to search for duplicate files' -r
complete -c find_duplicate_files -s b -l min_size -d 'Set a minimum file size (in bytes) to search for duplicate files' -r
complete -c find_duplicate_files -s B -l max_size -d 'Set a maximum file size (in bytes) to search for duplicate files' -r
complete -c find_duplicate_files -s i -l input_dir -d 'Set the input directory where to look for duplicate files [default: current directory]' -r -F
complete -c find_duplicate_files -s r -l result_format -d 'Print the result in the chosen format' -r -f -a "{json	'',yaml	'',personal	''}"
complete -c find_duplicate_files -s c -l clear_terminal -d 'Clear the terminal screen before listing the duplicate files'
complete -c find_duplicate_files -s f -l full_path -d 'Prints full path of duplicate files, otherwise relative path'
complete -c find_duplicate_files -s o -l omit_hidden -d 'Omit hidden files (starts with \'.\'), otherwise search all files'
complete -c find_duplicate_files -s s -l sort -d 'Sort result by number of duplicate files, otherwise sort by file size'
complete -c find_duplicate_files -s t -l time -d 'Show total execution time'
complete -c find_duplicate_files -s v -l verbose -d 'Show intermediate runtime messages'
complete -c find_duplicate_files -s h -l help -d 'Print help (see more with \'--help\')'
complete -c find_duplicate_files -s V -l version -d 'Print version'
