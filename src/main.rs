use clap::{Parser, Subcommand};
use std::path::PathBuf;

// Import the library
use vast_parser::{parser, unwrap};
use vast_parser::async_api;

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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Parse { input, pretty } => {
            // Fetch the VAST content asynchronously
            let content = unwrap::fetch_vast_content_async(input).await?;
            
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
            // Fetch the VAST content asynchronously
            let content = unwrap::fetch_vast_content_async(input).await?;
            
            // Unwrap the VAST asynchronously
            let vast = async_api::unwrap_vast(&content).await?;
            
            // Print the unwrapped VAST
            if *pretty {
                println!("{:#?}", vast);
            } else {
                println!("{:?}", vast);
            }
        },
        Commands::Stitch { input, output } => {
            // Fetch the VAST content asynchronously
            let content = unwrap::fetch_vast_content_async(input).await?;
            
            // Stitch the VAST asynchronously
            let stitched_xml = async_api::stitch_vast(&content).await?;
            
            // Output the stitched VAST
            if let Some(output_path) = output {
                tokio::fs::write(output_path, &stitched_xml).await?;
                println!("Stitched VAST written to {}", output_path.display());
            } else {
                println!("{}", stitched_xml);
            }
        },
    }

    Ok(())
}
