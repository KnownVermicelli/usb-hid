#![deny(unsafe_code)]
#![deny(warnings)]
#![feature(proc_macro)]
#![feature(used)]
#![no_std]

extern crate cortex_m;
extern crate cortex_m_rtfm as rtfm;
extern crate stm32f103xx_hal as hal;

use hal::prelude::*;
use hal::stm32f103xx;

use cortex_m::peripheral::syst::SystClkSource;
use rtfm::{app, Threshold};

app! {
	device: stm32f103xx,

	// Here data resources are declared
	//
	// Data resources are static variables that are safe to share across tasks
	resources: {
		static ON: bool = false;
		static LED: hal::gpio::gpioc::PC13<hal::gpio::Output<hal::gpio::PushPull>>;
	},

	// Tasks corresponding to hardware interrupts
	// See stm32f103xx::Interrupts
	tasks: {
		// Here we declare that we'll use the SYS_TICK exception as a task
		SYS_TICK: {
			path: sys_tick,
			resources: [ON, LED],
		},
		CAN1_TX: {
			path: usb_high_priority_interrupt,
			resources: [ON],
		},
		CAN1_RX0: {
			path: usb_low_priority_interrupt,
			resources: [ON],
		},
	}
}

fn init(mut p: init::Peripherals, r: init::Resources) -> init::LateResources {
	// `init` can modify all the `resources` declared in `app!`
	r.ON;

	// power on GPIOC
	p.device.RCC.apb2enr.modify(|_, w| w.iopcen().enabled());

	// configure PC13 as output
	p.device.GPIOC.bsrr.write(|w| w.bs13().set());
	p.device
		.GPIOC
		.crh
		.modify(|_, w| w.mode13().output().cnf13().push());

	// configure the system timer to generate one interrupt every second
	p.core.SYST.set_clock_source(SystClkSource::Core);
	p.core.SYST.set_reload(8_000_000); // 1s
	p.core.SYST.enable_interrupt();
	p.core.SYST.enable_counter();

	let mut rcc = p.device.RCC.constrain();

	let mut gpioc = p.device.GPIOC.split(&mut rcc.apb2);

	let led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);

	init::LateResources { LED: led }
}

fn idle() -> ! {
	loop {
		rtfm::wfi();
	}
}

// This is the task handler of the SYS_TICK exception
//
// `_t` is the preemption threshold token. We won't use it in this program.
//
// `r` is the set of resources this task has access to. `SYS_TICK::Resources`
// has one field per resource declared in `app!`.
#[allow(unsafe_code)]
fn sys_tick(_t: &mut Threshold, mut r: SYS_TICK::Resources) {
	// toggle state
	*r.ON = !*r.ON;

	if *r.ON {
		r.LED.set_high();
	} else {
		r.LED.set_low();
	}
}

fn usb_high_priority_interrupt(_t: &mut Threshold, mut _r: CAN1_TX::Resources) {}

fn usb_low_priority_interrupt(_t: &mut Threshold, mut _r: CAN1_RX0::Resources) {}
