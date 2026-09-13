mod process {
    use std::path::Path;

    use ddb::{Errno, Process};
    use libc::{kill, pid_t};

    fn exists(pid: pid_t) -> bool {
        let status = unsafe { kill(pid, 0) };
        status != -1 && Errno::read() != libc::ESRCH
    }

    #[test]
    fn launch_success() {
        let process = Process::launch_attached(Path::new("yes")).unwrap();
        assert!(exists(process.pid()));
    }

    #[test]
    fn launch_no_such_program() {
        Process::launch_attached(Path::new("you_do_not_have_to_be_good"))
            .unwrap_err();
    }
}
