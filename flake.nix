{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };
  outputs = {
    self,
    nixpkgs,
    flake-utils,
  }:
    flake-utils.lib.eachDefaultSystem
    (
      system: let
        pkgs = import nixpkgs {
          inherit system;
        };
      in
        with pkgs; {
          devShells.default = mkShell {
            buildInputs = with pkgs; [
              rustc
              cargo
              rustlings
              clippy

              cargo-generate

              pkg-config
              openssl
              gst_all_1.gstreamer
              gst_all_1.gst-plugins-base
              gst_all_1.gst-plugins-good
              gst_all_1.gst-plugins-bad
              gst_all_1.gst-plugins-ugly
            ];
          };

          packages.default = let
            manifest = (pkgs.lib.importTOML ./amfm/Cargo.toml).package;
          in
            pkgs.rustPlatform.buildRustPackage {
              pname = manifest.name;
              version = manifest.version;

              cargoLock.lockFile = ./Cargo.lock;

              src = ./.;

              nativeBuildInputs = with pkgs; [
                pkg-config
                makeWrapper
              ];

              buildInputs =
                [
                  openssl
                  glib
                ]
                ++ (with gst_all_1; [
                  gstreamer
                  gst-plugins-base
                  gst-plugins-good
                  gst-plugins-bad
                ]);

              postInstall = ''
                wrapProgram $out/bin/${manifest.name} \
                  --set GST_PLUGIN_SYSTEM_PATH_1_0 "${gst_all_1.gstreamer.out}/lib/gstreamer-1.0:${gst_all_1.gst-plugins-base.out}/lib/gstreamer-1.0:${gst_all_1.gst-plugins-good.out}/lib/gstreamer-1.0:${gst_all_1.gst-plugins-bad.out}/lib/gstreamer-1.0"
              '';
            };
        }
    );
}
