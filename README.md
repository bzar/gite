# gite

A simple HTTP server for serving files straight from a git repository.

    USAGE:
        gite [FLAGS] [OPTIONS] [path]

    FLAGS:
        -d, --debug      
        -h, --help       Prints help information
        -V, --version    Prints version information

    OPTIONS:
        -i, --interface <interface>     [default: 127.0.0.1:3000]
        -r, --revision <revision>       [default: master]

    ARGS:
        <path>     [default: .]
