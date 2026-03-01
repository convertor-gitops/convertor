use crate::commands::{Command, Commander, pretty_command};
use clap::Parser;
use color_eyre::Result;
use color_eyre::eyre::eyre;
use color_eyre::owo_colors::OwoColorize;
use console::style;
use std::process::Command as StdCommand;

mod args;
mod commands;

#[derive(Debug, Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let cli = Cli::parse();
    let mut commands: Vec<StdCommand> = cli.command.create_command()?;
    let len = commands.len();
    for (i, command) in commands.iter_mut().enumerate() {
        let command_str = pretty_command(command);
        let instant = std::time::Instant::now();
        println!(
            "{} {}",
            style(format!("[{i}/{len}]")).green(),
            style(command_str).blue().bold().italic()
        );
        let status = command.status()?;
        if !status.success() {
            let message = format!("命令执行失败: {}, 状态: {}", pretty_command(command), status);
            return Err(eyre!("{}", message));
        }
        let elapsed = instant.elapsed();
        println!("{} {}", style("[完成]").green(), style(format!("耗时: {:.2?}", elapsed)).purple());
    }

    Ok(())
}

// fn init_log() {
//     tracing_subscriber::registry()
//         .with(
//             tracing_subscriber::fmt::layer()
//                 .with_target(true)
//                 .with_level(true)
//                 .with_file(false)
//                 .with_line_number(false)
//                 .with_thread_names(false)
//                 .with_ansi(std::io::stdout().is_terminal())
//                 .with_timer(tracing_subscriber::fmt::time::LocalTime::rfc_3339())
//                 .pretty()
//                 .compact(),
//         )
//         .init();
// }
