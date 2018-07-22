#![no_std]
#![no_main]
#[macro_use(entry,exception)]
extern crate cortex_m_rt;
extern crate cortex_m;
extern crate panic_abort;

#[macro_use]
extern crate stm32ral;
use stm32ral::{gpio, rcc, tim2, nvic};

/// Example of a function taking a `&RegisterBlock` which could be any GPIO.
fn set_pin_9(gpio: &gpio::RegisterBlock) {
    write_reg!(gpio, gpio, BSRR, BS9: Set);
}

entry!(main);
fn main() -> ! {
    // We can `take()` each peripheral to provide safe synchronised access
    let rcc = rcc::RCC::take().unwrap();
    let gpioa = gpio::GPIOA::take().unwrap();
    let gpioe = gpio::GPIOE::take().unwrap();
    let tim2 = tim2::TIM2::take().unwrap();
    let nvic = nvic::NVIC::take().unwrap();

    // Examples of reading from a register
    let _ = read_reg!(gpio, gpioe, IDR);
    let _ = read_reg!(gpio, gpioe, IDR, IDR0);
    let _ = gpioe.IDR.read();

    // Examples of writing to a register
    // (and specifically, here we enable the GPIOE clock).
    write_reg!(rcc, rcc, AHB1ENR, 0x00100000 | (1<<4));
    modify_reg!(rcc, rcc, AHB1ENR, |reg| reg | (1<<4));
    modify_reg!(rcc, rcc, AHB1ENR, GPIOEEN: Enabled);

    // Example of resetting a register.
    // Here we reset PA13, PA14, PA15 to their reset values.
    reset_reg!(gpio, gpioa, GPIOA, MODER, MODER13, MODER14, MODER15);

    // Enable TIM2 clock
    modify_reg!(rcc, rcc, APB1ENR, TIM2EN: Enabled);

    // Set up GPIO for output.
    // We can write a literal value (0b01) or a constant (Output).
    // Other constants include `Input`, `Analog`, and `Alternate`.
    // This example board has three LEDs on PE7, PE8, PE9, but you can modify it appropriately.
    modify_reg!(gpio, gpioe, MODER, MODER7: Output, MODER8: 0b01, MODER9: Output);

    // Set up TIM2 for interrupts at a slow rate
    write_reg!(tim2, tim2, DIER, UIE: Enabled);
    write_reg!(tim2, tim2, PSC, 1_000_000);
    write_reg!(tim2, tim2, ARR, 40);
    write_reg!(tim2, tim2, CR1, CEN: Enabled);

    // Enable TIM2 interrupt in NVIC.
    write_reg!(nvic, nvic, ISER0, 1<<(stm32ral::Interrupt::TIM2 as u8));
    // You could also do this using cortex_m:
    // peripherals.NVIC.enable(stm32ral::Interrupt::TIM2);

    // Example of passing a particular GPIO instance into a function
    set_pin_9(&gpioe);

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
    // Since the timer interrupt happens outside the scope of the main thread,
    // it's hard for it to access the tim2 and gpioe instances.
    // We could have set up a static global variable to pass them around, but
    // instead we can simply access them directly in this unsafe block.
    //
    // Note we didn't need to assign gpioe etc: we can just write `GPIOE`.
    //
    // Because both this interrupt handler and main both use the atomic BSRR register,
    // this is safe. If both were just doing `modify_reg!(..., ODR, ...)`, they
    // could race each other and potentially lose updates.
    unsafe {
        modify_reg!(tim2, TIM2, SR, UIF: 0);
        if read_reg!(gpio, GPIOE, ODR, ODR7 == High) {
            write_reg!(gpio, GPIOE, BSRR, BR7: Reset);
        } else {
            write_reg!(gpio, GPIOE, BSRR, BS7: Set);
        }
    }
}

// You must provide a HardFault exception handler.
exception!(HardFault, hard_fault);
fn hard_fault(ef: &cortex_m_rt::ExceptionFrame) -> ! {
    panic!("HardFault at {:#?}", ef);
}

// You must provide a default handler.
exception!(*, default_handler);
fn default_handler(irqn: i16) {
    panic!("Unhandled exception (IRQn = {})", irqn);
}
