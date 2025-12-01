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
  version = "cd5674c8727f72d3c16701546cd05b854cb0092c";

  src = fetchFromGitHub {
    owner = "Rotlug";
    repo = "amfm";
    rev = version;
    sha256 = "sha256-ghHSweuyjZq8dfoNJUJE+S5THO236CzKXPF5MumjGps=";
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
