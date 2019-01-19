with import <nixpkgs> {};
let
  moz_overlay = import (builtins.fetchTarball https://github.com/mozilla/nixpkgs-mozilla/archive/master.tar.gz);
  nixpkgs = import <nixpkgs> { overlays = [ moz_overlay ]; };
  channel = nixpkgs.rustChannelOf { date = "2019-01-08"; channel = "nightly"; };
  #rust = nixpkgs.latest.rustChannels.nightly.rust;
  rust = (channel.rust.override { extensions = [ "rust-src" "rls-preview" ]; });
  binutils = nixpkgs.binutils;
  pkgconfig = nixpkgs.pkgconfig;
  libclang = nixpkgs.llvmPackages.libclang;
  fuse = nixpkgs.fuse;
in
  llvmPackages.stdenv.mkDerivation rec {
    name = "kafka-jq-env";
    buildInputs = [
      libclang rust binutils pkgconfig fuse
    ];
    shellHook = "export LIBCLANG_PATH=${libclang}/lib";
  }
