{
  description = "connect-rs";

  # Flake inputs
  inputs = {
    nixpkgs.url = "https://flakehub.com/f/NixOS/nixpkgs/0.1";
    fenix = {
      url = "https://flakehub.com/f/nix-community/fenix/0.1";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    crane.url = "https://flakehub.com/f/ipetkov/crane/0";
  };

  # Flake outputs that other flakes can use
  outputs =
    { self, ... }@inputs:
    let
      inherit (inputs.nixpkgs) lib;

      lastModifiedDate = inputs.self.lastModifiedDate or inputs.self.lastModified or "19700101";
      version = "${builtins.substring 0 8 lastModifiedDate}-${inputs.self.shortRev or "dirty"}";
      meta = (builtins.fromTOML (builtins.readFile ./protoc-gen-connect-rs-axum/Cargo.toml)).package;

      pkgsFor =
        system:
        import inputs.nixpkgs {
          inherit system;
          config = {
            allowUnfree = true;
          };
          overlays = [
            inputs.fenix.overlays.default
            inputs.self.overlays.default
          ];
        };

      # Helpers for producing system-specific outputs
      supportedSystems = [
        "aarch64-linux"
        "x86_64-linux"
        "aarch64-darwin"
      ];
      forEachSupportedSystem =
        f:
        lib.genAttrs supportedSystems (
          system:
          f {
            inherit system;
            pkgs = pkgsFor system;
          }
        );
    in
    {
      # Development environments
      devShells = forEachSupportedSystem (
        { pkgs, ... }:
        {
          default = pkgs.mkShell {
            # Pinned packages available in the environment
            packages = with pkgs; [
              rustToolchain
              cargo-edit
              cargo-machete
              bacon
              rust-analyzer
              buf
              protoc-gen-prost
              protoc-gen-prost-serde
              self.formatter.${system}
            ];

            # Environment variables
            env = {
              RUST_SRC_PATH = "${pkgs.rustToolchain}/lib/rustlib/src/rust/library";
            };
          };
        }
      );

      packages = forEachSupportedSystem (
        { pkgs, system }:
        {
          default = self.packages.${system}.protoc-gen-connect-rs-axum;
          protoc-gen-connect-rs-axum = pkgs.craneBuildPkg "protoc-gen-connect-rs-axum";
        }
      );

      dockerImages =
        let
          linuxSystem = "x86_64-linux";
          linuxPkgs = pkgsFor linuxSystem;
        in
        forEachSupportedSystem (
          { pkgs, system }:
          {
            default = self.dockerImages.${system}.protoc-gen-connect-rs-axum;

            protoc-gen-connect-rs-axum = pkgs.dockerTools.buildLayeredImage {
              inherit (meta) name;
              config = {
                Entrypoint = [
                  (lib.getExe inputs.self.packages.${linuxSystem}.protoc-gen-connect-rs-axum)
                ];
              };
            };
          }
        );

      formatter = forEachSupportedSystem ({ pkgs, ... }: pkgs.nixfmt);

      overlays.default =
        final: prev:
        let
          system = final.hostPlatform.system;
          rustToolchain =
            with inputs.fenix.packages.${system};
            combine (
              with stable;
              [
                cargo
                clippy
                rustc
                rustfmt
                rust-src
              ]
            );

          craneBuildPkg =
            pkg:
            ((inputs.crane.mkLib final).overrideToolchain rustToolchain).buildPackage ({
              pname = meta.name;
              inherit (meta) version;
              cargoExtraArgs = "-p ${pkg}";
              src = builtins.path {
                name = "${pkg}-source";
                path = self;
              };
              meta.mainProgram = meta.name;
            });
        in
        {
          inherit craneBuildPkg rustToolchain;
        };
    };
}
