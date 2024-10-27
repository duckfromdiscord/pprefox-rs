use clap::Parser;
use parking_lot::RwLock;
use pprefox_rs::firefox_write;
use pprefox_rs::http::{http_server, AppState};
use pprefox_rs::json::*;
use pprefox_rs::nmh_files_setup;
use std::collections::{HashMap, VecDeque};
use std::io;
use std::sync::Arc;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc::UnboundedSender;

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    serve: bool,
}

// Separate thread for reading only
async fn read_loop(output_channels: Arc<RwLock<HashMap<String, UnboundedSender<ExtensionResponse>>>>) {
    loop {
        if let Ok(input) = read().await {
            let decoded_input = String::from_utf8_lossy(&input).to_string();
            if let Ok(resp) = serde_json::from_str::<ExtensionResponse>(&decoded_input) {
                let output_channels = output_channels.read();
                if let Some(tx) = output_channels.get(&resp.uuid) {
                    // Send the browser's response to a channel waiting for it
                    let _ = tx.send(resp);
                } // otherwise, there was a repeat response
                drop(output_channels);
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    // If running in serve mode, serve
    if args.serve {
        let state = AppState {
            incoming_receivers: Arc::new(HashMap::new().into()),
            outgoing: Arc::new(VecDeque::new().into()),
        };
        // Run HTTP server in a separate thread
        http_server(state.clone()).unwrap();
        // Run read loop in a separate thread
        tokio::spawn(read_loop(state.incoming_receivers));
        loop {
            // Main thread will send any messages in the outbound queue
            if let Some(l) = state.outgoing.try_read() {
                if !(*l).is_empty() {
                    drop(l);
                    if let Some(mut l) = state.outgoing.try_write() {
                        // Serialize and send the first request
                        if let Some(outgoing_request) = l.pop_front() {
                            write(
                                &outgoing_request
                                    .1
                                    .serialize(outgoing_request.0.to_string())
                                    .unwrap(),
                            )
                            .await?;
                        };
                    }
                }
            }
        }
    } else {
        // If not in server mode, setup registry and Native messaging files
        let exe_path = std::env::current_exe().unwrap();
        // `@echo off` is required in the script Firefox runs or it will get overloaded
        // Each message starts with a 4 byte length and we don't know what that'll be if we copy command line input
        // The next line is just the path to the currently running executable, and `-s` to enable server mode
        // TODO: Allow the user to pick that path themselves
        let batch_contents = "@echo off\r\n".to_string() + exe_path.to_str().unwrap() + " -s";
        // Write host JSON and batch file to AppData
        let host_path = nmh_files_setup(&batch_contents).unwrap();
        // Set up registry entries
        Ok(firefox_write(host_path.to_str().unwrap())?)
    }
}

// Async read from stdin
async fn read() -> io::Result<Vec<u8>> {
    let mut stdin = tokio::io::stdin();
    let mut length = [0; 4];
    stdin.read_exact(&mut length).await?;
    let mut buffer = vec![0; u32::from_ne_bytes(length) as usize];
    stdin.read_exact(&mut buffer).await?;
    Ok(buffer)
}

// Async write to stdout
async fn write(message: &[u8]) -> io::Result<()> {
    let mut stdout = tokio::io::stdout();
    let length = message.len() as u32;
    stdout.write_all(&length.to_ne_bytes()).await?;
    stdout.write_all(message).await?;
    stdout.flush().await?;
    Ok(())
}
