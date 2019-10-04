pub(crate) mod cpu;
pub(crate) mod ram;
pub(crate) mod exceptionprocessor;

use crate::ram::RAM;
use crate::cpu::CPU;

use std::io::{self, BufRead, Write};


fn main() {

    //initialize the ram
    let mut ram = RAM::new();

    //write a nice instruction into it
    //ram.write_mem(0, 0b001101_00000_00011_0000000011111111);

    //fill the ram with a memory dump from MARS
    ram.fill_memory("dumps/text_fib.bin".to_string(), "dumps/data_fib.bin".to_string());
    //TODO: tidy this up


    //initialize the cpu
    let mut cpu = CPU::new(ram);

    cpu.reset();

    let stdin = io::stdin();

    //main emulation loop
    loop {
        //print a promopt
        print!("\n>");
        io::stdout().flush().unwrap();

        //read one line in one line
        let line = stdin.lock().lines().next().unwrap().unwrap();
        let chunks: Vec<&str> = line.split_whitespace().collect();

        //check the command used
        match chunks[0] {   //TODO: clean up the optional argument parsing mess
            "help" => help(), // print help menu
            "clock" => clock(&mut cpu, if chunks.len() == 2 { chunks[1].parse().unwrap() } else { 1 }), //clock the cpu n times
            "readregs" => read_regs(&cpu, if chunks.len() == 2 { match chunks[1] { "d" => false, "h" => true, _ => false } } else { true }), // read out all registers in hex or decimal based on the 2nd argument
            "readmem" => read_mem(&cpu, u32::from_str_radix(chunks[1], 16).unwrap(), u32::from_str_radix(chunks[2], 16).unwrap()), //read memory from address to address
            "readinst" => read_inst(&cpu, if chunks.len() == 2 { chunks[1].parse().unwrap() } else { 0 }),
            "reset" => cpu.reset(), //reset the cpu
            "quit" | "q" => break, //quits the program

            _ => println!("Command not recognized, for a list of commands enter 'help'")
        }
    }
}

fn help() {
    println!("Please enter one of the following commands:\n
                \rhelp\t\t\tPrints this help menu\n
                \rclock [n]\t\t\tClocks the CPU n-times\n
                \rreadregs [d/H]\t\t\tPrints out all the CPU's registers in [d]ecimal or [h]ex\n
                \rreadmem A B\t\t\tPrints out memory contents from 0xA to 0xB\n
                \rreadinst [o]\t\t\tPrints an instruction in binary at offset o (default 0)\n
                \rreset\t\t\tResets the CPU\n
                \rquit / q\t\t\tQuits the program");
}

fn clock(cpu: &mut CPU, n: u16) {
    for _ in 0..n {
        cpu.clock();
    }
}

fn read_regs(cpu: &CPU, format_hex: bool) {
    cpu.print_reg(format_hex);
}

fn read_mem(cpu: &CPU, start: u32, end: u32) {
    cpu.print_mem(start, end);
}

fn read_inst(cpu: &CPU, offset: i16 ) {
    cpu.print_instruction(offset);
}
