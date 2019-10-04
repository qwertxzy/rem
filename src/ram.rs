use std::io::prelude::*;
use std::fs::File;

pub struct RAM {
    memory: Vec<u8>    // the actual RAM being 32bit addressable bytes
                        // goes from 0x0000_0000 to 0xFFFF_FFFF
}

// Memory Layout:
// stolen from MARS
// 
// 0xffffffff memory map limit address
// 0xffffffff kernel space high address
// 0xffff0000 MMIO base address
// 0xfffeffff kernel data segment limit address
// 0x90000000 .kdata base address
// 0x8ffffffc kernel text limit address
// 0x80000180 exception handler address
// 0x80000000 kernel space base address
// 0x80000000 .ktext base address
// 0x7fffffff user space high address
// 0x7fffffff data segment limit address
// 0x7ffffffc stack base address
// 0x7fffeffc stack pointer $sp
// 0x10040000 stack limit address
// 0x10040000 heap base address
// 0x10010000 .data base address
// 0x10008000 global pointer $gp
// 0x10000000 data segment base address
// 0x10000000 .extern base address
// 0x0ffffffc text lmit address
// 0x00400000 text base

impl RAM {
    //construct a new RAM
    pub fn new() -> RAM {
        RAM { memory: vec![0; std::u32::MAX as usize]}
    }

    //prime the memory with dumps from MARS
    pub fn fill_memory(&mut self, text: String, data: String) {
        println!("Beginning to read text segment into RAM...");
        
        //open the file
        let mut text_file = File::open(text).unwrap();
        //get the slice length in u8 chunks
        let text_len = self.memory[0x0040_0000..0x0ffff_ffc].len() * 4;
        //get a mutable pointer to the slice
        let text_mem_ptr = self.memory[0x0040_0000..0x0ffff_ffc].as_mut_ptr();
        //read the file into the slice in u8 bits
        text_file.read(unsafe {std::slice::from_raw_parts_mut(text_mem_ptr as *mut u8, text_len)}).unwrap();
        
        println!("Done with reading text segment!\nBeginning to read data segment into RAM...");

        //open the file
        let mut data_file = File::open(data).unwrap();
        //get the slice length in u8 chunks
        let data_len = self.memory[0x10010000..0x7fffffff].len() * 4;
        //get a mutable pointer to the slice
        let data_mem_ptr = self.memory[0x10010000..0x7fffffff].as_mut_ptr();
        //read the file into the slice in u8 bits
        data_file.read(unsafe {std::slice::from_raw_parts_mut(data_mem_ptr as *mut u8, data_len)}).unwrap();

        println!("Done with reading the data segment!");
    }

    //print a slice of the memory contents
    pub fn print_mem(&self, start: u32, end: u32) {
        println!("\t----- MEMORY CONTENTS -----\t");
        if start <= end {
            println!("Address:\t\tValue");
            for index in (start..=end).step_by(4) {
                println!("0x{:0>8X}:\t\t0x{:0>8X}", index, self.read_word(index));
            }
        }
        else {
            println!("Please specify a non-negative range")
        }
    }

    //read a byte from memory
    pub fn read_byte(&self, address: u32) -> u8 {
        let address = address as usize;
        return self.memory[address];
    }

    //write a byte to memory
    pub fn write_byte(&mut self, address: u32, byte: u8) {
        let address = address as usize;
        self.memory[address] = byte;
    }

    //read a half (2 consecutive bytes) from memory
    pub fn read_half(&self, address: u32) -> u16 {
        let address = address as usize;
        return u16::from_le_bytes([self.memory[address + 0], self.memory[address + 1]]);
    }

    //write a half (2 consecutive bytes) to memory
    pub fn write_half(&mut self, address: u32, half: u16) {
        let address = address as usize;
        let bytes = half.to_le_bytes();
        self.memory[address + 0] = bytes[0];
        self.memory[address + 1] = bytes[1];
    }

    //read a word (4 consecutive bytes) from memory
    pub fn read_word(&self, address: u32) -> u32 {
        let address = address as usize;
        return u32::from_le_bytes([self.memory[address + 0], self.memory[address + 1], self.memory[address + 2], self.memory[address + 3]]);
    }    

    //write a word (4 consecutive bytes) to memory
    pub fn write_word(&mut self, address: u32, word: u32) {
        let address = address as usize;
        let bytes = word.to_le_bytes();
        self.memory[address + 0] = bytes[0];
        self.memory[address + 1] = bytes[1];
        self.memory[address + 2] = bytes[2];
        self.memory[address + 3] = bytes[3];
    }
}