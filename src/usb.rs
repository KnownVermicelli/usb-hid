use hal::stm32f103xx;

pub fn enable_usb(rcc: &mut stm32f103xx::RCC) {
	// Enable USB module
	rcc.apb1enr.modify(|_, w| w.usben().enabled());

	// Reset USB module
	rcc.apb1rstr.modify(|_, w| w.usbrst().set_bit());
	rcc.apb1rstr.modify(|_, w| w.usbrst().clear_bit());

	// have to put 1.5k resistor on d+ do 3.3v : https://community.st.com/thread/41267-stm32f103-usb-circuit
}
