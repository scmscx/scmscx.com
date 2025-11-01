{
  description = "scmscx.com dev env";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs = { self, nixpkgs }: 
  let
    system = "x86_64-linux";
    pkgs = import nixpkgs { inherit system; };
  in {
    devShells.${system}.default = pkgs.mkShell {
      buildInputs = with pkgs; [
        rustc
        cargo
        pkg-config
        openssl
        unzip
        cmake
        llvmPackages.libclang
        clang
        clang-tools
        glibc
        glibc.dev
        podman-compose
      ];

      # Make sure the openssl-sys crate can find the headers/libs
      shellHook = ''
        export OPENSSL_DEV="${pkgs.openssl.dev}"
        export OPENSSL_LIB_DIR="${pkgs.openssl.out}/lib"
        export OPENSSL_INCLUDE_DIR="${pkgs.openssl.dev}/include"
        export PKG_CONFIG_PATH="${pkgs.openssl.dev}/lib/pkgconfig"
        export LIBCLANG_PATH="${pkgs.llvmPackages.libclang.lib}/lib"
        export C_INCLUDE_PATH="${pkgs.glibc.dev}/include"
        export CPLUS_INCLUDE_PATH="${pkgs.glibc.dev}/include"
      '';
    };
  };
}
