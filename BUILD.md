# Building Saturn

Saturn uses Tauri and has a Typescript frontend and Rust backend.
For contribution purposes, you can contribute to the app by building Saturn locally.

First, please install the following tools:

- [Node](https://nodejs.org/en)
- [Rust](https://www.rust-lang.org)
- [Yarn (Classic)](https://yarnpkg.com)

Before development, install dependencies using
```
yarn install
```

To run for development, use
```shell
yarn tauri dev
```

To build a binary for your platform, use
```shell
yarn tauri build
```
