#![no_std]
#![no_main]
#![allow(clippy::empty_loop)]

use core::num::Wrapping;
use core::{array, mem, ptr, slice};

use n64::boot::interrupts::im;
use n64::boot::{is_bbplayer, ms_to_ticks};
use n64::cop0::cop0;
use n64::si::{si, Si};
use n64::text::Colour;
use n64::types::{Align64, Align8};
use n64::vi::{vi, Mode};

/*fn timer() -> bool {
    static mut CURSOR: bool = false;

    let vi = vi();

    if unsafe { CURSOR } {
        vi.print_char(2, 8, Colour::WHITE, b'/');
    } else {
        vi.print_char(2, 8, Colour::WHITE, b'\\');
    }
    vi.next_framebuffer();

    unsafe { CURSOR = !CURSOR };

    let cop0 = cop0();
    cop0.set_count(0);

    true
}*/

#[no_mangle]
fn main() -> ! {
    /*let im = im();

    im.set_tmr_fn(Some(timer));
    im.set_tmr(true);*/

    let vi = vi();

    vi.init(Mode::NTSC);

    for _ in 0..2 {
        vi.clear_framebuffer();

        vi.print_string(2, 2, Colour::WHITE, "JoyBus poke/peek tool");

        vi.wait_vsync();
        vi.next_framebuffer();
    }

    /*let cop0 = cop0();
    cop0.set_compare(ms_to_ticks(500) / 2);*/

    let si = si();
    si.init_hw();

    /*vi.print_string(2, 4, Colour::WHITE, "si init");
    vi.next_framebuffer();*/

    let mut txidx = Wrapping(0);
    let mut rxidx = Wrapping(0);

    let mut packet = Align8([
        0xFF, 0x01, 0x04, 0x01, 0xFF, 0xFF, 0xFF, 0xFF, // 00
        0xFF, 0x01, 0x04, 0x01, 0xFF, 0xFF, 0xFF, 0xFF, // 08
        0xFF, 0x03, 0x02, 0x55, 0x00, 0x01, 0x00, 0x01, // 10
        0xFF, 0x01, 0x04, 0x01, 0xFF, 0xFF, 0xFF, 0xFF, // 18
        0xFE, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 20
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 28
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 30
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, // 38
    ]);

    loop {
        fn get<const N: usize>(
            si: &mut Si,
            packet: &mut Align8<[u8; 64]>,
            txidx: Wrapping<u8>,
            rxidx: &mut Wrapping<u8>,
        ) -> [u8; N] {
            let mut _get = |si: &mut Si| -> u8 {
                packet.0[0x15] = txidx.0;
                loop {
                    si.write(packet);
                    let resp = si.read();

                    /*let vi = n64::vi::vi();
                    for _ in 0..2 {
                        let mut x = 2;
                        let mut y = 5;

                        for i in 0..8 {
                            x = 2;
                            for j in 0..8 {
                                vi.print_u8(x * 3, y, Colour::RED, resp[i * 8 + j]);
                                x += 1;
                            }
                            y += 1;
                        }
                        vi.wait_vsync();
                        vi.next_framebuffer();
                    }*/

                    if resp[0x17] != rxidx.0 {
                        *rxidx = Wrapping(resp[0x17]);
                        break resp[0x16];
                    }
                }
            };

            /*let mut buf = [0; N];
            si.txrx(&[], Some(&mut buf));
            buf*/
            array::from_fn(|_| _get(si))
        }

        fn set(si: &mut Si, packet: &mut Align8<[u8; 64]>, txidx: &mut Wrapping<u8>, data: &[u8]) {
            let mut _set = |si: &mut Si, byte: u8| {
                *txidx += 1;
                packet.0[0x14] = byte;
                packet.0[0x15] = txidx.0;
                si.write(packet);
                let _ = si.read();

                /*let vi = n64::vi::vi();
                for _ in 0..2 {
                    let mut x = 2;
                    let mut y = 5;

                    for i in 0..8 {
                        x = 2;
                        for j in 0..8 {
                            vi.print_u8(x * 3, y, Colour::RED, resp[i * 8 + j]);
                            x += 1;
                        }
                        y += 1;
                    }
                    vi.wait_vsync();
                    vi.next_framebuffer();
                }*/
            };

            for &byte in data {
                _set(si, byte)
            }
        }

        /*vi.print_u32(2, 6, Colour::GREEN, cop0.status());
        vi.next_framebuffer();
        vi.print_u32(2, 6, Colour::GREEN, cop0.status());
        vi.next_framebuffer();

        vi.print_string(2, 4, Colour::WHITE, "tx si");
        vi.next_framebuffer();

        //si.txrx(b"test tx", None);

        vi.print_string(2, 4, Colour::WHITE, "read si");
        vi.next_framebuffer();

        vi.print_u32(2, 7, Colour::RED, cop0.status());
        vi.next_framebuffer();
        vi.print_u32(2, 7, Colour::RED, cop0.status());
        vi.next_framebuffer();*/

        let byte = get::<1>(si, &mut packet, txidx, &mut rxidx);

        /*vi.print_string(2, 4, Colour::WHITE, "got read");
        vi.next_framebuffer();*/

        if byte == [0xA5] {
            //si.txrx(&[0x5A], None);
            set(si, &mut packet, &mut txidx, &[0x5A]);

            vi.print_string(2, 3, Colour::BLACK, "████████████████████████████████████");
            vi.next_framebuffer();
            vi.print_string(2, 3, Colour::BLACK, "████████████████████████████████████");

            let oper = get::<1>(si, &mut packet, txidx, &mut rxidx)[0];

            match oper {
                0 => {
                    // read
                    vi.print_string(2, 3, Colour::GREEN, "Read          = ");
                    vi.next_framebuffer();
                    vi.print_string(2, 3, Colour::GREEN, "Read          = ");

                    let addr = u32::from_be_bytes(get::<4>(si, &mut packet, txidx, &mut rxidx));

                    vi.print_u32(7, 3, Colour::GREEN, addr);
                    vi.next_framebuffer();
                    vi.print_u32(7, 3, Colour::GREEN, addr);

                    let data =
                        unsafe { ptr::with_exposed_provenance::<u32>(addr as _).read_volatile() };

                    vi.print_u32(18, 3, Colour::GREEN, data);
                    vi.next_framebuffer();
                    vi.print_u32(18, 3, Colour::GREEN, data);

                    //si.txrx(&data.to_be_bytes(), None);
                    set(si, &mut packet, &mut txidx, &data.to_be_bytes());
                }
                1 => {
                    //write
                    vi.print_string(2, 3, Colour::YELLOW, "Write ");
                    vi.next_framebuffer();
                    vi.print_string(2, 3, Colour::YELLOW, "Write ");

                    let addr = u32::from_be_bytes(get::<4>(si, &mut packet, txidx, &mut rxidx));

                    vi.print_u32(8, 3, Colour::YELLOW, addr);
                    vi.next_framebuffer();
                    vi.print_u32(8, 3, Colour::YELLOW, addr);

                    let data = u32::from_be_bytes(get::<4>(si, &mut packet, txidx, &mut rxidx));

                    unsafe {
                        ptr::with_exposed_provenance_mut::<u32>(addr as _).write_volatile(data)
                    };
                }
                4 => {
                    //exec
                    vi.print_string(2, 3, Colour::RED, "Exec ");
                    vi.next_framebuffer();
                    vi.print_string(2, 3, Colour::RED, "Exec ");

                    let addr = u32::from_be_bytes(get::<4>(si, &mut packet, txidx, &mut rxidx));

                    vi.print_u32(7, 3, Colour::RED, addr);
                    vi.next_framebuffer();
                    vi.print_u32(7, 3, Colour::RED, addr);

                    let ptr = unsafe {
                        mem::transmute::<*const (), unsafe extern "C" fn()>(
                            ptr::with_exposed_provenance::<()>(addr as _),
                        )
                    };

                    unsafe { ptr() };
                }
                _ => {
                    vi.print_string(2, 3, Colour::RED, "Unknown command ");
                    vi.print_u8(18, 3, Colour::RED, oper);
                    vi.next_framebuffer();
                    vi.print_string(2, 3, Colour::RED, "Unknown command ");
                    vi.print_u8(18, 3, Colour::RED, oper);
                }
            }

            vi.next_framebuffer();
            //vi.wait_vsync();
        }

        /*vi.print_string(2, 4, Colour::WHITE, "wait vsync");
        vi.next_framebuffer();*/

        //vi.wait_vsync();
    }
}
