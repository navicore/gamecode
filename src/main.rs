mod agent;
mod app;
mod core;
mod examples;
mod ui;
mod visualization;

use std::env;

use tracing::{debug, error, trace, warn};

#[tokio::main]
async fn main() {
    // Check command-line arguments
    let args: Vec<String> = env::args().collect();

    // Check if trace flag is enabled
    let trace_mode = args.contains(&String::from("--trace"));
    let debug_mode = args.contains(&String::from("--debug"));
    if trace_mode {
        tracing_subscriber::fmt()
            .with_env_filter("warn,gamecode=trace")
            .with_target(true)
            .init();
        trace!("Trace mode enabled");
        // SAFETY: We're just setting log levels which doesn't impact memory safety
        unsafe {
            std::env::set_var("RUST_LOG", "warn,gamecode=trace,aws_config=debug");
        }
    } else if debug_mode {
        tracing_subscriber::fmt()
            .with_env_filter("error,gamecode=debug")
            .with_target(true)
            .init();
        debug!("Debug mode enabled");
        // SAFETY: We're just setting log levels which doesn't impact memory safety
        unsafe {
            std::env::set_var("RUST_LOG", "error,gamecode=debug,aws_config=warn");
        }
    } else {
        tracing_subscriber::fmt()
            .with_env_filter("error,gamecode=warn")
            .with_target(true)
            .init();

        // SAFETY: We're just setting log levels which doesn't impact memory safety
        unsafe {
            std::env::set_var("RUST_LOG", "error,gamecode=warn,aws_config=error");
        }
    }

    if args.len() > 1 && args[1] == "--test-bedrock" {
        // Bedrock integration test temporarily disabled during integration
        warn!("Bedrock integration test is temporarily disabled during modular architecture integration");
    } else {
        // Run the normal application
        app::run();
    }
}
