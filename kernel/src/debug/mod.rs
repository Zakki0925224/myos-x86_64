use crate::{
    arch::{self, idt::InterruptStackFrame},
    device::console,
    error::Result,
    print, println,
};
use dwarf::Dwarf;

pub mod dwarf;

pub fn user_app_debugger(stack_frame: &InterruptStackFrame, dwarf: &Dwarf) -> Result<()> {
    println!("Trapped at 0x{:x}", stack_frame.ins_ptr);

    for (_, abbrevmap) in &dwarf.die_tree {
        for (_, abbrev) in abbrevmap.iter() {
            println!("{:?}", abbrev);
        }
        break;
    }

    loop {
        print!("(dbg) ");
        let mut input_s = None;
        while input_s.is_none() {
            if let Ok(s) = arch::disabled_int(|| console::get_line()) {
                input_s = s;
            } else {
                arch::hlt();
            }
        }

        match input_s.unwrap().as_str().trim() {
            "q" => break,
            "c" => break,
            "s" => break,
            s => {
                println!("Invalid command: {:?}", s);
                continue;
            }
        }
    }

    Ok(())
}
