mod args;
mod error;

use std::{
    collections::vec_deque::VecDeque,
    io::{self, Write as _},
    process::ExitCode,
};

use ddb::{Attachment, Process, Recoverable as _, StateChange};

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
    // The spawned process, if any
    #[expect(unused)]
    process: Option<Process>,
    attachment: Attachment,
    is_running: bool,
    history: VecDeque<String>,
}

impl Cli {
    fn run() -> Result<(), Error> {
        Self::new()?.execute()
    }

    fn new() -> Result<Self, Error> {
        let args = Args::parse()?;

        let (process, attachment) = match args.command {
            Command::Attach { pid } => (None, Attachment::attach(pid)?),
            Command::Launch { path } => {
                let (p, a) = Attachment::launch(&path)?;
                (Some(p), a)
            }
        };

        Ok(Self {
            process,
            attachment,
            is_running: true,
            history: VecDeque::new(),
        })
    }

    fn execute(&mut self) -> Result<(), Error> {
        println!("attached to process {}", self.attachment.pid());

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
                if let Err(e) = self.attachment.resume().recover()? {
                    eprintln!("{e}");
                    return Ok(());
                }
            }
            "w" | "wait" => {
                let change = StateChange::wait_for_pid(self.attachment.pid())?;
                let pid = self.attachment.pid();
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
