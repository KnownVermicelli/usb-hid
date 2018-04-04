//! Blinks an LED
#![feature(used)]
#![deny(unsafe_code)]
#![deny(warnings)]
#![no_std]

extern crate cortex_m;
extern crate cortex_m_rt;
#[macro_use(block)]
extern crate nb;
extern crate stm32f103xx_hal as hal;

use hal::prelude::*;
use hal::stm32f103xx;
use hal::timer::Timer;

fn main() {
	let cp = cortex_m::Peripherals::take().unwrap();
	let dp = stm32f103xx::Peripherals::take().unwrap();

	let mut flash = dp.FLASH.constrain();
	let mut rcc = dp.RCC.constrain();

	// Try a different clock configuration
	let clocks = rcc.cfgr.freeze(&mut flash.acr);
	// let clocks = rcc.cfgr
	//     .sysclk(64.mhz())
	//     .pclk1(32.mhz())
	//     .freeze(&mut flash.acr);

	let mut gpioc = dp.GPIOC.split(&mut rcc.apb2);

	let mut led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);
	// Try a different timer (even SYST)
	let mut timer = Timer::syst(cp.SYST, 1.hz(), clocks);
	loop {
		block!(timer.wait()).unwrap();
		led.set_high();
		block!(timer.wait()).unwrap();
		led.set_low();
	}
}

#[link_section = ".vector_table.interrupts"]
#[used]
static INTERRUPTS: [extern "C" fn(); 240] = [default_handler; 240];

extern "C" fn default_handler() {
	let mut p = 120;
	p += 1;
	drop(p);
}
