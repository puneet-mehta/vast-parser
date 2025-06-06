use clap::{Parser, Subcommand};
use std::path::PathBuf;

// Import the library
use vast_parser::{parser, stitcher, unwrap};

/// VAST parser and unwrapper
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Parse a VAST file or URL
    Parse {
        /// Path to the VAST file or URL
        #[arg(short, long)]
        input: String,
        
        /// Pretty print the output
        #[arg(short, long)]
        pretty: bool,
    },
    
    /// Unwrap a VAST file or URL to find the InLine ad
    Unwrap {
        /// Path to the VAST file or URL
        #[arg(short, long)]
        input: String,
        
        /// Pretty print the output
        #[arg(short, long)]
        pretty: bool,
    },
    
    /// Stitch together a complete VAST XML with merged tracking elements
    Stitch {
        /// Path to the VAST file or URL
        #[arg(short, long)]
        input: String,
        
        /// Output file path (if not specified, prints to stdout)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Parse { input, pretty } => {
            // Fetch the VAST content
            let content = unwrap::fetch_vast_content(input)?;
            
            // Parse the VAST XML
            let vast = parser::parse_vast(&content)?;
            
            // Print the parsed VAST
            if *pretty {
                println!("{:#?}", vast);
            } else {
                println!("{:?}", vast);
            }
        },
        Commands::Unwrap { input, pretty } => {
            // Fetch the VAST content
            let content = unwrap::fetch_vast_content(input)?;
            
            // Unwrap the VAST
            let vast = unwrap::unwrap_vast(&content)?;
            
            // Print the unwrapped VAST
            if *pretty {
                println!("{:#?}", vast);
            } else {
                println!("{:?}", vast);
            }
        },
        Commands::Stitch { input, output } => {
            // Fetch the VAST content
            let content = unwrap::fetch_vast_content(input)?;
            
            // Stitch the VAST
            let stitched_xml = stitcher::stitch_vast(&content)?;
            
            // Output the stitched VAST
            if let Some(output_path) = output {
                std::fs::write(output_path, &stitched_xml)?;
                println!("Stitched VAST written to {}", output_path.display());
            } else {
                println!("{}", stitched_xml);
            }
        },
    }

    Ok(())
}
