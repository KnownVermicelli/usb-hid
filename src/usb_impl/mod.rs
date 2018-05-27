use super::usb::UsbDevice;
///
/// Completely platform specific implementation.
/// If you use other stm32 then address/gaps may differ.
/// If you use something completely different then there may be no point in looking at this code.
///
use bare_metal::Peripheral;
use hal::stm32f103xx;
use vcell::VolatileCell;

#[allow(unsafe_code)]
const PMA: Peripheral<PmaMemory> = unsafe { Peripheral::new(0x4000_6000) };

#[repr(C)]
struct PmaMemory {
	// there is actually only 256 fields, but are separated with gaps in the same size...
	fields: [VolatileCell<u16>; 512],
}

pub struct Stm32UsbDevice {
	_mem: &'static mut PmaMemory,
}

impl UsbDevice for Stm32UsbDevice {
	fn set_response(&mut self, _response: &[u8]) {}

	fn get_request_type(&self) -> u8 {
		1
	}
	fn get_request(&self) -> u8 {
		1
	}
	fn get_value(&self) -> u16 {
		1
	}
	fn get_index(&self) -> u16 {
		1
	}
	fn get_length(&self) -> u16 {
		1
	}

	fn set_address(&mut self, _address: u8) {}

	fn get_configuration(&self) -> u8 {
		0
	}
	fn set_configuration(&mut self, _configuration: u8) {}
	fn confirm_request(&mut self) {}
}

impl Stm32UsbDevice {
	pub fn new() -> Stm32UsbDevice {
		#[allow(unsafe_code)]
		let mem = unsafe { &mut *PMA.get() };
		Stm32UsbDevice { _mem: mem }
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
