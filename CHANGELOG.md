# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.1] - 2026-02-15

### Added

#### Operators (Tasks 027-029)
- **Augmented assignment operators** (`+=`, `-=`, `*=`, `/=`, `%=`) - Task 027
- **Logical operators** with short-circuit evaluation (`and`, `or`, `not`) - Task 028
- **Membership operator** (`in`) for strings, lists, dicts, and tuples - Task 029

#### String Operations (Task 030)
- String methods: `split()`, `strip()`, `startswith()`, `endswith()`
- String methods: `lower()`, `upper()`, `replace()`, `join()`
- Method chaining support

#### Data Structures (Tasks 031-032, 035)
- **Dictionary `.get()` method** with optional default value - Task 031
- **Multiple assignment and tuple unpacking** - Task 032
  - Unpack tuples: `a, b, c = (1, 2, 3)`
  - Unpack lists: `x, y = [10, 20]`
  - Variable swapping: `a, b = b, a`
- **Slicing support** for lists, strings, and tuples - Task 035
  - Basic slicing: `items[1:3]`
  - Negative indices: `items[-2:]`
  - Step parameter: `items[::2]`
  - Slice objects: `s = slice(1, 5, 2)`

#### String Formatting (Tasks 033-034)
- **F-string formatting** - Task 033
  - Basic interpolation: `f"Hello {name}"`
  - Expression evaluation: `f"Result: {x + y}"`
  - Nested expressions
- **`str()` builtin function** - Task 034
  - Convert any value to string representation

#### Type System (Task 036)
- **`isinstance()` builtin function** - Task 036
- Type objects: `int`, `float`, `bool`, `str`, `list`, `dict`, `tuple`, `NoneType`
- Runtime type checking

#### List Comprehensions (Task 037)
- **List comprehensions** with full syntax support - Task 037
  - Basic: `[x*2 for x in range(5)]`
  - With filter: `[x for x in items if x > 5]`
  - Complex expressions: `[x*2 + 1 for x in range(10) if x % 2 == 0]`
- **String iteration** support for loops and comprehensions - Task 039

#### Async/Await Support (Task 038)
- **Complete async/await implementation** with Tokio integration
  - `async def` function definitions
  - `await` expressions for coroutines
  - Coroutine objects (lazy evaluation)
- **asyncio builtin module**
  - `asyncio.sleep(seconds)` - True asynchronous sleep using Tokio runtime
  - Proper argument validation (non-negative numbers only)
- **Tokio runtime integration**
  - Real async I/O operations
  - Maintains synchronous API for backward compatibility
- Examples: `examples/async_sleep.py`

### Fixed
- **Test failures** caused by type object keywords - Task 039
  - Fixed tests using `list` and `dict` as variable names
  - Renamed to `my_list` and `my_dict` to avoid conflicts

### Changed
- List and tuple equality now compares content instead of pointer equality
- Enhanced Value enum with new variants: `Tuple`, `Slice`, `Type`, `BoundMethod`, `Coroutine`, `AsyncSleep`

### Tests
- Added 109 tests for Tasks 027-037
- Added 15 async/await tests (8 basic + 7 asyncio.sleep)
- **Total: 254 tests passing** (up from 128)

### Performance
- Short-circuit evaluation for logical operators (avoids unnecessary computation)

### Documentation
- Complete task documentation for Tasks 027-039
- Examples for all new features
- Async/await usage guide with timing demonstrations

## [0.1.0] - 2025-01-XX

Initial release with core Python features:
- Basic expressions and variables
- Functions and control flow
- Strings and print function
- Float support
- Lists and dictionaries
- For loops with break/continue
- Exception system (types, raise, try/except/finally)
- Import system (json, os, re modules)
- Comparison operators
- CLI tool (pyq)

[0.1.1]: https://github.com/VikingMew/quickpython/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/VikingMew/quickpython/releases/tag/v0.1.0
