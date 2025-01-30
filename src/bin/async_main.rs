#![no_std]
#![no_main]

use alloc::boxed::Box;
use alloc::vec;
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_hal::{rmt::Rmt, time::RateExtU32};
use esp_hal::clock::CpuClock;
use esp_hal::gpio::{Level, Output};
use log::{debug, info, error};
use esp_hal_smartled::{smartLedBuffer, SmartLedsAdapter};
use smart_leds::{
    brightness, gamma,
    hsv::{hsv2rgb, Hsv},
    SmartLedsWrite,
};
use ws2812_spi::Ws2812;

extern crate alloc;

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    // generator version: 0.2.2

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(72 * 1024);

    esp_println::logger::init_logger_from_env();

    let timer0 = esp_hal::timer::systimer::SystemTimer::new(peripherals.SYSTIMER);
    esp_hal_embassy::init(timer0.alarm0);

    info!("Embassy initialized!");

    let timer1 = esp_hal::timer::timg::TimerGroup::new(peripherals.TIMG0);
    info!("initing wifi??");
    let _init = esp_wifi::init(
        timer1.timer0,
        esp_hal::rng::Rng::new(peripherals.RNG),
        peripherals.RADIO_CLK,
    )
    .unwrap();
    info!("inited wifi??");

    let mut backlight = Output::new(peripherals.GPIO46, Level::Low);
    backlight.set_high();
    // Initialize the Delay peripheral, and use it to toggle the LED state in a
    // loop.
    //

    let led_pin = peripherals.GPIO16;
    let freq = 80.MHz();
    let rmt = Rmt::new(peripherals.RMT, freq).unwrap();

    info!("creating buffer??");
    let rmt_buffer = smartLedBuffer!(2);
    info!("created buffer??");
    let mut led = SmartLedsAdapter::new(rmt.channel0, led_pin, rmt_buffer);
    info!("created adapter");
    let mut color = Hsv {
        hue: 0,
        sat: 255,
        val: 255,
    };


    // TODO: Spawn some tasks
    let mut data;
    let _ = spawner;
    loop {
        info!("looping");
        for hue in 0..=255 {
            info!("hue: {hue:#?}");
            color.hue = hue;
            // Convert from the HSV color space (where we can easily transition from one
            // color to the other) to the RGB color space that we can then send to the LED
            data = [hsv2rgb(color)];
            // When sending to the LED, we do a gamma correction first (see smart_leds
            // documentation for details) and then limit the brightness to 10 out of 255 so
            // that the output it's not too bright.
            info!("writing to led");
            match led.write(brightness(gamma(data.iter().cloned()), 255)) {
                Ok(_) => {
                    debug!("write success")
                },
                Err(error) => {
                    error!("error: {error:#?}");
                }
            }
            info!("wrote to led");
            Timer::after(Duration::from_millis(20)).await;
        }

        info!("Hello world!");
        backlight.toggle();
        Timer::after(Duration::from_secs(1)).await;
    }

    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/v0.23.1/examples/src/bin
}
