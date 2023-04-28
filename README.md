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

## Deploy

Push dist to gh-pages branch:
```
# Remove dist from .gitignore
git add web/dist && git commit -m "Comment"
git subtree push --prefix web/dist origin gh-pages
```
