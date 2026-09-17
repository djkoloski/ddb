use core::assert_matches;

use ddb::{Command, Debugger, Register, State, f80, u8x8, u8x16};
use ddb_test::util::*;

#[test]
fn launch_fail_no_such_program() {
    Debugger::spawn_attached(Command::new("nonexistent")).unwrap_err();
}

#[test]
fn launch() {
    let (_process, debugger) = Debugger::spawn_attached(test_binary()).unwrap();
    assert!(process_exists(debugger.pid()));
    assert_eq!(
        read_process_state(debugger.pid()),
        ProcessState::TracingStopped
    );
}

#[test]
fn attach_fail_invalid_pid() {
    Debugger::attach(0).unwrap_err();
}

#[test]
fn attach() {
    let process = test_binary().spawn().unwrap();
    assert!(process_exists(process.pid()));
    assert_eq!(read_process_state(process.pid()), ProcessState::Running);

    let debugger = Debugger::attach(process.pid()).unwrap();
    assert_eq!(
        read_process_state(debugger.pid()),
        ProcessState::TracingStopped
    );
    assert_eq!(debugger.state(), State::Stopped);
}

#[test]
fn launch_resume() {
    let (_process, mut debugger) =
        Debugger::spawn_attached(test_binary()).unwrap();
    debugger.resume().unwrap();
    assert_matches!(
        read_process_state(debugger.pid()),
        ProcessState::Running | ProcessState::Sleeping,
    );
}

#[test]
fn attach_resume() {
    let process = test_binary().spawn().unwrap();
    let mut debugger = Debugger::attach(process.pid()).unwrap();
    debugger.resume().unwrap();
    assert_matches!(
        read_process_state(debugger.pid()),
        ProcessState::Running | ProcessState::Sleeping,
    )
}

#[test]
fn launch_fail_resume_terminated() {
    let (_process, mut debugger) =
        Debugger::spawn_attached(test_binary().arg("exit")).unwrap();
    debugger.resume().unwrap();
    let _ = debugger.wait_for_state_change().unwrap();
    debugger.resume().unwrap_err();
}

#[test]
fn read_registers() {
    let (_process, debugger) =
        Debugger::spawn_attached(test_binary().arg("exit")).unwrap();

    let rax = debugger.read_reg::<u64>(Register::rax).unwrap();
    assert_eq!(debugger.read_reg::<u32>(Register::eax).unwrap(), rax as u32,);
    assert_eq!(debugger.read_reg::<u16>(Register::ax).unwrap(), rax as u16,);
    assert_eq!(
        debugger.read_reg::<u8>(Register::ah).unwrap(),
        (rax >> 8) as u8,
    );
    assert_eq!(debugger.read_reg::<u8>(Register::al).unwrap(), rax as u8,);

    debugger.read_reg_value(Register::fcw);
    debugger.read_reg_value(Register::fsw);
    debugger.read_reg_value(Register::frip);
    debugger.read_reg_value(Register::mxcsr);

    debugger.read_reg::<f80>(Register::st0).unwrap();
    debugger.read_reg::<f80>(Register::st7).unwrap();

    debugger.read_reg::<u8x8>(Register::mm0).unwrap();
    debugger.read_reg::<u8x8>(Register::mm7).unwrap();

    debugger.read_reg::<u8x16>(Register::xmm0).unwrap();
    debugger.read_reg::<u8x16>(Register::xmm15).unwrap();
}

#[test]
fn write_registers() {
    let (_process, mut debugger) =
        Debugger::spawn_attached(test_binary().arg("exit")).unwrap();

    const TEST_DATA_8X8: [u8; 8] =
        [0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef];
    const TEST_DATA_8X10: [u8; 10] =
        [0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0x01, 0x23];
    const TEST_DATA_8X16: [u8; 16] = [
        0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0x01, 0x23, 0x45, 0x67,
        0x89, 0xab, 0xcd, 0xef,
    ];

    const TEST_80: f80 = f80::from_bytes(TEST_DATA_8X10);
    const TEST_64: u64 = 0x0123_4567_89ab_cdef;
    const TEST_32: u32 = 0x0123_4567;
    const TEST_16: u16 = 0x0123;
    const TEST_8: u8 = 0x01;
    const TEST_8X8: u8x8 = u8x8::from_bytes(TEST_DATA_8X8);
    const TEST_8X16: u8x16 = u8x16::from_bytes(TEST_DATA_8X16);

    debugger.write_reg(Register::rax, TEST_64).unwrap();
    assert_eq!(debugger.read_reg::<u64>(Register::rax).unwrap(), TEST_64);

    debugger.write_reg(Register::eax, TEST_32).unwrap();
    assert_eq!(debugger.read_reg::<u32>(Register::eax).unwrap(), TEST_32);

    debugger.write_reg(Register::ax, TEST_16).unwrap();
    assert_eq!(debugger.read_reg::<u16>(Register::ax).unwrap(), TEST_16);

    debugger.write_reg(Register::ah, TEST_8).unwrap();
    assert_eq!(debugger.read_reg::<u8>(Register::ah).unwrap(), TEST_8);

    debugger.write_reg(Register::al, TEST_8).unwrap();
    assert_eq!(debugger.read_reg::<u8>(Register::al).unwrap(), TEST_8);

    debugger.write_reg(Register::fcw, TEST_16).unwrap();
    assert_eq!(debugger.read_reg::<u16>(Register::fcw).unwrap(), TEST_16);

    debugger.write_reg(Register::fsw, TEST_16).unwrap();
    assert_eq!(debugger.read_reg::<u16>(Register::fsw).unwrap(), TEST_16);

    debugger.write_reg(Register::frip, TEST_64).unwrap();
    assert_eq!(debugger.read_reg::<u64>(Register::frip).unwrap(), TEST_64);

    debugger.write_reg(Register::st0, TEST_80).unwrap();
    assert_eq!(
        debugger
            .read_reg::<f80>(Register::st0)
            .unwrap()
            .to_ne_bytes(),
        TEST_DATA_8X10,
    );

    debugger.write_reg(Register::st7, TEST_80).unwrap();
    assert_eq!(
        debugger
            .read_reg::<f80>(Register::st7)
            .unwrap()
            .to_ne_bytes(),
        TEST_DATA_8X10,
    );

    debugger.write_reg(Register::mm0, TEST_8X8).unwrap();
    assert_eq!(
        debugger
            .read_reg::<u8x8>(Register::mm0)
            .unwrap()
            .to_ne_bytes(),
        TEST_DATA_8X8,
    );

    debugger.write_reg(Register::mm7, TEST_8X8).unwrap();
    assert_eq!(
        debugger
            .read_reg::<u8x8>(Register::mm7)
            .unwrap()
            .to_ne_bytes(),
        TEST_DATA_8X8,
    );

    debugger.write_reg(Register::xmm0, TEST_8X16).unwrap();
    assert_eq!(
        debugger
            .read_reg::<u8x16>(Register::xmm0)
            .unwrap()
            .to_ne_bytes(),
        TEST_DATA_8X16,
    );

    debugger.write_reg(Register::xmm7, TEST_8X16).unwrap();
    assert_eq!(
        debugger
            .read_reg::<u8x16>(Register::xmm7)
            .unwrap()
            .to_ne_bytes(),
        TEST_DATA_8X16,
    );
}
