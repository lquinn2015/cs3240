use crate::console::{kprint, kprintln, CONSOLE};
use shim::io::Write;
use stack_vec::StackVec;

/// Error type for `Command` parse failures.
#[derive(Debug)]
enum Error {
    Empty,
    TooManyArgs,
}

/// A structure representing a single shell command.
struct Command<'a> {
    args: StackVec<'a, &'a str>,
}

impl<'a> Command<'a> {
    /// Parse a command from a string `s` using `buf` as storage for the
    /// arguments.
    ///
    /// # Errors
    ///
    /// If `s` contains no arguments, returns `Error::Empty`. If there are more
    /// arguments than `buf` can hold, returns `Error::TooManyArgs`.
    fn parse(s: &'a str, buf: &'a mut [&'a str]) -> Result<Command<'a>, Error> {
        let mut args = StackVec::new(buf);
        for arg in s.split(' ').filter(|a| !a.is_empty()) {
            args.push(arg).map_err(|_| Error::TooManyArgs)?;
        }

        if args.is_empty() {
            return Err(Error::Empty);
        }

        Ok(Command { args })
    }

    /// Returns this command's path. This is equivalent to the first argument.
    fn path(&self) -> &str {
        self.args[0]
    }
}

/// Starts a shell using `prefix` as the prefix for each line. This function
/// returns if the `exit` command is called.
pub fn shell(prefix: &str) -> ! {
    const BEL: u8 = 0x07; // BELL
    const BS: u8 = 0x08; // Backspace
    const DEL: u8 = 0x7F; // DEL
    const NL: u8 = 0x0A; // New Line
    const CR: u8 = 0x0D; // Carriage return

    'cmd: loop {
        let mut stack_buf = [0u8; 512];
        let mut stack = StackVec::new(&mut stack_buf);
        let mut cmd_buf: [&str; 64] = [""; 64];

        kprint!("{}", prefix);

        'arg: loop {
            let mut console = CONSOLE.lock();
            let input = console.read_byte();

            // Ring bell on bad chars
            if !input.is_ascii() {
                console.write_byte(BEL);
                continue 'arg;
            }

            match input {
                BS | DEL => {
                    if stack.pop().is_some() {
                        console.write(&[BS, b' ', BS]).ok(); // backout the previous char
                    } else {
                        console.write_byte(BEL); // no chars to backspace
                    }
                }
                NL | CR => {
                    match Command::parse(
                        core::str::from_utf8(stack.into_slice()).unwrap(),
                        &mut cmd_buf,
                    ) {
                        Ok(_cmd) => {
                            console.write_byte(NL);
                            console.write_byte(CR);
                            break 'arg; // this command should be dispatched somehow
                        }
                        Err(Error::Empty) => {
                            console.write_byte(NL);
                            console.write_byte(CR);
                            continue 'cmd; // Bad commad new line try again
                        }
                        Err(Error::TooManyArgs) => {
                            console.write_byte(NL);
                            console.write_byte(CR);
                            console.write_byte(BEL);
                            continue 'cmd; // Bad commad new line try again
                        }
                    }
                }
                _ => match stack.push(input) {
                    Ok(_) => console.write_byte(input),
                    Err(_) => console.write_byte(BEL),
                },
            }
        }
        // comand is valid do something
    }
}
