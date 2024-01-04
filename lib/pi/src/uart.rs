use core::fmt;
use core::time::Duration;

use shim::const_assert_size;
use shim::io;

use volatile::prelude::*;
use volatile::{ReadVolatile, Reserved, Volatile};

use crate::common::IO_BASE;
use crate::gpio::{Function, Gpio};
use crate::timer;

/// The base address for the `MU` registers.
const MU_REG_BASE: usize = IO_BASE + 0x215040;

/// The `AUXENB` register from page 9 of the BCM2837 documentation.
const AUX_ENABLES: *mut Volatile<u8> = (IO_BASE + 0x215004) as *mut Volatile<u8>;

/// Enum representing bit fields of the `AUX_MU_LSR_REG` register.
#[repr(u8)]
enum LsrStatus {
    DataReady = 1,
    TxAvailable = 1 << 5,
}

#[repr(C)]
#[allow(non_snake_case)]
struct Registers {
    IO: Volatile<u8>,
    _r0: [Reserved<u8>; 3],
    IER: Volatile<u8>,
    _r1: [Reserved<u8>; 3],
    IIR: Volatile<u8>,
    _r2: [Reserved<u8>; 3],
    LCR: Volatile<u8>,
    _r3: [Reserved<u8>; 3],
    MCR: Volatile<u8>,
    _r4: [Reserved<u8>; 3],
    /// Reading clears
    LSR: ReadVolatile<u8>,
    _r5: [Reserved<u8>; 3],
    MSR: ReadVolatile<u8>,
    _r6: [Reserved<u8>; 3],
    SCRATCH: Volatile<u32>,
    CNTL: Volatile<u8>,
    _r8: [Reserved<u8>; 3],
    STAT: ReadVolatile<u32>,
    BAUD: Volatile<u16>,
}

const_assert_size!(Registers, 0x7E21506C - 0x7E215040);

/// The Raspberry Pi's "mini UART".
pub struct MiniUart {
    registers: &'static mut Registers,
    timeout: Option<Duration>,
}

#[repr(u16)]
pub enum BaudRate {
    Baud19200 = 1627,
    Baud38400 = 813,
    Baud76800 = 406,
    Baud115200 = 270,
}

impl MiniUart {
    /// Initializes the mini UART by enabling it as an auxiliary peripheral,
    /// setting the data size to 8 bits, setting the BAUD rate to ~115200 (baud
    /// divider of 270), setting GPIO pins 14 and 15 to alternative function 5
    /// (TXD1/RDXD1), and finally enabling the UART transmitter and receiver.
    ///
    /// By default, reads will never time out. To set a read timeout, use
    /// `set_read_timeout()`.
    pub fn new(baud: BaudRate) -> MiniUart {
        let registers = unsafe {
            // Enable the mini UART as an auxiliary device.
            (*AUX_ENABLES).or_mask(1);
            &mut *(MU_REG_BASE as *mut Registers)
        };

        Gpio::new(14).into_alt(Function::Alt5);
        Gpio::new(15).into_alt(Function::Alt5);

        // 115200 baud rate BCM2835 pg:10,19
        registers.BAUD.write(baud as u16);

        // set UART to 8-bit mode and clear DLAB access BCM2835 pg: 14
        registers.LCR.write(0b11);

        // clears the tx/rx fifos : pg 13
        registers.IIR.write(0b11);

        // enables Rx and Tx
        registers.CNTL.write(0b11);

        MiniUart {
            registers,
            timeout: None,
        }
    }

    /// Set the read timeout to `t` duration.
    pub fn set_read_timeout(&mut self, t: Duration) {
        self.timeout = Some(t);
    }

    /// Write the byte `byte`. This method blocks until there is space available
    /// in the output FIFO.
    pub fn write_byte(&mut self, byte: u8) {
        while !(self.registers.LSR.has_mask(LsrStatus::TxAvailable as u8)) {}
        self.registers.IO.write(byte);
    }

    pub fn clear(&mut self) {
        while self.has_byte() {
            self.read_byte();
        }
    }

    /// Returns `true` if there is at least one byte ready to be read. If this
    /// method returns `true`, a subsequent call to `read_byte` is guaranteed to
    /// return immediately. This method does not block.
    pub fn has_byte(&self) -> bool {
        self.registers.LSR.has_mask(LsrStatus::DataReady as u8)
    }

    /// Blocks until there is a byte ready to read. If a read timeout is set,
    /// this method blocks for at most that amount of time. Otherwise, this
    /// method blocks indefinitely until there is a byte to read.
    ///
    /// Returns `Ok(())` if a byte is ready to read. Returns `Err(())` if the
    /// timeout expired while waiting for a byte to be ready. If this method
    /// returns `Ok(())`, a subsequent call to `read_byte` is guaranteed to
    /// return immediately.
    pub fn wait_for_byte(&self) -> Result<(), ()> {
        let dur = if let Some(d) = self.timeout {
            d
        } else {
            Duration::default()
        };
        let wake = timer::current_time() + dur;
        while !self.has_byte() {
            if let Some(_dur) = self.timeout {
                if timer::current_time() > wake {
                    return Err(());
                }
            }
        }
        Ok(())
    }

    /// Reads a byte. Blocks indefinitely until a byte is ready to be read.
    pub fn read_byte(&mut self) -> u8 {
        while !self.has_byte() {}
        self.registers.IO.read()
    }
}

impl fmt::Write for MiniUart {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            if byte == b'\n' {
                self.write_byte(b'\r');
            }
            self.write_byte(byte);
        }
        Ok(())
    }
}

mod uart_io {
    use super::io;
    use super::MiniUart;
    use shim::ioerr;

    // The `io::Read::read()` implementation must respect the read timeout by
    // waiting at most that time for the _first byte_. It should not wait for
    // any additional bytes but _should_ read as many bytes as possible. If the
    // read times out, an error of kind `TimedOut` should be returned.
    impl io::Read for MiniUart {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            if self.wait_for_byte().is_err() {
                return ioerr!(TimedOut, "out of time");
            }
            let mut i = 0;
            while self.has_byte() && i < buf.len() {
                buf[i] = self.read_byte();
                i += 1;
            }
            Ok(i)
        }
    }
    // The `io::Write::write()` method must write all of the requested bytes
    // before returning.
    impl io::Write for MiniUart {
        /// `write` will block if no buffer space is available
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            for b in buf.iter() {
                self.write_byte(*b);
            }
            Ok(buf.len())
        }

        /// Flush is a nop since write will block on overflow
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
}
