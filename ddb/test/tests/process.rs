use ddb::Command;
use ddb_test::util::*;

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
fn launch_args() {
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

#[test]
fn drop_kills() {
    let process = test_binary().spawn().unwrap();

    let pid = process.pid();
    assert!(process_exists(pid));
    drop(process);
    // This test is potentially flaky because the PID of the spawned process
    // could be reused before we check again.
    assert!(!process_exists(pid));
}
