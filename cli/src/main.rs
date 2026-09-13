mod args;
mod error;

use std::{
    collections::vec_deque::VecDeque,
    io::{self, Write as _},
    process::ExitCode,
};

use ddb::{Process, Recoverable as _};

use crate::{
    args::{Args, Command},
    error::Error,
};

fn main() -> ExitCode {
    if let Err(e) = Cli::run() {
        eprintln!("{e}");
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}

const HISTORY_LIMIT: usize = 100;

struct Cli {
    process: Process,
    is_running: bool,
    history: VecDeque<String>,
}

impl Cli {
    fn run() -> Result<(), Error> {
        Self::new()?.execute()
    }

    fn new() -> Result<Self, Error> {
        let args = Args::parse()?;

        let process = match args.command {
            Command::Attach { pid } => Process::attach(pid)?,
            Command::Launch { path } => Process::launch_attached(&path)?,
        };

        Ok(Self {
            process,
            is_running: true,
            history: VecDeque::new(),
        })
    }

    fn execute(&mut self) -> Result<(), Error> {
        println!("attached to process {}", self.process.pid());

        let mut line = String::new();
        while self.is_running {
            print!("ddb> ");
            io::stdout().flush()?;

            line.clear();
            io::stdin().read_line(&mut line)?;
            line.pop();

            if line.is_empty() {
                line = self.history.back().cloned().unwrap_or_default();
            }

            self.handle_command(&line)?;

            if !line.is_empty() {
                self.history.push_back(line.clone());
                while self.history.len() > HISTORY_LIMIT {
                    self.history.pop_front();
                }
            }
        }

        Ok(())
    }

    fn handle_command(&mut self, command: &str) -> Result<(), Error> {
        let mut pieces = command.split(' ');

        let Some(c) = pieces.next() else {
            eprintln!("no command given");
            return Ok(());
        };

        match c {
            "h" | "help" => {
                println!("{HELP}");
            }
            "q" | "quit" => {
                self.is_running = false;
            }
            "c" | "continue" => {
                if let Err(e) = self.process.resume().recover()? {
                    eprintln!("{e}");
                    return Ok(());
                }
            }
            "w" | "wait" => {
                let change = self.process.wait_for_state_change()?;
                let pid = self.process.pid();
                println!(
                    "process {pid} {} with status {}",
                    change.state, change.signal
                );
            }
            _ => {
                eprintln!("unknown command '{c}'");
            }
        }

        Ok(())
    }
}

const HELP: &str = "\
Commands:

help (h)
    Print commands and descriptions.

quit (q)
    Halt debugging, detach, and exit. If the attached program was launched by
    ddb, it will be terminated. Otherwise, it will resume execution.

continue (c)
    Continue execution of the program.

wait (w)
    Wait for a state change in the attached program.";
