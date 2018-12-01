# RAM Utils
Utility program for my own needs

# Documentation
```
RAM Utils 0.1
Ralph Minderhoud <mail@ralphminderhoud.com>
Simple utilities

USAGE:
    ram-utils [SUBCOMMAND]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    help     Prints this message or the help of the given subcommand(s)
    lower    Convert files and/or directories to lower case
    upper    Convert files and/or directories to upper case
```

## Upper/Lower
```
ram-utils-upper 
Convert files and/or directories to upper case

USAGE:
    ram-utils upper [FLAGS] <path>

FLAGS:
    -h, --help            Prints help information
        --ignore-dirs     Ignore directories during conversion
        --ignore-files    Ignore files during conversion
    -r                    Convert directories recursively
    -V, --version         Prints version information

ARGS:
    <path>    File or directory path
```
