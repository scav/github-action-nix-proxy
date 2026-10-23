{
  description = "A simple forward proxy for github actions";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-26.05";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      nixpkgs,
      fenix,
      ...
    }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "aarch64-darwin"
      ];
      forAllSystems = f: nixpkgs.lib.genAttrs systems (system: f system);
    in
    {
      packages = forAllSystems (
        system:
        let
          pkgs = import nixpkgs { inherit system; };
          toolchain = fenix.packages.${system}.minimal.toolchain;

          github-action-nix-proxy =
            (pkgs.makeRustPlatform {
              cargo = toolchain;
              rustc = toolchain;
            }).buildRustPackage
              {

                pname = "github-action-nix-proxy";
                version = "0.0.1";
                src = ./.;
                cargoLock.lockFile = ./Cargo.lock;
                nativeBuildInputs = [
                  pkgs.git
                  toolchain
                ];

                doCheck = true;
                strip = true;

                # preBuild = ''
                # '';
              };
        in
        {
          default = github-action-nix-proxy;

          docker = pkgs.dockerTools.buildLayeredImage {
            name = "github-action-nix-proxy";
            tag = "latest";

            contents = [
              github-action-nix-proxy
              pkgs.cacert
              pkgs.iana-etc
            ];

            config = {
              Cmd = [ "bin/github-action-nix-proxy" ];
              Env = [
                "RUST_LOG=info"
                "TERM=xterm-256color"
                "SSL_CERT_FILE=/etc/ssl/certs/ca-bundle.crt"
              ];
            };
          };
        }
      );

      devShells = forAllSystems (
        system:
        let
          pkgs = import nixpkgs { inherit system; };
          arch = pkgs.stdenv.hostPlatform.parsed.cpu.name;
          toolchain = fenix.packages.${system}.combine [
            fenix.packages.${system}.stable.toolchain
            fenix.packages.${system}.stable.rust-src
          ];

          bi = pkgs.writeShellScriptBin "bi" ''
            echo "🔨 building image..."
            nix build .#packages.${arch}-linux.docker
            echo "🐳 loading image..."
            docker load < result
          '';
        in
        {
          default = pkgs.mkShell {
            buildInputs = [
              toolchain
            ];
            nativeBuildInputs = [
            ];
            packages = [
              pkgs.bacon
              pkgs.cargo-audit
              pkgs.cargo-deny
              pkgs.cargo-nextest
              pkgs.pkg-config
              bi
            ];
            NIXFWD_URL = "https://example.org";
          };
        }
      );

    };
}
