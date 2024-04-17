use crate::{print, println};

pub fn hexdump(data: &[u8]) {
    for i in 0..=data.len() / 16 {
        print!("{:08x} ", i * 16);
        let slice = &data[i * 16..(i * 16 + 16).min(data.len())];
        for (j, b) in slice.iter().enumerate() {
            if j % 8 == 0 {
                print!(" ");
            }

            print!("{:02x} ", b);
        }

        if slice.len() < 16 {
            for _ in 0..16 - slice.len() {
                print!("   ");
            }
            print!(" ");
        }

        print!(" |");
        for b in slice {
            if *b >= 0x20 && *b <= 0x7e {
                print!("{}", *b as char);
            } else {
                print!(".");
            }
        }
        println!("|");
    }

    println!();
}
