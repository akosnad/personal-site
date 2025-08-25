{ self, lib, inputs, flake-parts-lib, ... }:

let
  inherit (flake-parts-lib)
    mkPerSystemOption;
  cargoToml = builtins.fromTOML (builtins.readFile (self + /Cargo.toml));
  inherit (cargoToml.package) name version;

  listenSocket =
    lib.match "^([0-9.]+):([0-9]+)$"
      cargoToml.package.metadata.leptos.site-addr;
  listenPort = lib.elemAt listenSocket 1;

  gitRev = self.shortRev or "dirty";
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

          site.name = lib.mkOption {
            type = lib.types.str;
            default = name;
            description = "Project name";
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
                (lib.hasInfix "/locales/" path) ||
                (lib.hasInfix "/posts/" path) ||
                (lib.hasInfix "build\.rs" path) ||
                # Default filter from crane (allow .rs files)
                (config.site.craneLib.filterCargoSources path type)
              ;
            };
          };

          site.dockerImage = lib.mkOption {
            type = lib.types.package;
            description = "Docker image of the package";
            default = (pkgs.dockerTools.buildImage {
              inherit name;
              tag = gitRev;
              config = {
                Cmd = [ (lib.getExe self'.packages.${name}) ];
              };
            }).overrideAttrs {
              __structuredAttrs = true;
              # fail build if output is larger than 128MiB
              outputChecks.out.maxSize = 128 * 1024 * 1024;
            };
          };

          site.composeProject = lib.mkOption {
            type = lib.mkOptionType {
              name = "recursivelyMergedAttrs";
              description = "Attribute set recursively merged from all definitions";

              merge = loc: defs:
                lib.foldl' lib.recursiveUpdate { } (map (x: x.value) defs);

              check = lib.isAttrs;
            };
            description = "Docker compose project";
            default = { };
          };

          site.composeProjectFile = lib.mkOption {
            type = lib.types.package;
            description = "Docker compose project output (JSON)";
            default = pkgs.writeText "docker-compose.json" (builtins.toJSON config.site.composeProject);
          };
        };
        config =
          let
            inherit (config.site) craneLib src;

            recursiveWebFont = pkgs.stdenvNoCC.mkDerivation {
              name = "recursive-woff2";
              src = pkgs.recursive;
              buildInputs = with pkgs; [ woff2 parallel ];
              buildPhase = ''
                find ./share/fonts/truetype -type f -name "*.ttf" | \
                  parallel -j "$(nproc)" -- woff2_compress {}
              '';
              installPhase = ''
                mkdir -p $out
                find . -type f -name "*.woff2" -exec cp {} $out/. \;
                # also add a stable output for the variable font
                ln -s "$out/Recursive_VF_${pkgs.recursive.version}.woff2" "$out/Recursive_VF.woff2"
              '';
              setupHook = lib.getExe (pkgs.writeScriptBin "recursive-webfont-setup" /* sh */ ''
                addFontEnvVars() {
                  if [ -e "$1/Recursive_VF.woff2" ]; then
                    export RECURSIVE_FONT_DIR="$1"
                  fi
                }
                addEnvHooks "$hostOffset" addFontEnvVars
              '');
            };

            package = craneLib.mkCargoDerivation {
              inherit src;
              pname = name;
              version = version;

              cargoArtifacts = null;
              cargoVendorDir = craneLib.vendorMultipleCargoDeps {
                cargoLockList = [
                  "${self}/Cargo.lock"
                ];
              };

              nativeBuildInputs = (with pkgs; [
                cargo-leptos
                wasm-bindgen-cli_0_2_100
                recursiveWebFont

                pkg-config
                openssl
                cargo
                rustc
                lld
                makeWrapper
              ]) ++ [
                tailwindcss
                craneLib.removeReferencesToRustToolchainHook
                craneLib.removeReferencesToVendoredSourcesHook
              ];

              strictDeps = true;
              doCheck = false;
              dontCheck = true;
              doNotPostBuildInstallCargoBinaries = true;
              doInstallCargoArtifacts = false;

              buildPhaseCargoCommand = "cargo leptos build --release -vvv";
              cargoExtraArgs = "";
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
              meta.mainProgram = name;
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
            site.composeProject.services.backend = {
              image = "${name}:${gitRev}";
              ports = [ "${toString listenPort}:${toString listenPort}" ];
            };

            # Rust package
            packages.${name} = package;
            packages."docker-${name}" = config.site.dockerImage;
            packages.composeProject = config.site.composeProjectFile;
            packages.recursiveWebFont = recursiveWebFont;

            # Rust dev environment
            devShells.${name} = pkgs.mkShell {
              inputsFrom = [
                package
              ];
              packages = (with pkgs; [
                cargo-generate
                cargo-watch
                clippy
                libiconv
                recursiveWebFont

                just
                git-lfs
              ]);
            };

            checks.${name} = package;
          };
      });
  };
}
