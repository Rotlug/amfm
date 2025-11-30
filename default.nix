{
  pkg-config,
  fetchFromGitHub,
  rustPlatform,
  gst_all_1,
  openssl,
  glib,
  makeWrapper,
  ...
}:
rustPlatform.buildRustPackage rec {
  pname = "amfm";
  version = "9143135db95758726111766fc9e59689632187cb";

  src = fetchFromGitHub {
    owner = "Rotlug";
    repo = "amfm";
    rev = version;
    sha256 = "sha256-TG6besY1xrdc8XhjP9r7OkC8+yb+LnSbS8YAoNHCr1g=";
  };

  cargoBuildFlags = ["-p" "amfm"];

  cargoHash = "sha256-WyCIOhi7axe6bbDPfSs0nyna6cDxLmbFInkNMGI2iaw=";

  nativeBuildInputs = [
    pkg-config
    makeWrapper
  ];

  buildInputs =
    [
      openssl.dev
      glib.dev
    ]
    ++ (with gst_all_1; [
      gstreamer.dev
    ]);

  propagatedBuildInputs = with gst_all_1; [
    gstreamer
    gst-plugins-base
    gst-plugins-good
    gst-plugins-bad
  ];

  postInstall = ''
    wrapProgram $out/bin/amfm \
      --set GST_PLUGIN_SYSTEM_PATH_1_0 "${gst_all_1.gstreamer.out}/lib/gstreamer-1.0:${gst_all_1.gst-plugins-base.out}/lib/gstreamer-1.0:${gst_all_1.gst-plugins-good.out}/lib/gstreamer-1.0:${gst_all_1.gst-plugins-bad.out}/lib/gstreamer-1.0"
  '';
}
