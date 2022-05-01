{ naersk, src, lib, pkg-config, cmake, protobuf }:

naersk.buildPackage {
  pname = "data-accumulator";
  version = "0.1.0";

  src = ./.;

  cargoSha256 = lib.fakeSha256;

  nativeBuildInputs = [ pkg-config cmake protobuf ];
  #buildInputs = [ openssl libpqxx libiconv postgresql git ];

  meta = with lib; {
    description = "Simple rust server which collects data from telegram stations";
    homepage = "https://github.com/dump-dvb/data-accumulator";
  };
}
