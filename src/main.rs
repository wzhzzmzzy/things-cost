mod cli;
mod database;
mod models;
mod services;

use clap::Parser;
use cli::{Cli, CliHandler};

fn main() {
    let cli = Cli::parse();

    let handler = match CliHandler::new() {
        Ok(handler) => handler,
        Err(e) => {
            eprintln!("初始化失败: {}", e);
            std::process::exit(1);
        }
    };

    if let Err(e) = handler.handle_command(cli.command) {
        eprintln!("执行失败: {}", e);
        std::process::exit(1);
    }
}
