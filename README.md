# duplff

Find and remove duplicate files.

```
Usage: duplff [OPTIONS] <PATHS>...

Arguments:
  <PATHS>...  Directories to scan for duplicates

Options:
  -e, --ext <EXTENSIONS>     File extensions to include (e.g. py rs js)
  -m, --min-size <MIN_SIZE>  Minimum file size in bytes (default: 1)
  -M, --max-size <MAX_SIZE>  Maximum file size in bytes
  -p, --priority <PRIORITY>  Priority directories (files here are preferred to keep)
  -x, --exclude <EXCLUDE>    Exclude directories/patterns (glob, repeatable)
  -L, --follow-symlinks      Follow symbolic links
      --json                 Output JSON report (non-interactive)
      --dry-run              Show deletion plan without deleting (non-interactive)
      --csv                  Output CSV report (non-interactive)
      --no-cache             Disable hash cache
      --paranoid             Byte-by-byte verification after hash match
  -h, --help                 Print help
  -V, --version              Print version
```
