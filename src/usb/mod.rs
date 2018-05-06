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

	fn get_configuration(&self) -> u8;

	fn set_address(&mut self, u8);
	fn set_configuration(&mut self, u8);
	fn confirm_request(&mut self);
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
			// device level requests
			(0x80, Request::GetStatus) => {
				// Get status of device should return two bytes:
				// D0 - set if selfpowered
				// D1 - set if remote wakeup (can wakeup host during suspend)
				// D2-D15 - reserved.

				// TODO: respond with status.
			}
			(0x00, Request::ClearFeature) => {
				// Clear feature
				// value field hold feature selector

				// TODO: do something here
				ud.confirm_request();
			}
			(0x00, Request::SetFeature) => {
				// Set feature
				// value as above.

				// TODO: do something here
				ud.confirm_request();
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
				ud.confirm_request();
			}
			(0x80, Request::GetDescriptor) => {
				// Request for device descriptor
				// It is specified in value field.
				// index field - Zero or languageID,
				// length field - descriptor length
			}
			(0x00, Request::SetDescriptor) => {
				// No idea what this request is supposed to do :)

				// TODO: do something here
				ud.confirm_request();
			}
			(0x80, Request::GetConfiguration) => {
				// Requests current configuration.
				// More important for devices with many configurations.
				// Not configured device should response with 0u8.
				// Configuration id otherwise.

				// TODO: respond with current configuration
			}
			(0x00, Request::SetConfiguration) => {
				// As above - more important for multiple-configurations devices.
				// Sets configuration.
				ud.set_configuration(ud.get_value() as u8);
				ud.confirm_request();
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
				let response: [u8; 2] = [0, 0];
				ud.set_response(&response);
			}
			(0x01, Request::ClearFeature) => {
				// As above - basically not used.
				ud.confirm_request();
			}
			(0x81, Request::GetInterface) => {
				let response: u8 = 0;
				// for now responsing with interface0
				ud.set_response(&[response]);
			}
			(0x01, Request::SetInterface) => {
				// as above
				// more info: https://beyondlogic.org/usbnutshell/usb5.shtml#AlternateSetting
				ud.confirm_request();
			}

			// endpoint level requests
			(a, b) => panic!("Unknown request: {:?}, {:?}", a, b),
		}
	}
}
