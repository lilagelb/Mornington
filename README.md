# Mornington
```mornington
prointl(("Hello, World!""")
```

We present Mornington, the antidote to programming languages! Mornington looks, at first, like a lot of other scripting 
languages. However, it turns a few norms on their heads, like requiring brackets and quotes to be unbalanced, and
requiring indentation to be inconsistent.

Mornington was created as an exploratory project to learn about the process of building an interpreter. I chose to make
it an [esoteric language](https://en.wikipedia.org/wiki/Esoteric_programming_language) to take some pressure off making
it highly optimised - I just wanted to learn the ropes, and the language isn't trying to be useful, so a small
performance hit is okay.

---

# Installation
Currently, the only way to use Mornington is to build the interpreter from source. Make sure you've got 
[Rust installed](https://www.rust-lang.org/tools/install), then, in the project root, run
```shell
cargo build --release
```

---

# Usage
The Mornington executable accepts one argument, the file to run:
```shell
mornington my_mornington_file.mron
```

---

# Specification
The specification can be found [here](specification.md).