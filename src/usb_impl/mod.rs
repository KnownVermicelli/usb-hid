///
/// Completely platform specific implementation.
/// If you use other stm32 then address/gaps may differ.
/// If you use something completely different then there may be no point in looking at this code.
/// 
use bare_metal::Peripheral;
use vcell::VolatileCell;
use super::usb;
use hal::stm32f103xx;

#[allow(unsafe_code)]
const PMA: Peripheral<PmaMemory> = unsafe {Peripheral::new(0x4000_6000)};

#[repr(C)]
struct PmaMemory{
    // there is actually only 256 fields, but are separated with gaps in the same size...
    fields: [VolatileCell<u16>; 512]
}


pub struct UsbMem {
	_mem: &'static mut PmaMemory
}

impl usb::UsbMem for UsbMem {

}

impl UsbMem {
    pub fn new() -> UsbMem {
	#[allow(unsafe_code)]        
        let mem = unsafe { &mut *PMA.get() };
        UsbMem{_mem: mem}
    }
}

pub fn enable_usb(rcc: &mut stm32f103xx::RCC) {
	// Enable USB module
	rcc.apb1enr.modify(|_, w| w.usben().enabled());

	// Reset USB module
	rcc.apb1rstr.modify(|_, w| w.usbrst().set_bit());
	rcc.apb1rstr.modify(|_, w| w.usbrst().clear_bit());

	// have to put 1.5k resistor on d+ do 3.3v : https://community.st.com/thread/41267-stm32f103-usb-circuit
}
