# Data Accumulator

![](https://img.shields.io/endpoint?label=data-accumulator.x86_64-linux&logo=github&logoColor=red&url=https%3A%2F%2Fhydra.hq.c3d2.de%2Fjob%2Fdvb-dump%2Fdata-accumulator%2Fdata-accumulator.x86_64-linux%2Fshield)
[![built with nix](https://builtwithnix.org/badge.svg)](https://builtwithnix.org)


Zentral piece of software which defines the endpoint where radio stations submitt there data. It also does deduplication of telegrams and authentication 
of stations and forwards the data to other services over GRPC or writes it into the database.

## Building

```
    nix build
```

## Conifguration

If you are using our flake I suggest taking a look at the options documentented here: (**TODO**)

### Environment Variables

**Database and Storage**

- **DATABASE_BACKEND**: possible values **POSTGRES**, **CSVFILE**, **EMPTY** if unspecified **EMPTY** is used
- **POSTGRES_HOST**: only necessary if **DATABASE_BACKEND** is **POSTGRES**
- **POSTGRES_PORT**: only necessary if **DATABASE_BACKEND** is **POSTGRES**
- **POSTGRES_TELEGRAMS_PASSWORD**: only necessary if **DATABASE_BACKEND** is **POSTGRES**
- **GRPC_HOST_X**: X can be an arbitrary value all environment variables which 
        fit this prefix will be interpreted as hosts where data should be send to 
        via grpc.


