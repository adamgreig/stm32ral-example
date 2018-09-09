#![no_std]
#![no_main]
extern crate cortex_m_rt;
extern crate cortex_m;
extern crate panic_abort;

use cortex_m_rt::{entry};

#[macro_use(write_reg, read_reg, modify_reg, reset_reg, interrupt)]
extern crate stm32ral;
use stm32ral::{gpio, rcc, tim2, nvic};

/// Example of a function taking a `&RegisterBlock` which could be any GPIO port.
fn set_pin_9(gpio: &gpio::RegisterBlock) {
    write_reg!(gpio, gpio, BSRR, BS9: Set);
}

#[entry]
fn main() -> ! {
    // We can `take()` each peripheral to provide safe synchronised access.
    let rcc = rcc::RCC::take().unwrap();
    let gpioa = gpio::GPIOA::take().unwrap();
    let gpioe = gpio::GPIOE::take().unwrap();
    let tim2 = tim2::TIM2::take().unwrap();
    let nvic = nvic::NVIC::take().unwrap();

    // Examples of reading from a register.
    let _idr = read_reg!(gpio, gpioe, IDR);
    let _idr0 = read_reg!(gpio, gpioe, IDR, IDR0);
    let (_idr0, _idr1, _idr2) = read_reg!(gpio, gpioe, IDR, IDR0, IDR1, IDR2);
    let _idr = gpioe.IDR.read();

    // Examples of writing to a register
    // (and specifically, here we enable the GPIOE clock three ways).
    write_reg!(rcc, rcc, AHB1ENR, 0x00100000 | (1<<4));
    modify_reg!(rcc, rcc, AHB1ENR, |reg| reg | (1<<4));
    modify_reg!(rcc, rcc, AHB1ENR, GPIOEEN: Enabled);

    // Enable TIM2 clock
    modify_reg!(rcc, rcc, APB1ENR, TIM2EN: Enabled);

    // We're done with RCC so we can release it now.
    // Other code or interrupt handlers etc could take() it.
    rcc::RCC::release(rcc);

    // Example of resetting a register.
    // Here we reset PA13, PA14, PA15 to their reset values.
    reset_reg!(gpio, gpioa, GPIOA, MODER, MODER13, MODER14, MODER15);

    // Set up GPIO for output.
    // We can write a literal value (0b01) or a constant (Output).
    // Other constants include `Input`, `Analog`, and `Alternate`.
    // This example board has three LEDs on PE7, PE8, PE9, but you can modify it appropriately.
    modify_reg!(gpio, gpioe, MODER, MODER7: Output, MODER8: 0b01, MODER9: Output);

    // Set up TIM2 for interrupts at a slow rate.
    write_reg!(tim2, tim2, DIER, UIE: Enabled);
    write_reg!(tim2, tim2, PSC, 1_000_000);
    write_reg!(tim2, tim2, ARR, 40);
    write_reg!(tim2, tim2, CR1, CEN: Enabled);

    // Now we've set up TIM2 we can also release it.
    // Since this call moves tim2, you can't use it afterwards.
    // We can now use `TIM2::take()` in the TIM2 interrupt handler below.
    tim2::TIM2::release(tim2);

    // Enable TIM2 interrupt in NVIC.
    write_reg!(nvic, nvic, ISER0, 1<<(stm32ral::Interrupt::TIM2 as u8));
    // You could also do this using cortex_m:
    // peripherals.NVIC.enable(stm32ral::Interrupt::TIM2);
    // We can also release NVIC now.
    nvic::NVIC::release(nvic);

    // Example of passing a particular GPIO instance into a function.
    set_pin_9(&gpioe);

    // In our main loop we'll just alternate which LEDs are lit one by one.
    // The TIM2 interrupt will also be modifying PE7 at the same time.
    loop {
        write_reg!(gpio, gpioe, BSRR, BR8: Reset, BR9: Reset);
        cortex_m::asm::delay(5_000_000);
        write_reg!(gpio, gpioe, BSRR, BS8: Set, BR9: Reset);
        cortex_m::asm::delay(5_000_000);
        write_reg!(gpio, gpioe, BSRR, BR8: Reset, BS9: Set);
        cortex_m::asm::delay(5_000_000);
    }

}

interrupt!(TIM2, tim2);
fn tim2() {
    // Since we released tim2 in the main function, we can safely take() it here,
    // clear the interrupt pending flag, and release it again for next time.
    // For performance reasons you'd probably prefer to just use unsafe code in an
    // interrupt handler (take-and-release has a lot more overhead), but it's done
    // like this here as an example of what is possible.
    let tim2 = tim2::TIM2::take().unwrap();
    modify_reg!(tim2, tim2, SR, UIF: 0);
    tim2::TIM2::release(tim2);

    // Since the timer interrupt and the main loop are both accessing GPIOE,
    // we can't use the safe `take()` interface here. We'll therefore use unsafe{}.
    //
    // Note we didn't need to assign gpioe etc, instead we can just write `GPIOE`.
    //
    // Because both this interrupt handler and main both use the atomic BSRR register,
    // this is safe. If both were just doing `modify_reg!(..., ODR, ...)`, they
    // could race each other and potentially lose updates.
    unsafe {
        if read_reg!(gpio, GPIOE, ODR, ODR7 == High) {
            write_reg!(gpio, GPIOE, BSRR, BR7: Reset);
        } else {
            write_reg!(gpio, GPIOE, BSRR, BS7: Set);
        }
    }
}
