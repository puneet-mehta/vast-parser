# VAST Parser

A Rust library and command-line tool for parsing and working with VAST (Video Ad Serving Template) XML files.

## Features

- Parse VAST XML files (supports VAST 2.0, 3.0, and 4.0)
- Follow wrapper chains to find inline ads
- Stitch together a complete VAST document from wrapper chains
- Handles both local files and remote URLs
- Cycle detection for circular wrapper references
- Command-line interface for easy usage

## Installation

```bash
# Clone the repository
git clone https://github.com/puneet-mehta/vast-parser.git
cd vast-parser

# Build the project
cargo build --release
```

## Usage

### Command-line Interface

The CLI provides three main commands:

#### Parse

Parse a VAST XML file or URL and display its contents:

```bash
cargo run --release -- parse -i samples/sample_vast.xml -p
```

Options:
- `-i, --input`: Path to the VAST file or URL (required)
- `-p, --pretty`: Pretty print the output

#### Unwrap

Unwrap a VAST file or URL to find the InLine ad by following wrapper chains:

```bash
cargo run --release -- unwrap -i samples/sample_wrapper_nested.xml -p
```

Options:
- `-i, --input`: Path to the VAST file or URL (required)
- `-p, --pretty`: Pretty print the output

#### Stitch

Stitch together a complete VAST XML with merged tracking elements:

```bash
cargo run --release -- stitch -i samples/sample_stitch_test.xml -o stitched_vast.xml
```

Options:
- `-i, --input`: Path to the VAST file or URL (required)
- `-o, --output`: Output file path (if not specified, prints to stdout)

### Library Usage

You can also use the library in your Rust code:

```rust
use vast_parser::{parser, unwrap, stitcher};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse a VAST file
    let content = std::fs::read_to_string("samples/sample_vast.xml")?;
    let vast = parser::parse_vast(&content)?;
    
    // Unwrap a VAST wrapper
    let content = std::fs::read_to_string("samples/sample_wrapper.xml")?;
    let unwrapped = unwrap::unwrap_vast(&content)?;
    
    // Stitch a VAST file
    let content = std::fs::read_to_string("samples/sample_stitch_test.xml")?;
    let stitched = stitcher::stitch_vast(&content)?;
    
    Ok(())
}
```

## Sample Files

The `samples` directory contains example VAST XML files for testing:

- `sample_vast.xml`: A simple VAST file with an InLine ad
- `sample_wrapper.xml`: A VAST wrapper pointing to sample_vast.xml
- `sample_wrapper_nested.xml`: A nested wrapper chain
- `sample_wrapper_circular.xml`: A circular reference test case
- `sample_stitch_test.xml`: A test file for the stitching functionality

## License

This project is licensed under the MIT License - see the LICENSE file for details. 