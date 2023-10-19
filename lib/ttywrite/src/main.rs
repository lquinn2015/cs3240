mod parsers;

use serial;
use structopt;
use structopt_derive::StructOpt;
use xmodem::Progress;
use xmodem::Xmodem;

use std::path::PathBuf;
use std::time::Duration;

use serial::core::{BaudRate, CharSize, FlowControl, SerialDevice, SerialPortSettings, StopBits};
use structopt::StructOpt;

use parsers::{parse_baud_rate, parse_flow_control, parse_stop_bits, parse_width};

#[derive(StructOpt, Debug)]
#[structopt(about = "Write to TTY using the XMODEM protocol by default.")]
struct Opt {
    #[structopt(
        short = "i",
        help = "Input file (defaults to stdin if not set)",
        parse(from_os_str)
    )]
    input: Option<PathBuf>,

    #[structopt(
        short = "b",
        long = "baud",
        parse(try_from_str = "parse_baud_rate"),
        help = "Set baud rate",
        default_value = "115200"
    )]
    baud_rate: BaudRate,

    #[structopt(
        short = "t",
        long = "timeout",
        parse(try_from_str),
        help = "Set timeout in seconds",
        default_value = "10"
    )]
    timeout: u64,

    #[structopt(
        short = "w",
        long = "width",
        parse(try_from_str = "parse_width"),
        help = "Set data character width in bits",
        default_value = "8"
    )]
    char_width: CharSize,

    #[structopt(help = "Path to TTY device", parse(from_os_str))]
    tty_path: PathBuf,

    #[structopt(
        short = "f",
        long = "flow-control",
        parse(try_from_str = "parse_flow_control"),
        help = "Enable flow control ('hardware' or 'software')",
        default_value = "none"
    )]
    flow_control: FlowControl,

    #[structopt(
        short = "s",
        long = "stop-bits",
        parse(try_from_str = "parse_stop_bits"),
        help = "Set number of stop bits",
        default_value = "1"
    )]
    stop_bits: StopBits,

    #[structopt(short = "r", long = "raw", help = "Disable XMODEM")]
    raw: bool,
}

enum Input {
    File(std::fs::File),
    Stdin(std::io::Stdin),
}

use std::io;
use std::io::{Read, Write};

impl io::Read for Input {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match *self {
            Input::File(ref mut file) => file.read(buf),
            Input::Stdin(ref mut stdin) => stdin.read(buf),
        }
    }
}

fn main() {
    let opt = Opt::from_args();
    let mut port = serial::open(&opt.tty_path).expect("path points to invalid TTY");

    let mut ioset = port.read_settings().unwrap();
    ioset.set_baud_rate(opt.baud_rate).unwrap();
    ioset.set_stop_bits(opt.stop_bits);
    ioset.set_flow_control(opt.flow_control);
    ioset.set_char_size(opt.char_width);
    port.write_settings(&ioset).unwrap();
    port.set_timeout(Duration::from_secs(opt.timeout)).unwrap();

    let mut input = if let Some(fd) = opt.input {
        Input::File(std::fs::File::open(fd).expect("File must exist"))
    } else {
        Input::Stdin(io::stdin())
    };

    if opt.raw {
        let mut buf = [0u8; 128];
        loop {
            let amt = input.read(&mut buf[..]);
            match amt {
                Ok(0) => break,
                Ok(amt) => {
                    if let Some(_err) = port.write(&buf[..amt]).err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    } else {
        Xmodem::transmit_with_progress(input, port, |p: Progress| {
            println!("Progress {:?}", p);
        })
        .unwrap();
    }
}
