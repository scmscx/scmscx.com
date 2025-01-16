<div align="center">
<h1>scmscx.com</h1>
  <a href="https://github.com/scmscx/scmscx.com">
    <img src="app/assets/favicon.svg" alt="Logo" width="80" height="80">
  </a>
</div>

## Getting Started

### Build Instructions

#### Linux:

- Recommended plugins for vscode:
  - rust analyzer
  - prettier
  - WSL
- Clone the repo
  - git clone --recurse-submodules git@github.com:scmscx/scmscx.com.git
- Install dependencies from your package manager:
  - Install dependencies: `git clang npm cmake pkg-config libssl-dev zip unzip python3-yaml python3-dotenv podman rustup`
- Install rust
- Install podman-compose, a relatively recent version since many bugs have been fixed as of 2025-01-02

#### Windows:

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

### Running the website locally

- use `make run` to run the backend + database
- use `make dev` to run the front-end

Then you can view the website by going to http://localhost:8080
