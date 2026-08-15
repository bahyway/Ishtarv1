{
  description = "DubSar IDE — VSCodium + Rust Jupyter kernel + Vulkan 7D particle simulation";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ rust-overlay.overlays.default ];
          config.allowUnfree = true;
        };

        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" "clippy" "rustfmt" ];
          targets = [ "x86_64-unknown-linux-gnu" ];
        };

        # Python environment for Jupyter host
        pythonEnv = pkgs.python312.withPackages (ps: with ps; [
          jupyter
          jupyterlab
          ipykernel
          ipywidgets
          pyzmq
          numpy
          matplotlib
          pillow
          traitlets
        ]);

      in {
        # ── Development shell ────────────────────────────────────────────────
        devShells.default = pkgs.mkShell {
          name = "dubsar-ide";

          buildInputs = with pkgs; [
            # Rust
            rustToolchain
            cargo-watch
            cargo-edit
            cargo-nextest

            # Vulkan (for bare-metal Fedora 44 path)
            vulkan-loader
            vulkan-headers
            vulkan-tools
            vulkan-validation-layers
            shaderc
            spirv-tools
            glslang

            # Windowing libs (Wayland + X11 fallback)
            wayland
            wayland-protocols
            libxkbcommon
            xorg.libX11
            xorg.libXcursor
            xorg.libXrandr
            xorg.libXi

            # Audio (Bevy/wgpu deps)
            alsa-lib
            pipewire

            # Python / Jupyter
            pythonEnv

            # Node.js for VSCodium extension build
            nodejs_22
            nodePackages.yarn
            nodePackages.typescript

            # VSCodium
            vscodium

            # Tools
            cmake
            ninja
            pkg-config
            gdb
            lldb
            jq
          ];

          # ── Environment variables ──────────────────────────────────────────
          shellHook = ''
            export RUST_SRC_PATH="${rustToolchain}/lib/rustlib/src/rust/library"
            export RUST_BACKTRACE=1
            export RUST_LOG="dubsar_kernel=debug,warn"

            # Vulkan (needed on bare-metal; safe to set in VM too)
            export VULKAN_SDK="${pkgs.vulkan-headers}"
            export VK_LAYER_PATH="${pkgs.vulkan-validation-layers}/share/vulkan/explicit_layer.d"
            export LD_LIBRARY_PATH="${pkgs.vulkan-loader}/lib:${pkgs.wayland}/lib:$LD_LIBRARY_PATH"

            # wgpu selects backend automatically; force software on VM
            # Comment this line out on bare-metal Fedora 44
            export WGPU_BACKEND="gl"

            echo ""
            echo "╔══════════════════════════════════════════════════╗"
            echo "║           DubSar IDE Dev Shell Ready             ║"
            echo "╠══════════════════════════════════════════════════╣"
            echo "║  cargo build          — build the kernel         ║"
            echo "║  cargo test           — run tests                ║"
            echo "║  cargo run -- --demo  — storage fragmentation    ║"
            echo "║  cargo run -- --install — install kernel spec    ║"
            echo "║  jupyter lab          — open JupyterLab          ║"
            echo "║  codium .             — open VSCodium            ║"
            echo "╚══════════════════════════════════════════════════╝"
            echo ""
          '';
        };

        # ── Bare-metal shell (Fedora 44, GTX 1650 / or better GPU) ──────────
        devShells.bare-metal = pkgs.mkShell {
          name = "dubsar-ide-bare-metal";
          buildInputs = with pkgs; [
            rustToolchain
            vulkan-loader
            vulkan-headers
            vulkan-validation-layers
            vulkan-tools
            shaderc
            spirv-tools
            pythonEnv
            nodejs_22
            nodePackages.yarn
            cmake ninja pkg-config
          ];
          shellHook = ''
            export RUST_SRC_PATH="${rustToolchain}/lib/rustlib/src/rust/library"
            export VK_LAYER_PATH="${pkgs.vulkan-validation-layers}/share/vulkan/explicit_layer.d"
            export LD_LIBRARY_PATH="${pkgs.vulkan-loader}/lib:$LD_LIBRARY_PATH"
            # No WGPU_BACKEND override — let wgpu pick Vulkan automatically
            echo "DubSar bare-metal shell ready (Vulkan backend enabled)"
            vulkaninfo --summary 2>/dev/null | head -5 || echo "vulkaninfo not found — install vulkan-tools"
          '';
        };
      });
}
