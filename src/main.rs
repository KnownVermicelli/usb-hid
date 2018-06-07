#![deny(unsafe_code)]
#![deny(warnings)]
#![feature(proc_macro)]
#![feature(used)]
#![no_std]
#![allow(non_camel_case_types)]

extern crate bare_metal;
extern crate cortex_m;
extern crate cortex_m_rtfm as rtfm;
extern crate panic_abort;
extern crate stm32f103xx_hal as hal;
extern crate vcell;

mod descriptors;
mod usb;
mod usb_impl;

use hal::prelude::*;
use hal::stm32f103xx;

use cortex_m::peripheral::syst::SystClkSource;
use rtfm::{app, Threshold};

use usb::Usb;
use usb_impl::enable_usb;
use usb_impl::Stm32UsbDevice;

app! {
    device: stm32f103xx,

    // Here data resources are declared
    //
    // Data resources are static variables that are safe to share across tasks
    resources: {

        static ON: bool = false;
        // this is considered LateResource - there is no initial value. It will be set in init function
        static USB: Usb<Stm32UsbDevice>;
    },

    // Tasks corresponding to hardware interrupts
    // See stm32f103xx::Interrupts
    tasks: {
        // Here we declare that we'll use the SYS_TICK exception as a task
        SYS_TICK: {
            path: sys_tick,
            resources: [ON],
        },
        // Interrupt with wrong name in stm32f103xx crate
        // Interrupt for both can_tx and usb_high_priority.
        CAN1_TX: {
            path: usb_high_priority_interrupt,
            resources: [USB],
        },
        // Interrupt with wrong name in stm32f103xx crate
        // Interrupt for both can_rx and usb_low_priority.
        CAN1_RX0: {
            path: usb_low_priority_interrupt,
            resources: [USB],
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

    // enable usb interface. This is completely platform specific.
    enable_usb(&mut rc);

    let mut rcc = rc.constrain();

    let mut gpioc = p.device.GPIOC.split(&mut rcc.apb2);

    let mut dev_usb = p.device.USB;

    dev_usb.cntr.modify(|_, w| w.pdwn().set_bit());

    let mut led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);
    led.set_high();

    let usb = Usb::new(
        &descriptors::DEVICE_DESCRIPTOR,
        Stm32UsbDevice::new(dev_usb, led),
    );
    init::LateResources { USB: usb }
}

fn idle() -> ! {
    loop {
        rtfm::wfi();
    }
}

fn sys_tick(_t: &mut Threshold, mut r: SYS_TICK::Resources) {
    // toggle state
    *r.ON = !*r.ON;

    // if *r.ON {
    //     r.LED.set_high();
    // } else {
    //     r.LED.set_low();
    // }
}

fn usb_high_priority_interrupt(_t: &mut Threshold, mut r: CAN1_TX::Resources) {
    r.USB.interrupt_high_priority();
}

fn usb_low_priority_interrupt(_t: &mut Threshold, mut r: CAN1_RX0::Resources) {
    r.USB.interrupt_low_priority();
}
