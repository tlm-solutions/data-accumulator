{ buildPackage
, lib
, pkg-config
, cmake
, protobuf
, postgresql_14
, zlib
, openssl
}:

buildPackage {
  pname = "data-accumulator";
  version = "0.5.0";

  src = ./.;

  cargoSha256 = lib.fakeSha256;

  nativeBuildInputs = [ pkg-config cmake ];
  buildInputs = [ protobuf zlib postgresql_14 openssl ];

  meta = {
    description = "Simple rust server which collects data from telegram stations";
    homepage = "https://github.com/dump-dvb/data-accumulator";
  };
}
