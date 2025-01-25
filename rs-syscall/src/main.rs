// use std::arch::asm;

// fn main() {
//     let message = b"Hello, world!\n";
//
//     unsafe {
//         asm!(
//             "syscall",
//             in("rax") 0x1,                  // Syscall number (1 = write)
//             in("rdi") 1,                    // File descriptor (1 = stdout)
//             in("rsi") message.as_ptr(),     // Pointer to the message
//             in("rdx") message.len(),        // Message length
//             lateout("rax") _,               // Return value (not used here)
//             lateout("rdi") _,               // Clobbered register
//             lateout("rsi") _,               // Clobbered register
//             lateout("rdx") _,               // Clobbered register
//         );
//     }
// }

use nix::unistd::write;
use std::{io::stdout, os::fd::AsFd};

fn main() {
    let message = b"Hello, world!\n";
    write(stdout().as_fd(), message).unwrap();
}

// use libc::{write, STDOUT_FILENO};
//
// fn main() {
//     let message = b"Hello, world!\n";
//     unsafe {
//         write(STDOUT_FILENO, message.as_ptr() as *const _, message.len());
//     }
// }
