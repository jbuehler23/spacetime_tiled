# Examples

Working examples showing how to use `spacetime_tiled` in real projects.

## Prerequisites

- [SpacetimeDB CLI](https://spacetimedb.com/install)
- Rust toolchain
- Running SpacetimeDB instance (`spacetime start`)

## simple_game

Basic server and client demonstrating map loading and querying.

**Quick start:**

```bash
cd simple_game/server
spacetime build
spacetime publish simple-game
spacetime call simple-game load_demo_map

cd ../client
spacetime generate --lang rust --out-dir src/module_bindings --project-path ../server
cargo run
```

See [simple_game/README.md](simple_game/README.md) for details.

## Workflow

1. **Create/edit TMX map** in Tiled Map Editor
2. **Build server**: `spacetime build`
3. **Publish**: `spacetime publish <module-name>`
4. **Load map data**: Call a reducer that uses `load_tmx_map_from_str()`
5. **Query data**: From reducers or clients

## WASM Constraint

SpacetimeDB modules can't access files. Two ways to load maps:

**Embedded** (recommended):
```rust
const MAP: &str = include_str!("../map.tmx");
load_tmx_map_from_str(ctx, "name", MAP)?;
```

**Client-uploaded**:
```rust
#[reducer]
pub fn upload_map(ctx: &ReducerContext, name: String, tmx: String) {
    load_tmx_map_from_str(ctx, &name, &tmx)?;
}
```

## Client Event Loop

The SpacetimeDB Rust SDK doesn't process messages automatically. You **must** start an event loop:

```rust
conn.subscription_builder()
    .on_applied(on_sub_applied)
    .subscribe(["SELECT * FROM tiled_map"]);

// Required! Without this, callbacks never fire
conn.run_threaded();
```

## Common Issues

**"No maps loaded" in client**
- Forgot `conn.run_threaded()` after subscribing

**"operation not supported on this platform"**
- Using `load_tmx_map()` in WASM - use `load_tmx_map_from_str()` instead

**"method not found" for table accessors**
- Missing trait imports like `use module_bindings::tiled_map_table::TiledMapTableAccess;`

**Build fails with clang error**
- Need LLVM/clang for WASM compilation (zstd-sys dependency)
