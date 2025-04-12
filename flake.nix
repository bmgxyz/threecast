{
  description = ''
    Convert the National Weather Service's (NWS) Digital Instantaneous Precipitation Rate (DIPR)
    radar product from its native data format into more common vector GIS formats'';

  inputs.nixpkgs.url = "github:NixOS/nixpkgs";

  outputs =
    { self, nixpkgs }:
    {
      packages.x86_64-linux.default =
        let
          pkgs = import nixpkgs {
            system = "x86_64-linux";
          };
        in
        pkgs.rustPlatform.buildRustPackage {
          pname = "dipr";
          version = "0.1.0";
          src = ./.;
          cargoLock = {
            lockFile = ./Cargo.lock;
          };
        };
    };
}
