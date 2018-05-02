
pub struct Usb<UM: UsbMem>  {
    _mem: UM,
    _device_descriptor: &'static[u8]
}

pub trait UsbMem {

}

impl<UM: UsbMem> Usb<UM> {
	pub fn new(mem: UM, device_descriptor: &'static [u8]) -> Usb<UM> {
		Usb{_mem: mem, _device_descriptor: device_descriptor}
	}

	pub fn interrupt_low_priority(&mut self) {

	}

	pub fn interrupt_high_priority(&mut self) {

	}
}
