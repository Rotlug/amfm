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
  version = "01715bd02fd6fac5a87d5063b41279b3ac728866";

  src = fetchFromGitHub {
    owner = "Rotlug";
    repo = "amfm";
    rev = version;
    sha256 = "sha256-ZmLwfb4C+hshUmf9XzQ7eiHgggwJ2DtRSM5VyqPmLnw=";
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
