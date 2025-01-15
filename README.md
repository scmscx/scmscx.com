General:

- This repo uses submodules. Make sure to clone with --recurse-submodules
- Recommended plugins:
  - rust analyzer
  - prettier

Linux:

TBD.

Windows:

- Recommended plugins for vscode:
  - rust analyzer
  - prettier
  - WSL
- The line endings must be LF instead of CRLF.
  - git clone --recurse-submodules --config core.autocrlf=false git@github.com:scmscx/scmscx.com.git
- WSL2 is required to build on windows. https://learn.microsoft.com/en-us/windows/wsl/install
  - `wsl.exe --list --online`
  - `wsl.exe --install Ubuntu`
- The vscode plugin 'WSL' is very helpful.
- Install dependencies: `sudo apt-get update && sudo apt-get install build-essential git clang npm cmake pkg-config libssl-dev zip unzip python3-yaml python3-dotenv podman rustup`
- Install rust: `rustup toolchain add stable`
- Install podman-compose: `sudo curl -o /usr/local/bin/podman-compose https://raw.githubusercontent.com/containers/podman-compose/main/podman_compose.py && sudo chmod +x /usr/local/bin/podman-compose`

OSX:

- Good luck.

To one the runbox you need two terminals:

- use `make run` to run the backend + database
- use `make dev` to run the front-end
