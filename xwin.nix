{ lib
, stdenv
, rustPlatform
, fetchFromGitHub
}:

rustPlatform.buildRustPackage rec {
  pname = "xwin";
  version = "0.3.1";

  src = fetchFromGitHub {
    owner = "Jake-Shadle";
    repo = "xwin";
    rev = "${version}";

    hash = "sha256-W2sNmEXWwuzLDZg3IrOvOL81K2zrqjUzqfaifqeTtqs=";
  };

  cargoHash = "sha256-rC2OSYlx3pNC1RRuhk2tdwl0uEsY4TBZGNIBdIV8TZk=";
  doCheck = false;
}