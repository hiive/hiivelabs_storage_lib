# HiiveLabs Storage Library

A robust Rust library providing persistent storage solutions with SQLite backend, compression, and unique identifier
management for game development and data persistence needs.

## Features

- **SQLite Storage Container**: Full-featured SQLite database integration
- **Storage Container Trait**: Generic storage interface for different backends
- **Unique ID Management**: UUID-based unique identifier system
- **Data Compression**: Built-in compression support using miniz_oxide
- **Binary Serialization**: Efficient data serialization with bitcode
- **Cryptographic Hashing**: SHA-2 and hex encoding for data integrity
- **Random Utilities Integration**: Leverages hiivelabs_rand_utils_lib for enhanced functionality

## Dependencies

- `hiivelabs_rand_utils_lib` - Random utilities and helper functions
- `rusqlite` - SQLite database interface (bundled SQLite included)
- `bitcode` - Binary serialization format
- `miniz_oxide` - Pure Rust compression implementation
- `uuid` - UUID generation and management
- `sha2` - SHA-2 cryptographic hashing
- `hex` - Hexadecimal encoding/decoding
- `rand` - Random number generation
- `getrandom` - Cross-platform random generation
- `log` - Logging framework
- `env_logger` - Environment-based logging configuration

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
hiivelabs_storage_lib = { version = "0.1.0", path = "path/to/hiivelabs_storage_lib" }
```

### Basic Examples

#### SQLite Storage Container

```rust
use hiivelabs_storage_lib::prelude::*;

// Create a new SQLite storage container
let storage = SqliteStorageContainer::new("database.db").unwrap();

// Use the storage container to store and retrieve data
// (Implementation depends on your specific data structures)
```

#### Storage Container Trait

```rust
use hiivelabs_storage_lib::prelude::*;

// Implement the StorageContainer trait for custom storage backends
struct MyCustomStorage;

impl StorageContainer for MyCustomStorage {
    // Implement required methods for your storage backend
}
```

#### Unique ID Management

```rust
use hiivelabs_storage_lib::prelude::*;

// Generate unique identifiers for your data
struct MyData {
    id: uuid::Uuid,
    content: String,
}

impl UniqueId for MyData {
    fn get_id(&self) -> &uuid::Uuid {
        &self.id
    }
}
```

## Architecture

The library provides a flexible storage architecture with:

- **Storage Container Interface**: Abstract trait allowing multiple storage backends
- **SQLite Implementation**: Production-ready SQLite storage with bundled database
- **Unique ID System**: UUID-based identification for stored objects
- **Compression Support**: Automatic data compression for space efficiency
- **Logging Integration**: Comprehensive logging for debugging and monitoring

## Key Components

- **SqliteStorageContainer**: Main storage implementation using SQLite
- **StorageContainer**: Generic trait for implementing custom storage backends
- **UniqueId**: Trait for objects that can be uniquely identified
- **Integrated Utilities**: String manipulation functions from rand_utils_lib

## Features

- **Bundled SQLite**: No external SQLite installation required
- **Thread-Safe**: Designed for multi-threaded applications
- **Compression**: Built-in data compression to reduce storage footprint
- **Cryptographic Security**: SHA-2 hashing for data integrity verification

## Version

Current version: 0.1.0

## License

MIT

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

This project uses the Rust 2021 edition.