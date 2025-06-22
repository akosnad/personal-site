{ self, lib, inputs, flake-parts-lib, ... }:

let
  inherit (flake-parts-lib)
    mkPerSystemOption;
in
{
  options = {
    perSystem = mkPerSystemOption
      ({ config, self', inputs', pkgs, system, ... }: {
        options = {
          site.overrideCraneArgs = lib.mkOption {
            type = lib.types.functionTo lib.types.attrs;
            default = _: { };
            description = "Override crane args for the site package";
          };

          site.craneLib = lib.mkOption {
            type = lib.types.lazyAttrsOf lib.types.raw;
            default = inputs.crane.mkLib pkgs;
          };

          site.src = lib.mkOption {
            type = lib.types.path;
            description = "Source directory for the site package";
            # When filtering sources, we want to allow assets other than .rs files
            # TODO: Don't hardcode these!
            default = lib.cleanSourceWith {
              src = self; # The original, unfiltered source
              filter = path: type:
                (lib.hasSuffix "\.html" path) ||
                (lib.hasSuffix "tailwind.config.js" path) ||
                # Example of a folder for images, icons, etc
                (lib.hasInfix "/assets/" path) ||
                (lib.hasInfix "/css/" path) ||
                # Default filter from crane (allow .rs files)
                (config.site.craneLib.filterCargoSources path type)
              ;
            };
          };
        };
        config =
          let
            cargoToml = builtins.fromTOML (builtins.readFile (self + /Cargo.toml));
            inherit (cargoToml.package) name version;
            inherit (config.site) craneLib src;

            # Crane builder for cargo-leptos projects
            craneBuild = rec {
              args = {
                inherit src;
                pname = name;
                version = version;
                buildInputs = with pkgs; [
                  cargo-leptos
                  wasm-bindgen-cli_0_2_100
                ] ++ [
                  tailwindcss
                ];
                nativeBuildInputs = with pkgs; [
                  pkg-config
                  openssl
                  cargo
                  rustc
                  lld
                ];
              };
              cargoArtifacts = craneLib.buildDepsOnly args;
              buildArgs = args // {
                inherit cargoArtifacts;
                buildPhaseCargoCommand = "cargo leptos build --release -vvv";
                cargoTestCommand = "cargo leptos test --release -vvv";
                cargoExtraArgs = "";
                doCheck = false;
                nativeBuildInputs = [
                  pkgs.makeWrapper
                ] ++ args.nativeBuildInputs;
                doNotPostBuildInstallCargoBinaries = true;
                installPhaseCommand = ''
                  mkdir -p $out/bin
                  cp target/release/${name} $out/bin/
                  cp -r target/site $out/bin/

                  find $out/bin -type f -exec wasm-opt -Oz -g '{}' \; -exec strip -s -v '{}' \; 2>/dev/null

                  patchelf --shrink-rpath \
                    $out/bin/${name}

                  wrapProgram $out/bin/${name} \
                    --set LEPTOS_SITE_ROOT $out/bin/site \
                    --set LEPTOS_ENV PROD \
                    --set LEPTOS_SITE_ADDR "0.0.0.0:3000"
                '';
                # postFixup = ''
                #   echo "Patching out references to the Rust toolchain..."
                #   __faketoolchain="$(echo '${rustToolchain}' | sed -E 's/[a-z0-9]{32}/eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee/')"
                #   find $out/bin -type f -exec sed -i "s;${rustToolchain};$__faketoolchain;g" '{}' \;
                # '';
                meta.mainProgram = name;
              };
              package = craneLib.buildPackage (buildArgs // config.site.overrideCraneArgs buildArgs);
            };

            rustDevShell = pkgs.mkShell {
              # shellHook = ''
              #   # For rust-analyzer 'hover' tooltips to work.
              #   export RUST_SRC_PATH="${rustToolchain}/lib/rustlib/src/rust/library";
              # '';
              inputsFrom = [
                craneBuild.package
              ];
              packages = [
                pkgs.libiconv
              ] ++ craneBuild.args.nativeBuildInputs;
            };

            tailwindcss = pkgs.nodePackages.tailwindcss.overrideAttrs
              (oa: {
                plugins = [
                  pkgs.nodePackages."@tailwindcss/aspect-ratio"
                  pkgs.nodePackages."@tailwindcss/forms"
                  pkgs.nodePackages."@tailwindcss/language-server"
                  pkgs.nodePackages."@tailwindcss/line-clamp"
                  pkgs.nodePackages."@tailwindcss/typography"
                ];
              });
          in
          {
            # Rust package
            packages.${name} = craneBuild.package;

            # Rust dev environment
            devShells.${name} = pkgs.mkShell {
              inputsFrom = [
                rustDevShell
              ];
              nativeBuildInputs = with pkgs; [
                tailwindcss
                cargo-leptos
                binaryen # Provides wasm-opt
              ];
            };
          };
      });
  };
}
