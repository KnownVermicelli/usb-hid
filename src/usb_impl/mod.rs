use super::hal;
use super::usb::UsbDevice;
///
/// Completely platform specific implementation.
/// If you use other stm32 then address/gaps may differ.
/// If you use something completely different then there may be no point in looking at this code.
///
/// stm32f103 usb pma memory layout (super confusing):
/// http://kevincuzner.com/2018/01/29/bare-metal-stm32-writing-a-usb-driver/#pma-stm32f103
///
use bare_metal::Peripheral;
use core::marker::Send;
use hal::prelude::*;
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
	fields: [VolatileCell<u32>; 256],
}

#[allow(unsafe_code)]
unsafe impl Send for PmaMemory {}

impl PmaMemory {
	fn set_memory(&self, index: usize, value: u16) {
		assert!(index < 256);
		let previous_value = self.fields[index].get();
		let new_value = (previous_value & 0xffff_0000) | value as u32;
		self.fields[index].set(new_value);
	}

	fn get_memory(&self, index: usize) -> u16 {
		assert!(index < 256);
		(self.fields[index].get() & 0x0000_ffff) as u16
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
	#[allow(unused)]
	led: hal::gpio::gpioc::PC13<hal::gpio::Output<hal::gpio::PushPull>>,
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

const MAX_PACKET_SIZE: u32 = 64;

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
		// let lenght_of_response = response.len() as u16;
		// write length of response into response_length buffer area.

		self.set_mem(1, 5); // lenght_of_response);
		self.set_mem(2, 5); // lenght_of_response);

		self.led.set_low();
	}

	fn should_reset(&self) -> bool {
		//
		self.usb.istr.read().reset().bit_is_set()
	}

	fn clear_reset(&mut self) {
		self.usb.istr.modify(|_, w| w.ctr().clear_bit());
	}

	fn clear_transfer_flags(&mut self) {
		self.usb
			.istr
			.modify(|_, w| w.susp().clear_bit().sof().clear_bit().esof().clear_bit());
	}

	fn connection_type(&self) -> bool {
		self.usb.istr.read().dir().bit_is_set()
	}
	fn get_endpoint(&self) -> u8 {
		self.usb.istr.read().ep_id().bits()
	}

	fn transfer_correct(&self) -> bool {
		self.usb.istr.read().ctr().bit_is_set()
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

	fn set_address(&mut self, address: u8) {
		#[allow(unsafe_code)]
		self.usb
			.daddr
			.modify(|_, w| unsafe { w.add().bits(address) });
	}

	fn get_configuration(&self) -> u8 {
		0
	}

	fn set_configuration(&mut self, _configuration: u8) {}

	fn confirm_request(&mut self) {
		self.toggle_tx_stall();
	}

	fn confirm_tx(&mut self) {
		self.toggle_tx_out();
	}

	fn confirm_response(&mut self) {
		self.toggle_out();
	}

	fn reset_state(&mut self) {
		// // clear pma area
		let area = self.deref();
		area.set_memory(0, 0x40);
		area.set_memory(1, 0x0);
		area.set_memory(2, 0x20);

		area.set_memory(3, (0x8000 | ((MAX_PACKET_SIZE / 32) - 1) << 10) as u16);

		area.set_memory(4, 0x100);
		area.set_memory(5, 0x0);

		#[allow(unsafe_code)]
		self.usb.ep0r.modify(|_, w| unsafe {
			w.ep_type()
				.bits(0b01)
				.stat_tx()
				.bits(0b10)
				.stat_rx()
				.bits(0b11)
		});

		#[allow(unsafe_code)]
		self.usb.ep1r.modify(|_, w| unsafe {
			w.ep_type()
				.bits(0b11)
				.stat_tx()
				.bits(0b11)
				.stat_rx()
				.bits(0b10)
				.ea()
				.bits(0b1)
		});

		self.usb.daddr.write(|w| w.ef().set_bit());

		// self.device_state = UsbDeviceState::Default;
		// reset endpoints registers
	}
}

impl Stm32UsbDevice {
	pub fn new(
		usb: USB,
		led: hal::gpio::gpioc::PC13<hal::gpio::Output<hal::gpio::PushPull>>,
	) -> Stm32UsbDevice {
		#[allow(unsafe_code)]
		let mem = creat_pma_memory();
		Stm32UsbDevice { mem, usb, led }
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
