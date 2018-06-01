// USB device descriptor. Basically similar for all devices.
// greatly written about at https://beyondlogic.org/usbnutshell/usb5.shtml
// all 2 u8s wide fields are backwards (lessImportant and moreImportant)
pub const DEVICE_DESCRIPTOR: [u16; 9] = [
    0x0118, // Type of this descriptor
    0x0200, // usb version 2.00
    0x0000, // Device class - 0 means read from interface descriptor
    0x4000, // 64  - maximum packet size. Potentially device-specific
    0x1D6B, // VendorID - 1d6b is Linux Foundation VendorID,
    0x0012, // ProductID - 0012; they are not using this productId
    0x0001, // Device release number - 0.01
    0x0201, // Product string index and manufacturer string index
    0x0003, // SerialNumber string index and Number of configurations
            // last one should be 0x01, temporary changed. TODO: rollback once supported
];
