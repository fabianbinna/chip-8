# CHIP-8 Emulator
A CHIP-8 emulator written in Rust and compiled into WebAssembly.

## Install & Run

Setup the toolchain for compiling Rust programs to WebAssembly and integrate them into JavaScript: https://rustwasm.github.io/docs/book/game-of-life/setup.html

Build the project:
```
wasm-pack build
cd web
npm install
npm run start
```
