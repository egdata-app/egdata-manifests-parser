# Epic Games Manifest Parser

A Rust library for parsing Epic Games manifest files. This library provides both synchronous and asynchronous interfaces for reading and parsing manifest files used by Epic Games.

## Features

- Parse Epic Games manifest files
- Support for both synchronous and asynchronous operations
- Handles compressed and uncompressed manifests
- SHA-1 hash verification
- Comprehensive error handling

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
egdata-manifests-parser = "0.1.0"
```

### Synchronous Example

```rust
use egdata_manifests_parser::Manifest;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manifest = Manifest::load("path/to/manifest.manifest")?;

    println!("Manifest version: {}", manifest.header.version);
    if let Some(meta) = &manifest.meta {
        println!("App name: {}", meta.app_name);
        println!("Build version: {}", meta.build_version);
    }

    Ok(())
}
```

### Asynchronous Example

```rust
use egdata_manifests_parser::Manifest;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manifest = Manifest::load_async("path/to/manifest.manifest").await?;

    println!("Manifest version: {}", manifest.header.version);
    if let Some(meta) = &manifest.meta {
        println!("App name: {}", meta.app_name);
        println!("Build version: {}", meta.build_version);
    }

    Ok(())
}
```

## License

This project is licensed under the MIT License - see the LICENSE file for details.
