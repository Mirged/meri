# Meri - Simple Assembly-like Language Emulator

Meri is a simple emulator for an assembly-like language written in Rust. It provides a basic virtual CPU with instructions for moving and manipulating registers.

## Getting Started

These instructions will help you set up and run the Meri emulator on your local machine.

### Prerequisites

Ensure that you have Rust installed on your system. If not, you can install it from [https://www.rust-lang.org/tools/install](https://www.rust-lang.org/tools/install).

### Building the Project

Clone the repository to your local machine:

```bash
git clone https://github.com/Mirged/meri.git
cd meri
```

Build the project using Cargo:

```bash
cargo build
```

### Running the Emulator

Run the emulator with a Meri assembly file:

```bash
cargo run -- <path-to-your-assembly-file>
```

For example:

```bash
cargo run -- examples/sample_program.meri --print-state
```

### Options

- `--print-state`: Print CPU state after program execution.

## Writing Meri Assembly Code

Write your Meri assembly code in a text file with the `.meri` extension. Example:

```assembly
MovRegReg 0 1;
AddRegReg 2 3;
JmpAddr 5;
HLT;
```

## Contributing

If you have any improvements or bug fixes, feel free to open an issue or submit a pull request. See the [CONTRIBUTING.md](CONTRIBUTING.md) file for details

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
