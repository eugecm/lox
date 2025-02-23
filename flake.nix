{
  description = "Lox";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    crane.url = "github:ipetkov/crane";

    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, crane, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};

        craneLib = crane.mkLib pkgs;

        commonArgs = {
          src = craneLib.cleanCargoSource ./.;
          strictDeps = true;

          buildInputs = [
          ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
            pkgs.libiconv
          ];
        };

        lox = craneLib.buildPackage (commonArgs // {
          cargoArtifacts = craneLib.buildDepsOnly commonArgs;
        });
      in
      {
        checks = {
          inherit lox;
        };

        packages.default = lox;

        apps.default = flake-utils.lib.mkApp {
          drv = lox;
        };

        devShells.default = craneLib.devShell {
          checks = self.checks.${system};

          packages = [
          ];
        };
      });
}
