use super::usb::UsbDevice;
///
/// Completely platform specific implementation.
/// If you use other stm32 then address/gaps may differ.
/// If you use something completely different then there may be no point in looking at this code.
///
use bare_metal::Peripheral;
use core::marker::Send;
use hal::stm32f103xx;
use hal::stm32f103xx::USB;
use vcell::VolatileCell;

fn creat_pma_memory() -> Peripheral<PmaMemory> {
	#[allow(unsafe_code)]
	unsafe {
		Peripheral::new(0x4000_6000)
	}
}

#[repr(C)]
struct PmaMemory {
	// there is actually only 256 fields, but are separated with gaps in the same size...
	fields: [VolatileCell<u16>; 512],
}

#[allow(unsafe_code)]
unsafe impl Send for PmaMemory {}

impl PmaMemory {
	fn set_memory(&self, index: usize, value: u16) {
		assert!(index < 256);
		let real_index = index * 2;
		self.fields[real_index].set(value);
	}

	fn get_memory(&self, index: usize) -> u16 {
		assert!(index < 256);
		let real_index = index * 2;

		self.fields[real_index].get()
	}

	fn set_from_buffer(&self, start_index: usize, values: &[u16]) {
		let end_index = start_index + values.len();
		assert!(start_index < 256);
		assert!(end_index < 256);

		for (offset, value) in values.iter().enumerate() {
			self.set_memory(start_index + offset, *value);
		}
	}
}

pub struct Stm32UsbDevice {
	mem: Peripheral<PmaMemory>,
	usb: USB,
}
#[allow(unsafe_code)]
unsafe impl Send for Stm32UsbDevice {}

const EP_MASK: u32 = 0x0F0F;
const EP_TX_MASK: u32 = 0x0030;
const EP_RX_MASK: u32 = 0x3000;
const EP_TX_RX_MASK: u32 = (EP_TX_MASK | EP_RX_MASK);

const EP_TX_VALID: u32 = 0x0030;
const EP_RX_VALID: u32 = 0x3000;
const EP_TX_RX_VALID: u32 = (EP_TX_VALID | EP_RX_VALID);

const EP_TX_STALL: u32 = 0x0010;
const EP_STATUS_OUT: u32 = 0x0100;

impl Stm32UsbDevice {
	fn toggle(&self, mask: u32, val: u32, flags: u32) {
		#[allow(unsafe_code)]
		self.usb
			.ep0r
			.modify(|r, w| unsafe { w.bits(((r.bits() & (EP_MASK | mask)) ^ val) | flags) });
	}
	pub fn toggle_tx_stall(&self) {
		self.toggle(EP_TX_RX_MASK, EP_RX_VALID | EP_TX_STALL, 0)
	}
	#[allow(unused)]
	pub fn toggle_tx_out(&self) {
		self.toggle(EP_TX_MASK, EP_TX_VALID, EP_STATUS_OUT)
	}

	#[allow(unused)]
	pub fn toggle_out(&self) {
		self.toggle(EP_TX_RX_MASK, EP_TX_RX_VALID, EP_STATUS_OUT)
	}

	fn deref(&self) -> &PmaMemory {
		#[allow(unsafe_code)]
		unsafe {
			&*self.mem.get()
		}
	}
	fn get_mem(&self, index: usize) -> u16 {
		self.deref().get_memory(index)
	}

	fn set_mem(&mut self, index: usize, value: u16) {
		self.deref().set_memory(index, value);
	}

	fn set_from_buffer(&mut self, index: usize, buff: &[u16]) {
		self.deref().set_from_buffer(index, buff);
	}
}

impl UsbDevice for Stm32UsbDevice {
	fn set_response(&mut self, response: &[u16]) {
		// writing response to pma buffer
		self.set_from_buffer(0x20, response);
		let lenght_of_response = response.len() as u16;
		// write length of response into response_length buffer area.
		self.set_mem(0x1, lenght_of_response);
	}

	fn get_request_type(&self) -> u8 {
		(self.get_mem(0x10) & 0xff) as u8
	}

	fn get_request(&self) -> u8 {
		((self.get_mem(0x10) & 0xff00) >> 8) as u8
	}

	fn get_value(&self) -> u16 {
		self.get_mem(0x11)
	}

	fn get_index(&self) -> u16 {
		self.get_mem(0x12)
	}

	fn get_length(&self) -> u16 {
		self.get_mem(0x13)
	}

	fn set_address(&mut self, _address: u8) {}

	fn get_configuration(&self) -> u8 {
		0
	}

	fn set_configuration(&mut self, _configuration: u8) {}

	fn confirm_request(&mut self) {
		self.toggle_tx_stall();
	}
}

impl Stm32UsbDevice {
	pub fn new(usb: USB) -> Stm32UsbDevice {
		#[allow(unsafe_code)]
		let mem = creat_pma_memory();
		Stm32UsbDevice { mem, usb }
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
