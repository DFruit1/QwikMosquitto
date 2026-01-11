{
  description = "Qwik + Rust MQTT recorder with Postgres and Mosquitto";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.05";
    flake-utils.url = "github:numtide/flake-utils";
    nixhomeserver.url = "github:example/nixhomeserver";
  };

  outputs = { self, nixpkgs, flake-utils, nixhomeserver }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
        };
      in
      {
        packages = {
          mqtt-recorder = pkgs.callPackage ./nix/packages/mqtt-recorder.nix { };
          default = self.packages.${system}.mqtt-recorder;
        };

        devShells.default = pkgs.mkShell {
          packages = with pkgs; [
            cargo
            nix
            nodejs_20
            pnpm
            postgresql
            rustc
            mosquitto
          ];
        };

        formatter = pkgs.alejandra;
      })
    // {
      nixosModules = {
        mosquitto = import ./nix/modules/mosquitto.nix;
        postgres = import ./nix/modules/postgres.nix;
        mqtt-recorder = import ./nix/modules/mqtt-recorder.nix;
      };

      lib = {
        inherit nixhomeserver;
      };
    };
}
