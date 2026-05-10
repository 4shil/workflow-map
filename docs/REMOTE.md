# Remote Source Support

workflow-map can parse workflows directly from remote URLs. The file is downloaded and cached under:

```
~/.cache/workflow-map/remote/
```

## Usage

```
workflow-map https://raw.githubusercontent.com/org/repo/main/workflow.yaml
```

## Refresh cache

```
workflow-map https://raw.githubusercontent.com/org/repo/main/workflow.yaml --refresh
```

## Notes

- Only http/https URLs are supported.
- The cached file name is derived from the URL.
- Use `--refresh` to bypass cache.
