# strseq: Immutable String Segmentation in Rust

`strseq` is a Rust library designed for handling multiple immutable string segments efficiently. It offers a lean approach to represent string hierarchies without constant delimiter checks or allocating distinct strings for individual tokens.

## Features

- **Compact Storage with `StringSequence`**: 
  - Stores multiple strings in a single linear buffer.
  - Span information (begin, end) is located at the buffer's front.
  - Actual string content fills the rest of the buffer.

- **Shared Sequences with `SharedStringSequence`**: 
  - Functions similarly to `StringSequence`.
  - Reference-counted, allowing cheap cloning and sharing across contexts.

- **Mutable Operations with `MutableStringSequence`**:
  - Contains two distinct dynamic buffers: one for indices and another for text.
  - Supports efficient mutations like pop, push, and insert.

## Use Cases

- Hierarchical path representation.
- Token-based systems where separate string allocation for each token is inefficient.

## Getting Started

In your `Cargo.toml`, add following:

```
strseq = "0.1.0"
```

## Features

- `serde`: Enables serialization and deserialization of `StringSequence` and `SharedStringSequence`, `MutableStringSequence` using [Serde](https://serde.rs/). 


## Serde representation

All defined structs are represented as list of strings.

## Feedback

We welcome contributions, feedback, and issues on our [GitHub repository](https://github.com/kang-sw/strseq-rs).
