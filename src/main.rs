//! The starter code slowly blinks the LED, and sets up
//! USB logging.

#![no_std]
#![no_main]

use teensy4_bsp as bsp;
use teensy4_panic as _;
use core::time::Duration;
use embedded_hal::timer::CountDown;
use smart_leds::{RGB8, SmartLedsWrite};
use teensy4_bsp::hal::gpio::GPIO;
use teensy4_bsp::hal::iomuxc;
use crate::iomuxc::{Config};

mod logging;
mod systick;

// uses pin 2 as the ws2812 output
// pin 13 to blink the LED at the same time

#[cortex_m_rt::entry]
fn main() -> ! {
    let mut periphs = bsp::Peripherals::take().unwrap();
    let mut pins = bsp::pins::t40::from_pads(periphs.iomuxc);

    let mut led = bsp::configure_led(pins.p13);
    led.set();

    // See the `logging` module docs for more info.
    assert!(logging::init().is_ok());

    log::info!("o hej");

    // Prepare the ARM clock to run at ARM_HZ.
    let (_, _ipg) = periphs.ccm.pll1.set_arm_clock(
        bsp::hal::ccm::PLL1::ARM_HZ, // we can overclock to 816_000_000 without cooling
        &mut periphs.ccm.handle,
        &mut periphs.dcdc,
    );

    let mut systick = systick::new(cortex_m::Peripherals::take().unwrap().SYST);

    systick.delay_ms(2000);
    log::info!("configuring 3MHz peripheral clock...");

    let mut clock_cfg = periphs.ccm.perclk.configure(
        // Divide 24MHz osc clock by 8 to get 3MHz
        &mut periphs.ccm.handle,
        bsp::hal::ccm::perclk::PODF::DIVIDE_8,
        bsp::hal::ccm::perclk::CLKSEL::OSC,
    );

    let (_, _, _, mut timer) = periphs.pit.clock(&mut clock_cfg);
    log::info!("{:?}", timer.clock_period());
    timer.start(Duration::from_nanos(333));

    iomuxc::configure(&mut pins.p2, Config::zero());
    let mut pin = GPIO::new(pins.p2).output();
    pin.set_fast(true);

    log::info!("Hello world2 {:?}", timer.clock_period());
    let mut ws = ws2812_timer_delay::Ws2812::new(timer, pin);

    let mut data: [RGB8; 3] = [RGB8::default(); 3];
    let empty: [RGB8; 3] = [RGB8::default(); 3];

    data[0] = RGB8 {
        r: 0,
        g: 0,
        b: 0x10,
    };
    data[1] = RGB8 {
        r: 0,
        g: 0x10,
        b: 0,
    };
    data[2] = RGB8 {
        r: 0x10,
        g: 0,
        b: 0,
    };

    loop {
        led.toggle();
        ws.write(data.iter().cloned()).unwrap();
        log::info!("Hello world2");
        systick.delay_ms(500);

        ws.write(empty.iter().cloned()).unwrap();
        log::info!("Hello world3");
        systick.delay_ms(500);
    }
}
