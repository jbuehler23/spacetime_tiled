# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2025-01-XX

### Added
- Initial release
- Six SpacetimeDB tables for storing Tiled map data (maps, layers, tiles, tilesets, objects, properties)
- `load_tmx_map_from_str()` - In-memory TMX parser for WASM environments
- `load_tmx_map()` - Filesystem-based loader for non-WASM environments (testing/CLI tools)
- Support for CSV-encoded tile data
- GID flip flag extraction (horizontal, vertical, diagonal)
- Custom property storage for all Tiled elements
- Complete working example with server module and interactive Rust client

### Known Issues
- Parser only supports CSV tile encoding (not base64 or other formats)
- Polygon/polyline vertex data not stored (only shape type)
- No support for tile animations
- No support for Wang sets
- Image data not stored (only image path references)
- External tileset files (.tsx) need to be embedded or sent as strings

### Notes
- Built for SpacetimeDB 1.4.0
- Uses `quick-xml` 0.36 for in-memory XML parsing
- Requires LLVM/clang toolchain for WASM compilation (zstd-sys dependency)
