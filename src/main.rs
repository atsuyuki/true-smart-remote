#![no_main]
#![no_std]

use cortex_m::delay::Delay;
use embedded_hal::digital::v2::{InputPin, OutputPin};
use panic_reset as _;
use rp2040_hal::{
    gpio::{bank0::Gpio16, Output, Pin, PushPull},
    Clock, Watchdog,
};

use embedded_hal::PwmPin;
use rp2040_hal::pwm::{FreeRunning, Slices};

use rp2040_hal::pac;

/// The linker will place this boot block at the start of our program image. We
/// need this to help the ROM bootloader get our code up and running.
#[link_section = ".boot2"]
#[used]
pub static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER_W25Q080;

const EXTERNAL_CRYSTAL_FREQUENCY_HZ: u32 = 12_000_000;

#[cortex_m_rt::entry]
fn main() -> ! {
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();

    let mut watchdog = Watchdog::new(pac.WATCHDOG);

    let clocks = rp2040_hal::clocks::init_clocks_and_plls(
        EXTERNAL_CRYSTAL_FREQUENCY_HZ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().0);

    let sio = rp2040_hal::Sio::new(pac.SIO);

    let pins =
        rp2040_hal::gpio::Pins::new(pac.IO_BANK0, pac.PADS_BANK0, sio.gpio_bank0, &mut pac.RESETS);

    let mut led_pin = pins.gpio25.into_push_pull_output();
    let mut power_pin = pins.gpio1.into_push_pull_output();

    // 諸々やる前に自己保持をかけておく
    led_pin.set_high().unwrap();
    power_pin.set_high().unwrap();

    let mut pwm_pin = pins.gpio17.into_push_pull_output();
    let mut ir_pin = pins.gpio16.into_push_pull_output();

//    pwm_pin.set_drive_strength(rp2040_hal::gpio::OutputDriveStrength::TwelveMilliAmps);
//    ir_pin.set_drive_strength(rp2040_hal::gpio::OutputDriveStrength::TwelveMilliAmps);

    let button_4 = pins.gpio11.into_pull_down_input();
    let button_5 = pins.gpio12.into_pull_down_input();
    let button_6 = pins.gpio13.into_pull_down_input();
    let button_1 = pins.gpio20.into_pull_down_input();
    let button_2 = pins.gpio19.into_pull_down_input();
    let button_3 = pins.gpio18.into_pull_down_input();

    // PWMの参考 : https://github.com/ac100v/pico-blink-pwm/blob/main/src/main.rs
    let pwm_slices = Slices::new(pac.PWM, &mut pac.RESETS);
    // RP2040のPWMは8つのスライスに分かれていて、各スライスが2つのピンに対応している。
    // GPIO17 はPWM0のチャネルB (RP2040 Datasheet 4.5.2)
    let mut pwm = pwm_slices.pwm0;
    pwm.set_ph_correct();
    pwm.enable();

    // PWMをフリーランモードで動作させる。
    // 125MHz / 137 / 12 / 2 = 38.017kHz
    let mut pwm = pwm.into_mode::<FreeRunning>();
    const PWM_PERIOD: u16 = 137;
    pwm.set_top(PWM_PERIOD - 1);
    pwm.set_div_int(12);
    pwm.set_div_frac(0);
    let mut channel_b = pwm.channel_b;
    let _channel_pin_b = channel_b.output_to(pwm_pin);
    channel_b.set_duty(PWM_PERIOD / 3); // デューティ 1/3

    let _dummy_command = [100, 100];

    // SONY 1F0C
    let _amp_input_game = [
        2462, 556, 666, 546, 666, 546, 1242, 554, 1240, 558, 1242, 554, 1242, 558, 1240, 554, 664,
        550, 666, 546, 664, 546, 664, 548, 1246, 556, 1240, 554, 670, 544, 666, 20310, 2464, 558,
        662, 546, 666, 546, 1240, 558, 1240, 556, 1240, 558, 1240, 556, 1242, 558, 662, 548, 664,
        546, 666, 548, 664, 550, 1240, 556, 1240, 558, 666, 546, 666, 20310, 2468, 554, 666, 544,
        666, 548, 1240, 556, 1242, 558, 1242, 556, 1240, 558, 1240, 554, 666, 544, 668, 546, 664,
        550, 664, 546, 1240, 560, 1238, 558, 666, 548, 666,
    ];

    // SONY 600D
    let _amp_input_tv = [
        2464, 552, 1242, 556, 1240, 556, 666, 546, 672, 540, 666, 544, 668, 548, 664, 542, 666,
        548, 664, 544, 668, 544, 666, 546, 1240, 558, 1240, 558, 666, 544, 1244, 21490, 2470, 552,
        1242, 558, 1240, 556, 664, 548, 666, 546, 664, 546, 668, 542, 670, 546, 664, 546, 664, 546,
        668, 544, 666, 542, 1246, 558, 1240, 556, 666, 546, 1242, 21496, 2466, 556, 1240, 556,
        1242, 556, 666, 546, 664, 546, 666, 546, 668, 540, 670, 544, 668, 544, 666, 548, 666, 548,
        664, 546, 1240, 558, 1240, 556, 668, 544, 1244, 21494, 2468, 556, 1240, 558, 1240, 560,
        666, 544, 666, 546, 666, 546, 666, 546, 666, 546, 668, 544, 666, 548, 664, 548, 664, 546,
        1244, 554, 1244, 556, 664, 546, 1244,
    ];

    // SONY 240C
    let _amp_vol_inc = [
        2464, 554, 666, 548, 1238, 556, 670, 544, 662, 548, 1240, 558, 666, 548, 664, 546, 664,
        546, 668, 544, 668, 544, 666, 548, 1240, 558, 1240, 556, 668, 548, 664, 22072, 2464, 556,
        664, 548, 1240, 558, 666, 546, 666, 546, 1242, 556, 664, 546, 668, 546, 664, 546, 668, 548,
        664, 544, 668, 546, 1238, 558, 1242, 558, 664, 548, 666, 22068, 2468, 554, 664, 546, 1242,
        558, 664, 548, 662, 548, 1242, 554, 668, 546, 664, 548, 666, 546, 666, 544, 666, 546, 666,
        546, 1242, 554, 1244, 554, 666, 548, 664, 22070, 2464, 554, 664, 550, 1238, 560, 666, 544,
        664, 550, 1238, 558, 664, 548, 664, 546, 666, 546, 668, 544, 664, 550, 664, 548, 1240, 558,
        1240, 554, 666, 548, 664,
    ];

    // SONY 640C
    let _amp_vol_dec = [
        2464, 554, 1240, 556, 1240, 556, 666, 546, 664, 546, 1242, 556, 664, 548, 666, 546, 666,
        548, 662, 546, 666, 546, 670, 542, 1242, 558, 1240, 558, 664, 544, 666, 21484, 2466, 556,
        1238, 558, 1240, 558, 668, 546, 666, 548, 1238, 560, 666, 544, 664, 548, 666, 546, 664,
        546, 666, 546, 666, 546, 1238, 556, 1244, 558, 664, 548, 664, 21484, 2468, 554, 1238, 558,
        1242, 554, 666, 546, 664, 546, 1242, 556, 670, 546, 664, 550, 662, 548, 662, 548, 666, 544,
        666, 548, 1238, 558, 1240, 558, 664, 546, 664,
    ];

    // PANASONIC 40040D082C29
    let _tv_ch_inc = [
        3508, 1708, 468, 400, 468, 1268, 472, 396, 470, 400, 470, 400, 468, 400, 466, 402, 470,
        398, 468, 400, 472, 396, 472, 398, 470, 396, 472, 398, 470, 1268, 470, 400, 470, 396, 472,
        398, 468, 398, 470, 398, 470, 402, 468, 1270, 470, 1266, 470, 398, 470, 1268, 472, 396,
        470, 398, 472, 398, 470, 400, 470, 1268, 470, 398, 472, 398, 470, 398, 472, 396, 472, 398,
        466, 1272, 468, 398, 470, 1268, 474, 1264, 474, 394, 472, 396, 470, 402, 470, 396, 472,
        1266, 472, 398, 470, 1268, 470, 400, 470, 398, 470, 1270, 470,
    ];

    // PANASONIC 40040D08ACA9
    let _tv_ch_dec = [
        3514, 1702, 470, 400, 470, 1266, 470, 400, 468, 398, 470, 398, 468, 402, 468, 400, 472,
        394, 476, 396, 470, 400, 470, 398, 472, 400, 470, 396, 476, 1262, 470, 400, 472, 396, 472,
        398, 466, 400, 474, 398, 470, 396, 472, 1268, 472, 1268, 470, 398, 470, 1268, 470, 398,
        476, 392, 472, 396, 472, 398, 472, 1266, 470, 396, 470, 402, 468, 398, 470, 1268, 468, 398,
        472, 1270, 470, 396, 472, 1268, 472, 1268, 466, 400, 470, 396, 472, 1268, 470, 398, 468,
        1270, 472, 396, 470, 1268, 468, 400, 474, 396, 468, 1268, 470,
    ];

    // EPSON (Repeat) C1AA09F6
    let _projector_power = [
        9016, 4440, 618, 1632, 620, 1632, 618, 506, 624, 508, 618, 506, 618, 510, 620, 510, 614,
        1634, 622, 1632, 618, 508, 616, 1634, 618, 510, 618, 1632, 618, 508, 620, 1634, 616, 510,
        618, 508, 618, 510, 618, 510, 618, 508, 618, 1632, 618, 512, 616, 510, 620, 1630, 620,
        1634, 618, 1634, 618, 1636, 618, 1630, 620, 508, 620, 1632, 616, 1636, 616, 510, 618,
        40806, 9018, 4438, 620, 1634, 618, 1634, 618, 510, 620, 504, 620, 510, 618, 510, 618, 506,
        618, 1634, 618, 1632, 618, 510, 618, 1632, 622, 508, 618, 1632, 620, 508, 620, 1632, 618,
        510, 616, 512, 618, 504, 620, 510, 618, 506, 620, 1632, 620, 506, 620, 510, 618, 1634, 618,
        1634, 618, 1632, 618, 1634, 620, 1632, 616, 512, 614, 1636, 618, 1636, 614, 512, 618,
    ];

    // SONY 68114
    let _amp_input_dvd = [
        2462, 556, 666, 544, 1242, 554, 1240, 558, 668, 544, 1242, 556, 664, 548, 666, 544, 668,
        546, 662, 550, 664, 544, 668, 542, 1244, 556, 666, 544, 668, 544, 668, 546, 1242, 556, 666,
        546, 1240, 556, 666, 546, 668, 14836, 2470, 550, 668, 546, 1242, 556, 1240, 554, 668, 548,
        1240, 558, 666, 546, 668, 546, 662, 548, 664, 548, 666, 544, 666, 544, 1244, 558, 664, 548,
        664, 548, 664, 548, 1242, 556, 666, 546, 1238, 562, 666, 544, 668, 14834, 2468, 556, 666,
        546, 1240, 558, 1242, 554, 666, 546, 1242, 558, 666, 544, 668, 546, 666, 544, 666, 548,
        666, 546, 666, 544, 1244, 556, 664, 550, 666, 544, 666, 548, 1240, 558, 664, 548, 1238,
        560, 664, 548, 664, 14836, 2466, 556, 666, 546, 1240, 556, 1242, 560, 666, 546, 1242, 554,
        666, 546, 664, 546, 666, 546, 666, 546, 666, 546, 666, 544, 1244, 558, 664, 546, 668, 542,
        668, 546, 1240, 556, 664, 546, 1242, 558, 664, 548, 666,
    ];

    // PANASONIC 40040D08BCB9
    let _tv_power = [
        3514, 1702, 466, 400, 472, 1268, 470, 398, 472, 398, 468, 400, 472, 398, 470, 400, 470,
        398, 470, 396, 470, 400, 470, 398, 470, 398, 470, 400, 470, 1268, 470, 398, 472, 396, 474,
        396, 470, 400, 472, 396, 472, 396, 474, 1266, 468, 1272, 470, 398, 472, 1264, 474, 396,
        468, 402, 468, 400, 470, 396, 474, 1268, 470, 398, 470, 398, 472, 396, 472, 1268, 470, 398,
        470, 1266, 472, 1266, 472, 1266, 470, 1270, 470, 398, 470, 398, 470, 1268, 468, 400, 468,
        1270, 472, 1268, 470, 1266, 472, 396, 474, 396, 470, 1270, 470,
    ];

    loop {
        send_if_pressed(&button_1, &_amp_input_game, &mut ir_pin, &mut delay);
        send_if_pressed(&button_2, &_tv_ch_inc, &mut ir_pin, &mut delay);
        send_if_pressed(&button_3, &_tv_ch_dec, &mut ir_pin, &mut delay);
        send_if_pressed(&button_4, &_amp_input_tv, &mut ir_pin, &mut delay);
        send_if_pressed(&button_5, &_amp_vol_inc, &mut ir_pin, &mut delay);
        send_if_pressed(&button_6, &_amp_vol_dec, &mut ir_pin, &mut delay);

        // 自己保持を解除
        power_pin.set_low().unwrap();
        delay.delay_ms(200);
    }
}

fn send_if_pressed<PIN: InputPin>(
    button: &PIN,
    command: &[u16],
    ir_pin: &mut Pin<Gpio16, Output<PushPull>>,
    delay: &mut Delay,
) {
    if button.is_high().ok().unwrap() {
        for (i, val) in command.iter().enumerate() {
            if i % 2 == 0 {
                ir_pin.set_high().unwrap();
                delay.delay_us(*val as u32);
            } else {
                ir_pin.set_low().unwrap();
                delay.delay_us(*val as u32);
            }
        }
        ir_pin.set_low().unwrap();
    }
}
