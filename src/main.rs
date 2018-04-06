#![deny(unsafe_code)]
#![deny(warnings)]
#![feature(proc_macro)]
#![feature(used)]
#![no_std]

extern crate cortex_m;
extern crate cortex_m_rtfm as rtfm;
extern crate stm32f103xx_hal as hal;

mod usb;

use hal::prelude::*;
use hal::stm32f103xx;

use cortex_m::peripheral::syst::SystClkSource;
use rtfm::{app, Threshold};

use usb::enable_usb;

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
		// Interrupt with wrong name in stm32f103xx crate
		CAN1_TX: {
			path: usb_high_priority_interrupt,
			resources: [ON],
		},
		// Interrupt with wrong name in stm32f103xx crate
		CAN1_RX0: {
			path: usb_low_priority_interrupt,
			resources: [ON],
		},
	}
}

fn enable_system_clock_interrupt(system_timer: &mut stm32f103xx::SYST) {
	// configure the system timer to generate one interrupt every second
	system_timer.set_clock_source(SystClkSource::Core);
	system_timer.set_reload(8_000_000); // 1s
	system_timer.clear_current();
	system_timer.enable_interrupt();
	system_timer.enable_counter();
}

fn init(mut p: init::Peripherals, r: init::Resources) -> init::LateResources {
	// `init` can modify all the `resources` declared in `app!`
	r.ON;

	enable_system_clock_interrupt(&mut p.core.SYST);

	// Reset and Clock registers
	let mut rc = p.device.RCC;

	enable_usb(&mut rc);

	let mut rcc = rc.constrain();

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
