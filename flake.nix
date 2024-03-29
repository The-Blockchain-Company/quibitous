{
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  inputs.flake-utils.url = "github:numtide/flake-utils";
  inputs.gitignore.url = "github:hercules-ci/gitignore.nix";
  inputs.gitignore.inputs.nixpkgs.follows = "nixpkgs";
  inputs.pre-commit-hooks.url = "github:cachix/pre-commit-hooks.nix";
  inputs.pre-commit-hooks.inputs.nixpkgs.follows = "nixpkgs";
  inputs.pre-commit-hooks.inputs.flake-utils.follows = "flake-utils";
  inputs.rust-overlay.url = "github:oxalica/rust-overlay";
  inputs.rust-overlay.inputs.flake-utils.follows = "flake-utils";
  inputs.rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
  # XXX: https://github.com/nix-community/naersk/pull/167
  #inputs.naersk.url = "github:nix-community/naersk";
  inputs.naersk.url = "github:yusdacra/naersk/feat/cargolock-git-deps";
  inputs.naersk.inputs.nixpkgs.follows = "nixpkgs";

  nixConfig.extra-substituters = [
    "https://hydra.tbco.io"
    "https://vit-ops.cachix.org"
  ];
  nixConfig.extra-trusted-public-keys = [
    "hydra.tbco.io:f/Ea+s+dFdN+3Y/G+FDgSq+a5NEWhJGzdjvKNGv0/EQ="
    "vit-ops.cachix.org-1:LY84nIKdW7g1cvhJ6LsupHmGtGcKAlUXo+l1KByoDho="
  ];

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    gitignore,
    pre-commit-hooks,
    rust-overlay,
    naersk,
  }:
    flake-utils.lib.eachSystem
    [
      flake-utils.lib.system.x86_64-linux
      flake-utils.lib.system.aarch64-linux
    ]
    (
      system: let
        readTOML = file: builtins.fromTOML (builtins.readFile file);
        workspaceCargo = readTOML ./Cargo.toml;

        pkgs = import nixpkgs {
          inherit system;
          overlays = [(import rust-overlay)];
        };

        rust = let
          _rust = pkgs.rust-bin.stable.latest.default.override {
            extensions = [
              "rust-src"
              "rust-analysis"
              "rls-preview"
              "rustfmt-preview"
              "clippy-preview"
            ];
          };
        in
          pkgs.buildEnv {
            name = _rust.name;
            inherit (_rust) meta;
            buildInputs = [pkgs.makeWrapper];
            paths = [_rust];
            pathsToLink = ["/" "/bin"];
            # XXX: This is needed because cargo and clippy commands need to
            # also be aware of other binaries in order to work properly.
            # https://github.com/cachix/pre-commit-hooks.nix/issues/126
            postBuild = ''
              for i in $out/bin/*; do
                wrapProgram "$i" --prefix PATH : "$out/bin"
              done
            '';
          };

        naersk-lib = naersk.lib."${system}".override {
          cargo = rust;
          rustc = rust;
        };

        mkPackage = name: let
          pkgCargo = readTOML ./${name}/Cargo.toml;
          cargoOptions =
            [
              "--package"
              name
            ]
            ++ (pkgs.lib.optionals (name == "quibitous") [
              "--features"
              "prometheus-metrics"
            ]);
        in
          naersk-lib.buildPackage {
            root = gitignore.lib.gitignoreSource self;

            cargoBuildOptions = x: x ++ cargoOptions;
            cargoTestOptions = x: x ++ cargoOptions;

            PROTOC = "${pkgs.protobuf}/bin/protoc";
            PROTOC_INCLUDE = "${pkgs.protobuf}/include";

            nativeBuildInputs = with pkgs; [
              pkg-config
              protobuf
              rustfmt
            ];

            buildInputs = with pkgs; [
              openssl
            ];
          };

        workspace =
          builtins.listToAttrs
          (
            builtins.map
            (name: {
              inherit name;
              value = mkPackage name;
            })
            workspaceCargo.workspace.members
          );

        quibitous-entrypoint = let
          script =
            pkgs.writeShellScriptBin "entrypoint"
            ''
              set -exuo pipefail

              ulimit -n 1024

              nodeConfig="$NOMAD_TASK_DIR/node-config.json"
              runConfig="$NOMAD_TASK_DIR/running.json"
              runYaml="$NOMAD_TASK_DIR/running.yaml"
              name="quibitous"

              chmod u+rwx -R "$NOMAD_TASK_DIR" || true

              function convert () {
                chmod u+rwx -R "$NOMAD_TASK_DIR" || true
                cp "$nodeConfig" "$runConfig"
                remarshal --if json --of yaml "$runConfig" > "$runYaml"
              }

              if [ "$RESET" = "true" ]; then
                echo "RESET is given, will start from scratch..."
                rm -rf "$STORAGE_DIR"
              elif [ -d "$STORAGE_DIR" ]; then
                echo "$STORAGE_DIR found, not restoring from backup..."
              else
                echo "$STORAGE_DIR not found, restoring backup..."

                restic restore latest \
                  --verbose=5 \
                  --no-lock \
                  --tag "$NAMESPACE" \
                  --target / \
                || echo "couldn't restore backup, continue startup procedure..."
              fi

              set +x
              echo "waiting for $REQUIRED_PEER_COUNT peers"
              until [ "$(jq -e -r '.p2p.trusted_peers | length' < "$nodeConfig" || echo 0)" -ge $REQUIRED_PEER_COUNT ]; do
                sleep 1
              done
              set -x

              convert

              if [ -n "$PRIVATE" ]; then
                echo "Running with node with secrets..."
                exec quibitous \
                  --storage "$STORAGE_DIR" \
                  --config "$NOMAD_TASK_DIR/running.yaml" \
                  --genesis-block $NOMAD_TASK_DIR/block0.bin/block0.bin \
                  --secret $NOMAD_SECRETS_DIR/bft-secret.yaml \
                  "$@" || true
              else
                echo "Running with follower node..."
                exec quibitous \
                  --storage "$STORAGE_DIR" \
                  --config "$NOMAD_TASK_DIR/running.yaml" \
                  --genesis-block $NOMAD_TASK_DIR/block0.bin/block0.bin \
                  "$@" || true
              fi
            '';
        in
          pkgs.symlinkJoin {
            name = "entrypoint";
            paths =
              [script workspace.quibitous]
              ++ (with pkgs; [
                bashInteractive
                coreutils
                curl
                diffutils
                fd
                findutils
                gnugrep
                gnused
                htop
                jq
                lsof
                netcat
                procps
                remarshal
                restic
                ripgrep
                strace
                tcpdump
                tmux
                tree
                utillinux
                vim
                yq
              ]);
          };

        pre-commit = pre-commit-hooks.lib.${system}.run {
          src = self;
          hooks = {
            alejandra = {
              enable = true;
            };
            rustfmt = {
              enable = true;
              entry = pkgs.lib.mkForce "${rust}/bin/cargo-fmt fmt -- --check --color always";
            };
          };
        };

        warnToUpdateNix = pkgs.lib.warn "Consider updating to Nix > 2.7 to remove this warning!";
      in rec {
        packages = {
          inherit (workspace) quibitous qcli;
          inherit quibitous-entrypoint;
          default = workspace.quibitous;
        };

        devShells.default = pkgs.mkShell {
          PROTOC = "${pkgs.protobuf}/bin/protoc";
          PROTOC_INCLUDE = "${pkgs.protobuf}/include";
          buildInputs =
            [rust]
            ++ (with pkgs; [
              pkg-config
              openssl
              protobuf
            ]);
          shellHook =
            pre-commit.shellHook
            + ''
              echo "=== Quibitous development shell ==="
              echo "Info: Git hooks can be installed using \`pre-commit install\`"
            '';
        };

        checks.pre-commit = pre-commit;

        hydraJobs = packages;

        defaultPackage = warnToUpdateNix packages.default;
        devShell = warnToUpdateNix devShells.default;
      }
    );
}
