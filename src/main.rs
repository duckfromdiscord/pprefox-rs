use clap::Parser;
use directories::ProjectDirs;
use pprefox_rs::http::{http_server, AppState};
use pprefox_rs::json::*;
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    serve: bool,
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
        natemess::io::spawn_read_loop(move |input| {
            let output_channels = state.incoming_receivers.clone();
            let decoded_input = String::from_utf8_lossy(&input).to_string();
            if let Ok(resp) = serde_json::from_str::<ExtensionResponse>(&decoded_input) {
                let output_channels = output_channels.read();
                if let Some(tx) = output_channels.get(&resp.uuid) {
                    // Send the browser's response to a channel waiting for it
                    let _ = tx.send(resp);
                } // otherwise, there was a repeat response
                drop(output_channels);
            }
        });
        loop {
            // Main thread will send any messages in the outbound queue
            if let Some(l) = state.outgoing.try_read() {
                if !(*l).is_empty() {
                    drop(l);
                    if let Some(mut l) = state.outgoing.try_write() {
                        // Serialize and send the first request
                        if let Some(outgoing_request) = l.pop_front() {
                            natemess::io::write(
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
        // Directories to store the config and serve script
        let proj_dirs = ProjectDirs::from("", "duck", "pprefox_rs")
            .expect("Could not initialize project directories for pprefox-rs");

        std::fs::create_dir_all(proj_dirs.data_dir())?;

        let host_path = proj_dirs.data_dir().join("host.json");
        let script_path = proj_dirs.data_dir().join("nmhhost.bat");

        
        natemess::install::nmh_files_setup(
            &batch_contents,
            host_path.clone(),
            script_path,
            "pprefox@duckfromdiscord.github.io",
            "pprefox_rs",
            "pprefox_rs",
        )
        .unwrap();

        // Set up registry entries
        Ok(natemess::install::firefox_registry_setup(
            host_path.to_str().unwrap(),
            "pprefox_rs",
        )?)
    }
}
