mod server;
mod sparql;
mod stdio_reader;

use std::{
    fs::{File, OpenOptions},
    io::{self, Read, Write},
    path::PathBuf,
    process::exit,
    rc::Rc,
};

use futures::lock::Mutex;
use log::LevelFilter;
use log4rs::{
    append::file::FileAppender,
    config::{Appender, Root},
    encode::pattern::PatternEncoder,
    Config,
};
use server::{format_raw, handle_message, Server};

use clap::{Parser, Subcommand};
use stdio_reader::StdioMessages;
use tokio::{runtime, task::LocalSet};

/// qlue-ls: An SPARQL language server and formatter
#[derive(Debug, Parser)]
#[command(version, about, long_about= None)]
struct Cli {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Run the language server
    Server,
    /// Run the formatter on a given file OR stdin
    Format {
        /// overwrite given file
        #[arg(short, long)]
        writeback: bool,
        /// Avoid writing formatted file back; instead, exit with a non-zero status code if any files would have been modified, and zero otherwise
        #[arg(short, long)]
        check: bool,
        /// Omit to read from stdin
        path: Option<PathBuf>,
    },
    /// Watch the logs (linux users only)
    Logs,
}

fn get_logfile_path() -> PathBuf {
    let mut app_dir = dirs_next::data_dir().expect("Failed to find data directory");
    app_dir.push("qlue-ls");
    if !app_dir.exists() {
        std::fs::create_dir_all(&app_dir).expect("Failed to create app directory");
    }
    app_dir.join("qlue-ls.log")
}

fn configure_logging() {
    let logfile_path = get_logfile_path();
    let logfile = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{l} - {m}{n}")))
        .build(logfile_path)
        .expect("Failed to create logfile");

    let config = Config::builder()
        .appender(Appender::builder().build("file", Box::new(logfile)))
        .build(Root::builder().appender("file").build(LevelFilter::Info))
        .unwrap();

    log4rs::init_config(config).expect("Failed to configure logger");
}

fn send_message(message: String) {
    print!("Content-Length: {}\r\n\r\n{}", message.len(), message);
    io::stdout().flush().expect("No IO errors or EOFs");
}

fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    configure_logging();

    let cli = Cli::parse();
    match cli.command {
        Command::Server => {
            // Start server and listen to stdio
            let rt = runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("Single threaded runtime should be usable");
            let local = LocalSet::new();
            rt.block_on(local.run_until(async {
                let server = Server::new(send_message);
                let server_rc = Rc::new(Mutex::new(server));
                for message in StdioMessages::new() {
                    handle_message(server_rc.clone(), message).await
                }
            }));
        }
        Command::Format {
            path,
            writeback,
            check,
        } => {
            if let Some(path) = path {
                match File::open(path.clone()) {
                    Ok(mut file) => {
                        let mut contents = String::new();
                        file.read_to_string(&mut contents)
                            .expect("Could not read file");
                        match format_raw(contents.clone()) {
                            Ok(formatted_contents) => {
                                let unchanged = formatted_contents == contents;
                                if check {
                                    if unchanged {
                                        println!("{} is already formatted", path.to_string_lossy());
                                        exit(0);
                                    } else {
                                        println!("{} would be reformatted", path.to_string_lossy());
                                        exit(1);
                                    }
                                }
                                if writeback {
                                    if unchanged {
                                        println!("{} left unchanged", path.to_string_lossy());
                                    } else {
                                        let mut file = OpenOptions::new()
                                            .write(true)
                                            .truncate(true)
                                            .append(false)
                                            .open(path.clone())
                                            .expect("Could not write to file");
                                        file.write_all(formatted_contents.as_bytes())
                                            .expect("Unable to write");
                                        println!("{} reformatted", path.to_string_lossy());
                                    }
                                } else {
                                    println!("{}", formatted_contents);
                                }
                            }
                            Err(e) => panic!("Error during formatting:\n{}", e),
                        }
                    }
                    Err(e) => {
                        panic!("Could not open file: {}", e)
                    }
                };
            } else {
                // No path was given -- read from stdin
                let mut buffer = Vec::new();
                io::stdin()
                    .read_to_end(&mut buffer)
                    .expect("Should read all bytes from stdin");
                // for line in io::stdin().lock().read_to_end(buffer).lines() {
                //     buffer += &line.unwrap();
                //     buffer += "\n";
                // }
                let input = String::from_utf8(buffer).expect("input should be valid UTF8");
                match format_raw(input) {
                    Ok(res) => {
                        print!("{}", res);
                    }
                    Err(e) => panic!("Error during formatting:\n{}", e),
                }
            }
        }
        Command::Logs => {
            let logfile_path = get_logfile_path();
            if !std::process::Command::new("tail")
                .args(["--lines", "0"])
                .arg("-f")
                .arg(logfile_path)
                .status()
                .expect("Failed to start 'tail' process")
                .success()
            {
                println!("tail exited with non-zero status");
            }
        }
    };
}
