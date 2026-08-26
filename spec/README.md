# QuickPython specifications

> **Status: current documentation index.** This page distinguishes checked-in
> behavior from proposals and external references. The maintained CPython
> compatibility contract is [`docs/limitations/`](../docs/limitations/README.md).

QuickPython is an embeddable Python-like language implemented as a Rust
stack-machine VM. It intentionally supports a subset of Python and does not
claim bytecode, runtime, or standard-library compatibility with CPython.

## Current implementation at a glance

- **Parsing and compilation:** `rustpython_parser` AST nodes are compiled into
  `ByteCode = Vec<Instruction>`. Each instruction is a Rust enum variant; there
  is no compact variable-width in-memory encoding.
- **Values:** [`Value`](../src/value.rs) is a Rust enum. Scalars are stored in
  enum payloads, strings are owned `String` values, and shared mutable
  containers use `Rc<RefCell<...>>`. QuickPython does not currently use NaN
  boxing, tagged pointers, or string interning.
- **Memory management:** Rust ownership manages values. `Rc` reference counts
  shared runtime objects, but there is no Python-style cycle detector or
  tracing garbage collector.
- **Execution:** [`VM`](../src/vm.rs) is a stack interpreter. Async syntax is
  accepted, but awaiting a coroutine runs it sequentially; `asyncio.sleep`
  blocks on a newly created Tokio runtime rather than participating in a task
  scheduler.
- **Objects:** built-in runtime variants, modules, regex matches, and a small
  type enum exist. User-defined classes, instances, slots, descriptors, and
  dynamic attributes are not implemented.
- **Serialization:** `.pyq` has a small private versioned format. The serializer
  supports only selected scalar, arithmetic, variable, jump, conversion, and
  iteration instructions; it rejects functions, collections, exceptions,
  imports, async/generator instructions, and other families. See the
  [serialization contract](../docs/limitations/serialization.md).

## Specification index

The status in this table is repeated inside every indexed document. “Proposal”
and “external reference” documents are not statements of current behavior.

| Document | Status | Purpose |
| --- | --- | --- |
| [Architecture](architecture.md) | **Current behavior** | Grounded overview of the checked-in runtime |
| [Language specification](language-spec.md) | **Historical checklist; non-current** | Early feature plan; tests and limitations are authoritative |
| [API design](api-design.md) | **Proposal; non-current** | Aspirational embedding API |
| [File formats](file-formats.md) | **Proposal; non-current** | Aspirational `.pyq` layout |
| [CLI design](cli-design.md) | **Proposal; non-current** | Commands beyond the checked-in CLI |
| [Async/await](async-await.md) | **Proposal; non-current** | Aspirational async runtime design |
| [Comparison operators fix](comparison-operators-fix.md) | **Historical implementation note** | Record of earlier comparison work |
| [Exception system](exception-system.md) | **Historical implementation note** | Earlier exception design and rollout notes |
| [Generators](generators.md) | **Historical implementation note** | Earlier generator design and rollout notes |
| [Import system](import-system.md) | **Historical implementation note** | Earlier import design and rollout notes |
| [Module examples](module-examples.md) | **Example reference; non-contractual** | Usage sketches, not a compatibility promise |
| [QuickJS bytecode analysis](quickjs-bytecode-analysis.md) | **External design reference** | Research notes; not QuickPython behavior |
| [QuickJS design reference](quickjs-design-reference.md) | **External design reference** | Research notes; not QuickPython behavior |

## Design goals versus current behavior

Small startup cost, a compact runtime, richer Rust interoperation, native async
scheduling, interned strings, compact bytecode, and optimized object layouts may
remain useful goals. They are not implemented merely because an older design
document describes them. Any future implementation must update the architecture
and the appropriate limitations page together.
