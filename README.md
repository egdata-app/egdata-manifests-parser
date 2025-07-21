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
npm install @egdata/manifests-parser
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
import { parseManifestSync } from '@egdata/manifests-parser';

try {
    const manifest = parseManifestSync('path/to/manifest.manifest');
    
    console.log('Manifest version:', manifest.header.version);
    if (manifest.meta) {
        console.log('App name:', manifest.meta.appName);
        console.log('Build version:', manifest.meta.buildVersion);
    }
    
    console.log('Files count:', manifest.fileList?.count || 0);
    console.log('Chunks count:', manifest.chunkList?.count || 0);
} catch (error) {
    console.error('Error parsing manifest:', error.message);
}
```

#### Asynchronous Example

```javascript
import { parseManifestAsync } from '@egdata/manifests-parser';

async function parseManifest() {
    try {
        const manifest = await parseManifestAsync('path/to/manifest.manifest');
        
        console.log('Manifest version:', manifest.header.version);
        if (manifest.meta) {
            console.log('App name:', manifest.meta.appName);
            console.log('Build version:', manifest.meta.buildVersion);
        }
    } catch (error) {
        console.error('Error parsing manifest:', error.message);
    }
}

parseManifest();
```

#### Parse from Buffer

```javascript
import { parseManifestBuffer } from '@egdata/manifests-parser';
import { readFileSync } from 'fs';

const buffer = readFileSync('path/to/manifest.manifest');
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
    chunkList?: ChunkDataList;
    fileList?: FileManifestList;
}
```

#### ManifestHeader
```typescript
interface ManifestHeader {
    headerSize: number;
    dataSizeUncompressed: number;
    dataSizeCompressed: number;
    sha1Hash: string;
    storedAs: number;
    version: number;
    guid: string;
    rollingHash: number;
    hashType: number;
}
```

#### ManifestMeta
```typescript
interface ManifestMeta {
    dataSize: number;
    dataVersion: number;
    featureLevel: number;
    isFileData: boolean;
    appId: number;
    appName: string;
    buildVersion: string;
    launchExe: string;
    launchCommand: string;
    prereqIds: string[];
    prereqName: string;
    prereqPath: string;
    prereqArgs: string;
    buildId?: string;
}
```

#### ChunkDataList
```typescript
interface ChunkDataList {
    dataSize: number;
    dataVersion: number;
    count: number;
    elements: Array<Chunk>;
    chunkLookup: Record<string, number>;
}
```

#### FileManifestList
```typescript
interface FileManifestList {
    dataSize: number;
    dataVersion: number;
    count: number;
    fileManifestList: Array<FileManifest>;
}
```

#### Chunk
```typescript
interface Chunk {
    guid: string;
    hash: string;
    shaHash: string;
    group: number;
    windowSize: number;
    fileSize: string;
}
```

#### FileManifest
```typescript
interface FileManifest {
    filename: string;
    symlinkTarget: string;
    shaHash: string;
    fileMetaFlags: number;
    installTags: Array<string>;
    chunkParts: Array<ChunkPart>;
    fileSize: number;
    mimeType: string;
}
```

#### ChunkPart
```typescript
interface ChunkPart {
    dataSize: number;
    parentGuid: string;
    offset: number;
    size: number;
    chunk?: Chunk;
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
