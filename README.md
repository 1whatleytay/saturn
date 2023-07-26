# Saturn - Modern MIPS Environment

Saturn is a development environment for educational learning with the MIPS processor architecture.

Saturn contains a custom-made editor, interpreter, debugger and assembler for MIPS code in order to deliver a solid and stable experience throughout.

![Saturn Early Development Screenshot](README.png)

# Project Goals

- **Performance and Stability** - A fast environment that can keep up with all your use cases.
- **Easy-to-use Tools** - Quick and simple interfaces that provide what you need at a glance.
- **In-Place Debugging** - Set breakpoints and read values in-line with your source.

# Building

Saturn is built with Tauri and has a Typescript and Rust component.
To build, please install the following tools:

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

# Required Tasks

- [x] Basic Editor
- [x] Basic Interpreter
- [x] Basic Debugger
- [x] Basic Assembler
- [x] Syntax Highlighting
- [x] Bitmap Display
- [x] Console Events
- [x] Syscall Handling
- [x] Assembler Breakpoint Information
- [x] Compile Assembly In-Editor
- [x] Breakpoint Assembly In-Editor
- [x] Keyboard Input
- [x] File Loading

# Important Tasks

- [x] Memory Editing and Copying
- [x] Register Editing and Copying
- [x] Finding Text (Cmd + F)
- [x] Finished Execution State
- [x] Editor Performance (Smooth Cmd + A @ 4000 lines)
- [ ] Floating Point Co-processor
- [x] MIDI and Other Syscalls
- [x] Variable Name Suggestions
- [x] Improve Bitmap Display
- [x] Add Time Travel
