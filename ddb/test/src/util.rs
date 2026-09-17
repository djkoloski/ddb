use std::{fs::File, io::Read};

use ddb::{Command, Errno};
use libc::{kill, pid_t};

pub fn test_binary() -> Command {
    Command::new(std::env::var_os("CARGO_BIN_EXE_ddb_test").unwrap())
}

pub fn process_exists(pid: pid_t) -> bool {
    let status = unsafe { kill(pid, 0) };
    status != -1 && Errno::read() != libc::ESRCH
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessState {
    Running,
    Sleeping,
    Waiting,
    Zombie,
    Stopped,
    TracingStopped,
    Dead,
}

pub fn read_process_state(pid: pid_t) -> ProcessState {
    let mut stat = File::open(format!("/proc/{pid}/stat")).unwrap();
    let mut buf = String::new();
    stat.read_to_string(&mut buf).unwrap();
    let last_paren = buf.rfind(')').unwrap();
    match &buf[last_paren + 2..last_paren + 3] {
        "R" => ProcessState::Running,
        "S" => ProcessState::Sleeping,
        "D" => ProcessState::Waiting,
        "Z" => ProcessState::Zombie,
        "T" => ProcessState::Stopped,
        "t" => ProcessState::TracingStopped,
        "X" | "x" => ProcessState::Dead,
        state => panic!("invalid process state '{state}'"),
    }
}
