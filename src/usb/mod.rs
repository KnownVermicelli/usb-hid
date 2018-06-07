mod requests;

pub trait UsbDevice {
	fn set_response(&mut self, &[u16]);

	fn should_reset(&self) -> bool;
	fn clear_reset(&mut self);

	fn clear_transfer_flags(&mut self);

	fn get_endpoint(&self) -> u8;

	fn transfer_correct(&self) -> bool;

	fn connection_type(&self) -> bool;
	// 5 methods below are meant to query received request information.
	// These things are part of each usb request as per specification (total 8 bytes).
	// More information: https://beyondlogic.org/usbnutshell/usb6.shtml
	fn get_request_type(&self) -> u8;
	fn get_request(&self) -> u8;
	fn get_value(&self) -> u16;
	fn get_index(&self) -> u16;
	fn get_length(&self) -> u16;

	fn get_configuration(&self) -> u8;

	fn set_address(&mut self, u8);
	fn set_configuration(&mut self, u8);
	fn confirm_request(&mut self);
	fn confirm_response(&mut self);
	fn confirm_tx(&mut self);

	fn reset_state(&mut self);
}

// pub const MANUFACTURER_STR: [u16; 19] = [
// 	0x0326, 0x0052, 0x75, 0x73, 0x74, 0x79, 0x20, 0x4d, 0x61, 0x6e, 0x75, 0x66, 0x61, 0x63, 0x74,
// 	0x75, 0x72, 0x65, 0x72,
// ];

pub struct Usb<UD: UsbDevice> {
	_device_descriptor: &'static [u16],
	ud: UD,
	pending_address: u8,
}

impl<UD: UsbDevice> Usb<UD> {
	pub fn new(device_descriptor: &'static [u16], ud: UD) -> Usb<UD> {
		Usb {
			_device_descriptor: device_descriptor,
			ud: ud,
			pending_address: 0,
		}
	}

	pub fn interrupt_low_priority(&mut self) {
		self.check_interrupt();
	}

	pub fn interrupt_high_priority(&mut self) {
		self.check_interrupt();
	}

	fn check_interrupt(&mut self) {
		self.ud.set_response(&self._device_descriptor);
		if self.ud.should_reset() {
			self.ud.reset_state();
			self.ud.clear_reset();
		}
		self.ud.clear_transfer_flags();
		if self.ud.transfer_correct() {
			let endpoint = self.ud.get_endpoint();
			match endpoint {
				0 => self.control_connection(),
				_ => self.check_user_request(),
			}
		}
	}

	fn check_user_request(&mut self) {
		//
	}

	fn control_connection(&mut self) {
		if self.ud.connection_type() {
			self.check_request()
		} else {
			if self.pending_address != 0 {
				self.ud.set_address(self.pending_address);
				self.pending_address = 0;
			} else {
				//whatever
			}
			self.ud.confirm_tx();
		}
	}
	fn check_request(&mut self) {
		use self::requests::Request;

		let request_type = self.ud.get_request_type();
		let request = requests::Request::from_u8(self.ud.get_request());

		// all request_type values below and their association with requests
		// specified by USB specs. See: link at the top of this file.

		// Not sure if true but looks like all requests that expect return data
		// has request_type value as 0x8Z (Z is anything).
		match (request_type, request) {
			// device level requests
			(0x80, Request::GetStatus) => {
				// Get status of device should return two bytes:
				// D0 - set if selfpowered
				// D1 - set if remote wakeup (can wakeup host during suspend)
				// D2-D15 - reserved.

				self.ud.set_response(&[0]);
				// TODO: respond with status.
				self.ud.confirm_response();
			}
			(0x00, Request::ClearFeature) => {
				// Clear feature
				// value field hold feature selector

				// TODO: do something here
				self.ud.confirm_request();
			}
			(0x00, Request::SetFeature) => {
				// Set feature
				// value as above.

				// TODO: do something here
				self.ud.confirm_request();
			}
			(0x00, Request::SetAddress) => {
				// Request sent during initial phase.
				// This is address assigned to our device.
				// Should be forwarded to device.

				// Code below should be moved to different place and address temporary
				// written somewhere. Spec says that address should be set AFTER
				// completion of the status stage.
				// Probable source: https://beyondlogic.org/usbnutshell/usb4.shtml#Control

				let address = self.ud.get_value() as u8;
				self.pending_address = address;
				// self.ud.set_address(address);
				self.ud.confirm_request();
			}
			(0x80, Request::GetDescriptor) => {
				// Request for device descriptor
				// It is specified in value field.
				// index field - Zero or languageID,
				// length field - descriptor length
				let p = [0, 1, 2, 3];
				self.ud.set_response(&p);
				//self.ud.confirm_response();
			}
			(0x00, Request::SetDescriptor) => {
				// No idea what this request is supposed to do :)

				// TODO: do something here
				self.ud.confirm_request();
			}
			(0x80, Request::GetConfiguration) => {
				// Requests current configuration.
				// More important for devices with many configurations.
				// Not configured device should response with 0u8.
				// Configuration id otherwise.

				// TODO: respond with current configuration
				let conf = self.ud.get_configuration() as u16;
				self.ud.set_response(&[conf]);
				self.ud.confirm_response();
			}
			(0x00, Request::SetConfiguration) => {
				// As above - more important for multiple-configurations devices.
				// Sets configuration.
				let value = self.ud.get_value() as u8;
				self.ud.set_configuration(value);
				self.ud.confirm_request();
			}

			// Interface level requests
			// All interface level requests has written interface index
			// in index fields.
			// Index field:
			//     D8-D15: reserved
			//     D0-D7: interface number
			(0x81, Request::GetStatus) => {
				// Analogous to device level GetStatus
				// both bytes of response are reserved for future use.
				let response: [u16; 2] = [0, 0];
				self.ud.set_response(&response);
				self.ud.confirm_response();
			}
			(0x01, Request::ClearFeature) => {
				// As above - basically not used.
				self.ud.confirm_request();
			}
			(0x81, Request::GetInterface) => {
				let response: u16 = 0;
				// for now responsing with interface0
				self.ud.set_response(&[response]);
				self.ud.confirm_response();
			}
			(0x01, Request::SetInterface) => {
				// as above
				// more info: https://beyondlogic.org/usbnutshell/usb5.shtml#AlternateSetting
				self.ud.confirm_request();
			}

			// endpoint level requests
			(a, b) => panic!("Unknown request: {:?}, {:?}", a, b),
		}
	}
}
