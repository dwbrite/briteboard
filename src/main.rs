#![feature(asm_const)]

#![no_std]
#![no_main]

use cortex_m::delay::Delay;
use embedded_graphics::prelude::*;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::primitives::{PrimitiveStyleBuilder, StrokeAlignment};
use teensy4_bsp as bsp;
use teensy4_panic as _;
use smart_leds::{RGB8, SmartLedsWrite};


use teensy4_bsp::hal::iomuxc;

use crate::iomuxc::{Config, configure, SlewRate, Speed};


use ssd1351::prelude::SSD1351_SPI_MODE;
use ssd1351::properties::DisplayRotation;
use ssd1351::properties::DisplaySize::Display128x128;
use teensy4_bsp::hal::ccm::spi::{ClockSelect, PrescalarSelect};
use embedded_hal::blocking::spi::{Transfer, Write};
use embedded_hal::digital::v2::OutputPin;
use ssd1351::builder::Builder;
use ssd1351::mode::{GraphicsMode, RawMode};

use teensy4_bsp::hal::gpio::GPIO;
use teensy4_bsp::hal::spi::ClockSpeed;

use teensy4_bsp::pins::imxrt_iomuxc::DriveStrength;


mod logging;
mod systick;

// uses pin 2 as the ws2812 output
// pin 13 to blink the LED at the same time

#[cortex_m_rt::entry]
fn main() -> ! {
    let mut periphs = bsp::Peripherals::take().unwrap();
    let mut pins = bsp::pins::t40::from_pads(periphs.iomuxc);

    let mut systick = systick::new(cortex_m::Peripherals::take().unwrap().SYST);

    // let mut led = bsp::configure_led(pins.p13);
    // led.set();

    assert!(logging::init().is_ok());
    systick.delay_ms(2000);

    // Prepare the ARM clock to run at ARM_HZ.
    let (_, _ipg) = periphs.ccm.pll1.set_arm_clock(
        bsp::hal::ccm::PLL1::ARM_HZ, // we can overclock to 816_000_000 without cooling
        &mut periphs.ccm.handle,
        &mut periphs.dcdc,
    );

    let mut data: [RGB8; 3] = [RGB8::default(); 3];
    let empty: [RGB8; 3] = [RGB8::default(); 3];

    data[0] = RGB8 {
        r: 0x10,
        g: 0x00,
        b: 0x00,
    };
    data[1] = RGB8 {
        r: 0x00,
        g: 0x10,
        b: 0x00,
    };
    data[2] = RGB8 {
        r: 0x00,
        g: 0x00,
        b: 0x10,
    };

    configure(&mut pins.p2,{
        Config::zero()
            .set_drive_strength(DriveStrength::R0_7)
            .set_speed(Speed::Max)
            .set_slew_rate(SlewRate::Fast)
    });
    let mut pin = GPIO::new(pins.p2).output();
    pin.set_fast(true);

    let mut ws = ws2812_nop_imxrt1062::Ws2812::new(pin);

    let (_, _, _, mut one_builder) = periphs.spi.clock(&mut periphs.ccm.handle, ClockSelect::Pll2, PrescalarSelect::LPSPI_PODF_2);
    let mut spi = one_builder.build(pins.p11, pins.p12, pins.p13);
    spi.set_clock_speed(ClockSpeed(24_000_000)); // 24MHz
    spi.enable_chip_select_0(pins.p10);

    systick.delay_ms(100);

    // RESET
    configure(&mut pins.p8, Config::zero().set_slew_rate(SlewRate::Fast).set_speed(Speed::Max).set_drive_strength(DriveStrength::R0_7));
    let mut reset = GPIO::new(pins.p8).output();
    reset.set_fast(true);

    // DC
    configure(&mut pins.p9, Config::zero().set_slew_rate(SlewRate::Fast).set_speed(Speed::Max).set_drive_strength(DriveStrength::R0_7));
    let mut dc = GPIO::new(pins.p9).output();
    dc.set_fast(true);

    let mut display: GraphicsMode<_> = Builder::new().connect_spi(spi, dc).into();
    display.reset(&mut reset, &mut systick);
    display.init();
    display.clear();

    loop {

        display.set_pixel(30, 30, 0xFFFF);
        ws.write(data.iter().cloned()).unwrap();
        systick.delay_ms(500);

        display.clear();
        ws.write(empty.iter().cloned()).unwrap();
        systick.delay_ms(500);
    }
}

