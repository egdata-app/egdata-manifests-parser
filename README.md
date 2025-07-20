# Epic Games Manifest Parser

A high-performance parser for Epic Games manifest files, available as both a Rust library and a Node.js native addon.

## Features

- Parse Epic Games manifest files (`.manifest` files)
- Support for both synchronous and asynchronous operations
- Handles compressed and uncompressed manifests
- SHA-1 hash verification
- Comprehensive error handling
- High-performance native implementation
- Cross-platform support (Windows, macOS, Linux)

## Installation

### Node.js Package

```bash
npm install egdata-manifests-parser
```

### Rust Library

Add this to your `Cargo.toml`:

```toml
[dependencies]
egdata-manifests-parser = "0.1.1"
```

## Usage

### Node.js

#### Synchronous Example

```javascript
const { parseManifestSync } = require('egdata-manifests-parser');

try {
    const manifest = parseManifestSync('path/to/manifest.manifest');
    
    console.log('Manifest version:', manifest.header.version);
    if (manifest.meta) {
        console.log('App name:', manifest.meta.app_name);
        console.log('Build version:', manifest.meta.build_version);
    }
    
    console.log('Files count:', manifest.file_list?.count || 0);
    console.log('Chunks count:', manifest.chunk_list?.count || 0);
} catch (error) {
    console.error('Error parsing manifest:', error.message);
}
```

#### Asynchronous Example

```javascript
const { parseManifestAsync } = require('egdata-manifests-parser');

async function parseManifest() {
    try {
        const manifest = await parseManifestAsync('path/to/manifest.manifest');
        
        console.log('Manifest version:', manifest.header.version);
        if (manifest.meta) {
            console.log('App name:', manifest.meta.app_name);
            console.log('Build version:', manifest.meta.build_version);
        }
    } catch (error) {
        console.error('Error parsing manifest:', error.message);
    }
}

parseManifest();
```

#### Parse from Buffer

```javascript
const { parseManifestBuffer } = require('egdata-manifests-parser');
const fs = require('fs');

const buffer = fs.readFileSync('path/to/manifest.manifest');
const manifest = parseManifestBuffer(buffer);
```

### Rust Library

#### Synchronous Example

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

#### Asynchronous Example

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

## API Reference

### Node.js Functions

- `parseManifestSync(path: string): Manifest` - Parse manifest file synchronously
- `parseManifestAsync(path: string): Promise<Manifest>` - Parse manifest file asynchronously
- `parseManifestBuffer(buffer: Buffer): Manifest` - Parse manifest from buffer

### Data Structures

#### Manifest
```typescript
interface Manifest {
    header: ManifestHeader;
    meta?: ManifestMeta;
    chunk_list?: ChunkDataList;
    file_list?: FileManifestList;
}
```

#### ManifestHeader
```typescript
interface ManifestHeader {
    header_size: number;
    data_size_uncompressed: number;
    data_size_compressed: number;
    sha1_hash: string;
    stored_as: number;
    version: number;
    guid: string;
    rolling_hash: number;
    hash_type: number;
}
```

#### ManifestMeta
```typescript
interface ManifestMeta {
    data_size: number;
    data_version: number;
    feature_level: number;
    is_file_data: boolean;
    app_id: number;
    app_name: string;
    build_version: string;
    launch_exe: string;
    launch_command: string;
    prereq_ids: string[];
    prereq_name: string;
    prereq_path: string;
    prereq_args: string;
    build_id?: string;
}
```

## Development

### Building the Node.js Package

```bash
# Install dependencies
npm install

# Build for current platform
npm run build

# Build for all platforms
npm run universal
```

### Building the Rust Library

```bash
cargo build --release
```

## License

This project is licensed under the MIT License - see the LICENSE file for details.
