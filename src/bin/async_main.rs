#![no_std]
#![no_main]

use alloc::boxed::Box;
use alloc::vec;
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_hal::{rmt::Rmt, time::RateExtU32};
use esp_hal::clock::CpuClock;
use esp_hal::gpio::{Pull, Level, Input, Output};
use log::{debug, info, error};
use esp_hal_smartled::{smartLedBuffer, SmartLedsAdapter};
use smart_leds::{
    brightness, gamma,
    hsv::{hsv2rgb, Hsv},
    SmartLedsWrite,
    RGB8,
};
use ws2812_spi::Ws2812;

extern crate alloc;

#[embassy_executor::task]
async fn blink_backlight(mut backlight_pin: Output<'static>) {
    loop {
        Timer::after(Duration::from_secs(1)).await;
        info!("Hello world!");
        backlight_pin.toggle();
    }
}

#[embassy_executor::task]
async fn watch_key(mut key_pin: Input<'static>) {
    loop {
        let mut del_var = 2000;

        key_pin.wait_for_rising_edge().await;
        info!("key press??");
        del_var = del_var - 300;
        // If updated delay value drops below 300 then reset it back to starting value
        if del_var < 500 {
            del_var = 2000;
        }
        info!("surpased delay value");
        // Pub
    }

}

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
    const LED_COUNT: usize = 6;
    let rmt_buffer = smartLedBuffer!(6);
    info!("created buffer??");
    let mut led = SmartLedsAdapter::new(rmt.channel0, led_pin, rmt_buffer);
    info!("created adapter");
    let mut color = Hsv {
        hue: 0,
        sat: 255,
        val: 255,
    };

    let mut data: [RGB8; LED_COUNT] = [(0, 0, 0).into(); LED_COUNT];

    //let _ = spawner;
    let res = spawner.spawn(blink_backlight(backlight));
    match res {
        Ok(_) => info!("spawned successfully"),
        Err(error) => error!("Error spawning task: {error}"),
    }

    let mut key1 = Input::new(peripherals.GPIO14, Pull::Up);
    let res = spawner.spawn(watch_key(key1));
    match res {
        Ok(_) => info!("spawned successfully"),
        Err(error) => error!("Error spawning task: {error}"),
    }
    let mut key2 = Input::new(peripherals.GPIO21, Pull::Up);
    let res = spawner.spawn(watch_key(key1));
    match res {
        Ok(_) => info!("spawned successfully"),
        Err(error) => error!("Error spawning task: {error}"),
    }

    loop {
        //info!("looping");
        for hue in 0..=255 {
            // Convert from the HSV color space (where we can easily transition from one
            // color to the other) to the RGB color space that we can then send to the LED
            // When sending to the LED, we do a gamma correction first (see smart_leds
            // documentation for details) and then limit the brightness to 10 out of 255 so
            // that the output it's not too bright.
            //info!("writing to led");
            for i in 0..LED_COUNT {
                //info!("hue: {hue:#?}");
                color.hue = hue * (i as u8);
                data[i] = hsv2rgb(color);
            }
            match led.write(brightness(gamma(data.iter().cloned()), 100)) {
                Ok(_) => {
                    debug!("write success")
                },
                Err(error) => {
                    error!("error: {error:#?}");
                }
            }

            //info!("wrote to led");
            Timer::after(Duration::from_millis(20)).await;
        }

    }

    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/v0.23.1/examples/src/bin
}
