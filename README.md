<div align="center">

# QuickPython

**A lightweight Python bytecode VM written in Rust.**

[![License: MPL 2.0](https://img.shields.io/badge/license-MPL--2.0-blue.svg)](LICENSE)
[![CI](https://github.com/VikingMew/quickpython/actions/workflows/ci.yml/badge.svg)](https://github.com/VikingMew/quickpython/actions/workflows/ci.yml)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org/)

> **Alpha** — This project is under active development. APIs may change, features may be incomplete, and bugs are expected. Contributions and feedback are welcome.

</div>

---

QuickPython compiles Python source code to custom bytecode and executes it on a stack-based virtual machine. It focuses on being small, fast to start, and easy to extend — not on full CPython compatibility.

## Features

- **Bytecode compiler & VM** — compiles Python to custom bytecode, executes on a stack-based VM
- **Core Python subset** — variables, functions, control flow, exceptions, lists, dicts, strings, floats
- **Built-in modules** — `json`, `os`, `re`
- **Extension system** — register Rust-native modules as Python imports
- **Bytecode serialization** — compile to `.pyq` files for faster loading
- **Iterator safety** — detects list modification during iteration

## Getting Started

### Install from source

```bash
git clone https://github.com/VikingMew/quickpython.git
cd quickpython
cargo build --release
```

The binary is at `target/release/quickpython`.

### Quick example

```python
import json

def fibonacci(n):
    if n <= 1:
        return n
    return fibonacci(n - 1) + fibonacci(n - 2)

results = []
for i in range(10):
    results.append(fibonacci(i))

print("Fibonacci sequence:")
print(json.dumps(results))
```

```bash
$ quickpython run fib.py
Fibonacci sequence:
[0, 1, 1, 2, 3, 5, 8, 13, 21, 34]
```

## Usage

```bash
quickpython run script.py              # run a Python file
quickpython run script.pyq             # run compiled bytecode
quickpython compile script.py          # compile to script.pyq
quickpython compile script.py -o out.pyq
```

## Supported Python Features

| Category | Features |
|---|---|
| **Types** | `int`, `float`, `bool`, `None`, `str`, `list`, `dict` |
| **Operators** | `+` `-` `*` `/`, `==` `!=` `<` `<=` `>` `>=` |
| **Control flow** | `if`/`elif`/`else`, `while`, `for`/`in`, `break`, `continue`, `pass` |
| **Functions** | `def`, `return`, recursion, local scopes |
| **Exceptions** | `raise`, `try`/`except`/`finally`, 10 exception types |
| **Built-ins** | `print()`, `len()`, `int()`, `float()`, `range()` |
| **Collections** | list indexing/`append`/`pop`, dict indexing/`keys()` |
| **Modules** | `import json`, `import os`, `import re` |

## Embed in Rust

QuickPython is a library. Add it as a dependency and evaluate Python from Rust:

```rust
use quickpython::Context;

fn main() {
    let mut ctx = Context::new();
    ctx.set("x", quickpython::Value::Int(42));
    ctx.eval("print(x + 1)").unwrap();
}
```

### Extension modules

Create a separate crate to add custom Python modules:

```rust
use quickpython::{Module, Value, ExceptionType, register_extension_module};

pub fn init() {
    register_extension_module("mymod", || {
        let mut m = Module::new("mymod");
        m.add_function("hello", |args| {
            Ok(Value::String("hello from rust".to_string()))
        });
        m
    });
}
```

Then from Python:

```python
import mymod
print(mymod.hello())
```

See [`quickpython-llm/`](quickpython-llm/) for a full example that wraps OpenAI-compatible APIs as a Python module.

## Comparison with other runtimes

### vs [RustPython](https://github.com/RustPython/RustPython)

RustPython targets near-complete Python 3.14 compatibility — classes, generators, decorators, pip install, the works. The trade-off is a multi-MB binary, slower startup, and complexity. QuickPython goes the opposite direction: a minimal core that covers the 80% of Python you actually need for scripting (functions, loops, exceptions, dicts/lists). If your use case is "run user scripts that call into Rust modules", you don't need a full Python runtime. QuickPython gives you a small binary, fast startup, and a straightforward Rust extension API without dragging in the entire Python ecosystem.

### vs [Monty](https://github.com/pydantic/monty)

Monty (by the Pydantic team) is built for a specific scenario: sandboxing LLM-generated Python code. It offers snapshot/resume of interpreter state and strict security isolation — no filesystem or network access unless explicitly granted. QuickPython is a general-purpose embeddable VM, not a sandbox. It provides built-in modules (`os`, `json`, `re`) and an extension system designed for adding capabilities, not restricting them. Monty is also experimental (v0.0.3) with no class support yet. If you're building an AI agent runtime, look at Monty. If you're embedding Python scripting in a Rust application, QuickPython is a better fit.

### vs [MicroPython](https://github.com/micropython/micropython)

MicroPython is the mature choice for running Python on microcontrollers — 256KB flash, 16KB RAM, battle-tested on ESP32/RP2040/STM32. It's written in C with a C extension API. QuickPython is Rust-native: if your host application is Rust, extending QuickPython means writing Rust functions, not C. MicroPython has broader Python compatibility (3.4 core + async/await) and a huge hardware ecosystem, but it's not designed for Rust integration. QuickPython is smaller in scope but fits naturally into Rust projects with zero C interop overhead.

### vs [QuickJS](https://github.com/niclas-niclas/quickjs)

QuickJS is the gold standard for tiny embeddable scripting engines — 367KB binary, near-complete ES2023, created by Fabrice Bellard. The obvious difference: it runs JavaScript, not Python. If your users or scripts are Python-based, QuickJS isn't an option. QuickPython follows a similar philosophy — small, embeddable, no JIT, bytecode VM — but for the Python language. QuickJS is more mature and has broader language coverage, but QuickPython offers Python syntax which is more accessible for non-developer users writing configuration scripts or automation.

### vs [mlua](https://github.com/mlua-rs/mlua)

mlua provides excellent Rust bindings to Lua (5.1-5.5, LuaJIT, Luau). Lua is faster than Python (especially with LuaJIT), has a smaller footprint (~293KB), and is proven in games, Redis, nginx, and Neovim. The trade-off is language familiarity: Python is far more widely known. If your users are writing scripts, Python syntax has a lower learning curve than Lua's 1-indexed tables and `local` scoping. QuickPython lets you offer Python as the scripting language while keeping the Rust-native extension story. If raw performance matters more than language choice, mlua + LuaJIT is hard to beat.

### vs [Starlark](https://github.com/facebook/starlark-rust)

Starlark is a Python dialect designed by Google for build system configuration (Bazel, Buck2). It's deterministic and hermetic — same input always gives same output, no filesystem or network access. The Rust implementation (`starlark-rust`) by Meta is mature and well-tested with LSP and DAP support. But Starlark is intentionally restricted: no `while`, `try`, `raise`, `class`, `import`, or `yield`. It's a configuration language, not a scripting language. QuickPython supports all of these — loops, exceptions, modules, recursion — because it's designed for general-purpose scripting, not locked-down config evaluation. If you need deterministic hermetic evaluation, Starlark is purpose-built for that. If you need users to write actual programs, QuickPython gives them the Python features they expect.

### Summary

| | QuickPython | RustPython | Monty | MicroPython | QuickJS | mlua | Starlark |
|---|---|---|---|---|---|---|---|
| **Written in** | Rust | Rust | Rust | C | C | Rust + C | Rust |
| **Scripting lang** | Python subset | Python 3.14 | Python subset | Python 3.4+ | JS ES2023 | Lua 5.x | Python dialect |
| **Focus** | Embed in Rust | Full Python | AI sandbox | MCU/embedded | Minimal JS | Lua in Rust | Build config |
| **Binary size** | Small | Multi-MB | ~4.5MB | ~256KB | ~367KB | ~293KB | Small |
| **Python compat** | Core subset | Near-complete | Partial | 3.4 core | N/A | N/A | Restricted subset |
| **Extensions** | Rust modules | Rust + pip | External fns | C modules | C API | Rust UserData | Rust types |
| **Maturity** | Alpha | Mature | Experimental | Mature | Mature | Mature | Mature |

## Contributing

QuickPython is in alpha. Bug reports, feature requests, and PRs are welcome. Please open an issue before submitting large changes.

## License

MPL-2.0
