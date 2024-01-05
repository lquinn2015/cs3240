use crate::console::kprintln;
use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    kprintln!("Welp thats a Panic!!");

    if let Some(s) = info.payload().downcast_ref::<&str>() {
        kprintln!("Panic payload: {:?}", s);
    }

    if let Some(loc) = info.location() {
        kprintln!("panic occured in {}, at line {}", loc.file(), loc.line());
    } else {
        kprintln!("Panic loc details not found");
    }

    if let Some(msg) = info.message() {
        kprintln!("");
        kprintln!("Panic message: {:?} ", msg);
    }

    loop {}
}
