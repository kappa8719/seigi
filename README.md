# Seigi

A collection of basic components for building Rust WebAssembly frontend projects.

## Overview

Seigi provides essential building blocks for web applications compiled to WebAssembly using Rust. The library focuses on delivering unstyled, flexible components that integrate seamlessly with your existing design system.

## Crates

### `seigi_toast`

An unstyled toast notification component that provides the core functionality without imposing any visual design. Perfect for implementing custom toast notifications that match your application's look and feel.

### `seigi_focus`

Focus management utilities including focus traps and programmatic focus control. Essential for building accessible web applications with proper keyboard navigation and focus handling.

### `seigi_form`

Comprehensive form handling with field validation and support for customizable multi-staged forms. Simplifies complex form workflows while maintaining flexibility for custom validation logic.

## Getting Started

#### Use root re-exports

```toml
[dependencies]
seigi = "0.2"
```

#### Individual crates as dependencies

```toml
[dependencies]
seigi_toast = "0.2"
seigi_focus = "0.1"
```

## Philosophy

- **Unstyled by default**: Components provide functionality without imposing visual design
- **Modular**: Use only the components you need

## License

Licensed under MIT License.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
