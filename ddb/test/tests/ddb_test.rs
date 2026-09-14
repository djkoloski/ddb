mod util {
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
}

mod process {
    use ddb::Command;

    use super::util::*;

    #[test]
    fn launch_fail_no_such_program() {
        Command::new("nonexistent").spawn().unwrap_err();
    }

    #[test]
    fn launch() {
        let process = test_binary().spawn().unwrap();
        assert!(process_exists(process.pid()));
    }

    #[test]
    fn kill_on_drop() {
        let process = test_binary().spawn().unwrap();

        let pid = process.pid();
        assert!(process_exists(pid));
        drop(process);
        // This test is potentially flaky because the PID of the spawned process
        // could be reused before we check again.
        assert!(!process_exists(pid));
    }

    #[test]
    fn test_args() {
        let mut pipe = [0; 2];
        let status = unsafe { libc::pipe(pipe.as_mut_ptr()) };
        assert!(status >= 0);

        let args = vec!["1234", "abcd", "hello world"];
        let expected = args.join(",");
        let _process = test_binary()
            .arg("echo")
            .arg(pipe[1].to_string())
            .args(args.into_iter())
            .spawn()
            .unwrap();

        const BUF_SIZE: usize = 1024;
        let mut buf = [0u8; BUF_SIZE];
        let bytes =
            unsafe { libc::read(pipe[0], buf.as_mut_ptr().cast(), BUF_SIZE) };
        assert!(bytes >= 0);
        assert_eq!(str::from_utf8(&buf[..bytes as usize]).unwrap(), expected);
    }
}

mod attachment {
    use core::assert_matches;

    use ddb::{Attachment, Command, State};

    use super::util::*;

    #[test]
    fn launch_no_such_program() {
        Attachment::spawn_attached(Command::new("nonexistent")).unwrap_err();
    }

    #[test]
    fn launch_success() {
        let (process, _attachment) =
            Attachment::spawn_attached(test_binary()).unwrap();
        assert!(process_exists(process.pid()));
        assert_eq!(
            read_process_state(process.pid()),
            ProcessState::TracingStopped
        );
    }

    #[test]
    fn attach_invalid_pid() {
        Attachment::attach(0).unwrap_err();
    }

    #[test]
    fn attach_success() {
        let process = test_binary().spawn().unwrap();
        assert!(process_exists(process.pid()));
        assert_eq!(read_process_state(process.pid()), ProcessState::Running);

        let attachment = Attachment::attach(process.pid()).unwrap();
        assert_eq!(
            read_process_state(process.pid()),
            ProcessState::TracingStopped
        );
        assert_eq!(attachment.state(), State::Stopped);
    }

    #[test]
    fn launch_resume_success() {
        let (process, mut attachment) =
            Attachment::spawn_attached(test_binary()).unwrap();
        attachment.resume().unwrap();
        assert_matches!(
            read_process_state(process.pid()),
            ProcessState::Running | ProcessState::Sleeping,
        );
    }

    #[test]
    fn attach_resume_success() {
        let process = test_binary().spawn().unwrap();
        let mut attachment = Attachment::attach(process.pid()).unwrap();
        attachment.resume().unwrap();
        assert_matches!(
            read_process_state(process.pid()),
            ProcessState::Running | ProcessState::Sleeping,
        )
    }
}
