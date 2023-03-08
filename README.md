# Data Accumulator

![](https://img.shields.io/endpoint?label=data-accumulator.x86_64-linux&logo=github&logoColor=red&url=https%3A%2F%2Fhydra.hq.c3d2.de%2Fjob%2Ftlm-solutions%2Fdata-accumulator%2Fdata-accumulator.x86_64-linux%2Fshield)
[![built with nix](https://builtwithnix.org/badge.svg)](https://builtwithnix.org)


Zentral piece of software which defines the endpoint where radio stations submitt there data. It also does deduplication of telegrams and authentication 
of stations and forwards the data to other services over GRPC or writes it into the database.

## Building

```bash
    $ nix build
```

## Conifguration

If you are using our flake I suggest taking a look at the options documentented here: (**TODO**)

### Environment Variables

**Database and Storage**

- **POSTGRES_HOST**: default host "127.0.0.1"
- **POSTGRES_PORT**: default port 8080
- **POSTGRES_USER** user for for postgres default datacare
- **POSTGRES_DATABASE** database to use default is tlms
- **POSTGRES_TELEGRAMS_PASSWORD**: default pw "default_pw"
- **GRPC_HOST_X**: X can be an arbitrary value all environment variables which 
        fit this prefix will be interpreted as hosts where data should be send to 
        via grpc.

### Commandline Arguments


```bash
data-accumulator 0.3.0
dump@dvb.solutions
data collection server with authentication and statistics

USAGE:
    data-accumulator [OPTIONS]

OPTIONS:
    -h, --host <HOST>    [default: 127.0.0.1]
        --help           Print help information
    -o, --offline
    -p, --port <PORT>    [default: 8080]
    -v, --verbose
    -V, --version        Print version information
```

The host and port option reference under which address the REST-Server should run which is exposed so client boxes 
can talk with this piece of software. Secondly the option `--offline` deactivates all authentication and postgres needs 
this option if primarly used by the `mobile-box`.
