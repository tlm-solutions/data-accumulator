{ buildPackage
, stdenv
, lib
, pkg-config
, cmake
, protobuf
, postgresql_14
, zlib
, openssl
}:

let
  data-accumulator = buildPackage {
    pname = "data-accumulator";
    version = "0.5.0";

    src = ./.;

    cargoSha256 = lib.fakeSha256;

    nativeBuildInputs = [ pkg-config cmake ];
    buildInputs = [ protobuf zlib postgresql_14 openssl ];

    meta = {
      description = "Simple rust server which collects data from telegram stations";
      homepage = "https://github.com/tlm-solutions/data-accumulator";
    };
  };
in
stdenv.mkDerivation {
  name = "data-accumulator-patchelf";
  src = data-accumulator;

  buildPhase = "";

  installPhase = ''
    mkdir -p $out/bin
    cp $src/bin/data-accumulator $out/bin/data-accumulator
    chmod +w $out/bin/data-accumulator
  
    patchelf --replace-needed libpq.so.5 ${postgresql_14.lib}/lib/libpq.so $out/bin/data-accumulator

    # check if the patch succeded and exit if a depedency is not found
    local patch_succeded

    patch_succeded=$(ldd $out/bin/data-accumulator | grep "not found" | wc -l || true)
    if [[ "$patch_succeded" -ne 0 ]]; then
      echo "Patching failed" && exit 1
    fi
  '';
}