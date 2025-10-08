# rust-tig Examples

This directory contains examples demonstrating how to use various features of rust-tig.

## Configuration Demo

The `config_demo.rs` example shows how to programmatically work with the configuration system:

### Run the example

```bash
cargo run --example config_demo
```

### What it demonstrates

1. **Loading configuration**: Get the default config path and load the configuration
2. **Displaying settings**: Show all keybindings, colors, and settings
3. **Modifying configuration**: Change keybindings, colors, and settings
4. **Saving configuration**: Save modified config to a file
5. **Verifying changes**: Load the saved config to verify changes persist
6. **YAML serialization**: Display the generated YAML content
7. **Initializing defaults**: Create a default config file if it doesn't exist

### Example output

```
=== rust-tig Configuration Demo ===

Default config path: ~/.config/rust-tig/config.yaml

Loading configuration...
Configuration loaded successfully!

--- Current Configuration ---

Keybindings:
  Global:
    quit: q
    help: ?
    refresh: r
...
```

## Adding Your Own Examples

To create a new example:

1. Create a new file in this directory: `examples/your_example.rs`
2. Add a main function with your example code
3. Run it with: `cargo run --example your_example`

### Example template

```rust
use rust_tig::config::Config;
use anyhow::Result;

fn main() -> Result<()> {
    // Your example code here
    Ok(())
}
```

## See Also

- [CONFIGURATION.md](../ai/CONFIGURATION.md) - Comprehensive configuration documentation
- [config.example.yaml](../config.example.yaml) - Example configuration file
- [API documentation](https://docs.rs/rust-tig) - Full API reference
