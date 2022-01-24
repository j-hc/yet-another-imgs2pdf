# yet-another-imgs2pdf

Converts and merges multiple images into a single PDF. Customizable

# Usage
```
yet-another-imgs2pdf 0.1.0
scrubjay55
USAGE:
    yet-another-imgs2pdf.exe [OPTIONS] --out <out> <--imgs <imgs>...|--dir <dir>>

OPTIONS:
    -d, --dir <dir>                      Directory to folder of images
        --dpi <dpi>                      [default: 100.0]
    -h, --scale-height <scale-height>    [default: 1280]
        --help                           Print help information
    -i, --imgs <imgs>...                 Paths to multiple images seperated with a whitespace
    -o, --out <out>
    -s, --auto-sort
    -t, --pdf-title <pdf-title>
    -V, --version                        Print version information
    -w, --scale-width <scale-width>      [default: 720]
```


# Build

- Install Rust
```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

- Clone the repo and build with cargo

```
git clone https://github.com/scrubjay55/yet-another-imgs2pdf
cd yet-another-imgs2pdf
cargo build --release
```