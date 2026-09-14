use core::ffi::c_int;
use std::env::args;

fn main() {
    match args().nth(1).as_deref() {
        None => loop {
            println!("y");
        },
        Some("echo") => {
            let fd = args().nth(2).unwrap().parse::<c_int>().unwrap();
            let args = args().skip(3).collect::<Vec<_>>().join(",");
            unsafe {
                libc::write(fd, args.as_ptr().cast(), args.len());
            }
        }
        Some("exit") => (),
        Some(_) => eprintln!("unrecognized command"),
    }
}
