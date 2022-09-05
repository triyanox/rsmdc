# rsmdc - Markdown to HTML converter

**rsmdc** is a Markdown to HTML converter written in Rust.

# Usage 

First install the binary:

```bash
cargo install rsmdc
```

Then you can use it like this to print the HTML to stdout:

```bash
rsmdc --filename [file]
```

Or like this to write the HTML to a file:

```bash
rsmdc --filename [file] --save [path]
```
