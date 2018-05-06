// USB device descriptor. Basically similar for all devices.
// greatly written about at https://beyondlogic.org/usbnutshell/usb5.shtml
// all 2 u8s wide fields are backwards (lessImportant and moreImportant)
pub const DEVICE_DESCRIPTOR: [u8; 18] = [
    18,   // Length of this descriptor
    0x01, // Type of this descriptor
    0x00,
    0x02, // usb version 2.00
    0x00, // Device class - 0 means read from interface descriptor
    0x00, // bDeviceSubClass
    0x00, // bDeviceProtocol
    0x40, // 64  - maximum packet size. Potentially device-specific
    0x6B,
    0x1D, // VendorID - 1d6b is Linux Foundation VendorID,
    0x04,
    0x01, // ProductID - 0012; they are not using this productId
    0x01,
    0x00, // Device release number - 0.01
    0x01, // Manufacturer string index
    0x02, // Product string index
    0x03, // SerialNumber string index
    0x00, // Number of configurations
          // last one should be 0x01, temporary changed. TODO: rollback once supported
];
