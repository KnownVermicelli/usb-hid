mod requests;
pub struct Usb {
	_device_descriptor: &'static [u8],
}

pub trait UsbDevice {
	fn set_response(&mut self, &[u8]);

	// 5 methods below are meant to query received request information.
	// These things are part of each usb request as per specification (total 8 bytes).
	// More information: https://beyondlogic.org/usbnutshell/usb6.shtml
	fn get_request_type(&self) -> u8;
	fn get_request(&self) -> u8;
	fn get_value(&self) -> u16;
	fn get_index(&self) -> u16;
	fn get_length(&self) -> u16;

	fn set_address(&mut self, u8);
}

impl Usb {
	pub fn new(device_descriptor: &'static [u8]) -> Usb {
		Usb {
			_device_descriptor: device_descriptor,
		}
	}

	pub fn interrupt_low_priority<UD: UsbDevice>(&mut self, ud: &mut UD) {
		self.check_request(ud);
	}

	pub fn interrupt_high_priority<UD: UsbDevice>(&mut self, ud: &mut UD) {
		self.check_request(ud);
	}

	fn check_request<UD: UsbDevice>(&mut self, ud: &mut UD) {
		use self::requests::Request;

		let request_type = ud.get_request_type();
		let request = requests::Request::from_u8(ud.get_request());

		// all request_type values below and their association with requests
		// specified by USB specs. See: link at the top of this file.

		// Not sure if true but looks like all requests that expect return data
		// has request_type value as 0x8Z (Z is anything).
		match (request_type, request) {
			(0x80, Request::GetStatus) => {
				// Get status of device should return two bytes:
				// D0 - set if selfpowered
				// D1 - set if remote wakeup (can wakeup host during suspend)
				// D2-D15 - reserved.
			}
			(0x00, Request::ClearFeature) => {
				// Clear feature
				// value field hold feature selector
			}
			(0x00, Request::SetFeature) => {
				// Set feature
				// value as above.
			}
			(0x00, Request::SetAddress) => {
				// Request sent during initial phase.
				// This is address assigned to our device.
				// Should be forwarded to device.

				// Code below should be moved to different place and address temporary
				// written somewhere. Spec says that address should be set AFTER
				// completion of the status stage.
				// Probable source: https://beyondlogic.org/usbnutshell/usb4.shtml#Control

				// let address = ud.get_value() as u8;
				// ud.set_address(address);
			}
			(0x80, Request::GetDescriptor) => {}
			(0x00, Request::SetDescriptor) => {}
			(0x80, Request::GetConfiguration) => {}
			(0x00, Request::SetConfiguration) => {}
			(a, b) => panic!("Unknown request: {:?}, {:?}", a, b),
		}
	}
}
