{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    crane.url = "github:ipetkov/crane/v0.20.3";
    treefmt-nix.url = "github:numtide/treefmt-nix";
  };

  outputs = inputs: inputs.flake-parts.lib.mkFlake { inherit inputs; } {
    systems = [ "x86_64-linux" ];
    imports = [
      inputs.treefmt-nix.flakeModule
      ./nix/flake-module.nix
    ];
    perSystem = { config, system, self', pkgs, lib, ... }: {
      treefmt.config = {
        projectRootFile = "flake.nix";
        programs = {
          nixpkgs-fmt.enable = true;
          rustfmt.enable = true;
          leptosfmt.enable = true;
          taplo.enable = true;
          prettier.enable = true;
        };
      };

      packages.default = self'.packages.personal-site;
      packages.docker = pkgs.dockerTools.buildImage {
        name = "personal-site";
        config = {
          Cmd = [ (lib.getExe self'.packages.personal-site) ];
        };
      };

      devShells.default = pkgs.mkShell {
        inputsFrom = [
          config.treefmt.build.devShell
          self'.devShells.personal-site
        ];
        packages = with pkgs; [
          just
          cargo-watch
          git-lfs
        ];
      };
    };
  };
}
