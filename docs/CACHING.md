# Caching System

workflow-map uses an in-memory cache to avoid re-parsing unchanged files. This is especially useful in watch mode where the same file is checked repeatedly.

## How It Works

1. When a file is parsed, the cache stores the `Workflow` result keyed by file path + modification time.
2. On subsequent parses, the cache checks if the file's mtime has changed.
3. If unchanged, the cached `Workflow` is returned immediately (no re-parse).
4. If changed, the file is re-parseed and the cache is updated.

## Cache Key

```
(path, mtime) -> Workflow
```

The modification time (`mtime`) is obtained from the filesystem via `std::fs::metadata()`.

## Implementation

```rust
pub struct WorkflowCache {
    entries: HashMap<PathBuf, CacheEntry>,
}

pub struct CacheEntry {
    pub workflow: Workflow,
    pub modified: SystemTime,
}
```

The cache is protected by a `Mutex` for thread safety:

```rust
static CACHE: once_cell::sync::Lazy<Mutex<WorkflowCache>> =
    once_cell::sync::Lazy::new(|| Mutex::new(WorkflowCache::new()));
```

## Disabling the Cache

Use the `--no-cache` flag to bypass caching:

```
workflow-map ./workflow.yaml --no-cache
```

Or set in config:

```toml
[parsers]
disable_cache = true
```

## When Cache is Bypassed

- First parse of any file
- When `--no-cache` is set
- When `parsers.disable_cache = true` in config
- When file mtime has changed since last parse

## Watch Mode Integration

In watch mode, the cache is checked on each file change event:

1. File watcher detects change
2. Cache checks mtime
3. If mtime changed: re-parse and update cache
4. If mtime unchanged: skip (file content may have changed but mtime didn't — rare)

## Performance Impact

| Scenario | Without Cache | With Cache |
|----------|--------------|------------|
| First parse | 50ms | 50ms |
| Re-parse (unchanged) | 50ms | <1ms |
| Watch mode (10 checks) | 500ms | 50ms + 9x<1ms |

## Cache Eviction

Currently the cache grows unbounded. Future versions will add LRU eviction:

```toml
[cache]
max_entries = 100
```

## Thread Safety

The global cache uses `once_cell::sync::Lazy` with `Mutex` to ensure safe concurrent access. This means the cache works correctly even if multiple threads parse files simultaneously.
