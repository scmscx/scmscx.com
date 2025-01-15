General:

- This repo uses submodules. Make sure to clone with --recurse-submodules
- Recommended plugins:
  - rust analyzer
  - prettier

Linux:

- Rust needs to be installed.

Windows:

- WSL2 is required to build on windows. https://learn.microsoft.com/en-us/windows/wsl/install
- Make sure Rust is installed. https://www.rust-lang.org/tools/install
- pkg-config is needed. `sudo apt-get install pkg-config`
- openssl libraries are needed. `sudo apt-get install libssl-dev`
- `sudo apt-get install build-essential`
- Watch out for windows line endings, various libraries like zlib will fail to build with windows line endings.

OSX:

- Good luck.

To one the runbox you need two terminals:

- use `make run` to run the backend + database
- use `make dev` to run the front-end
