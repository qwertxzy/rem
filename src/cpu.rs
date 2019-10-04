use crate::ram::RAM;
use crate::exceptionprocessor::ExceptionProcessor;

use std::convert::TryInto;

pub struct CPU {
    GPR: [u32; 32],     //register number 0 - 31
    HI: u32,            //register number 32
    LO: u32,            //register number 33
    PC: u32,            //register number 34

    MEM: RAM,
    //TODO: add FPU

    //TODO: implement the Exception coprocessor properly
    CP0: ExceptionProcessor
}

impl CPU {
    //construct a new cpu
    pub fn new(ram: RAM) -> CPU {
        CPU {GPR: [0; 32], HI: 0, LO: 0, PC: 0, MEM: ram, CP0: ExceptionProcessor::new()}
    }

    //do a clock cycle
    pub fn clock(&mut self) {
        //fetch next instruction word
        let word: u32 = self.MEM.read_word(self.PC);
        println!("Fetching next instruction from address {:#X}", self.PC);

        //split it into opcode and arguments
        let opcode: u8 = ((word & 0xFC00_0000) >> 26).try_into().unwrap();    //6 bits long
        let rs: u8 = ((word & 0x03E0_0000) >> 21).try_into().unwrap();        //5 bits long
        let rt: u8 = ((word & 0x001F_0000) >> 16).try_into().unwrap();        //5 bits long
        let imm: u16 = ((word & 0x0000_FFFF)).try_into().unwrap();            //16 bits long

        //SPECIAL opcodes
        if opcode == 0x00 {
            //hack the immediate field down into a third register & the new opcode
            let rd: u8 = ((imm & 0xF800) >> 11).try_into().unwrap();
            let special_opcode: u8 = ((imm & 0x003F)).try_into().unwrap();

            //debug printing
            println!("Found Opcode 0x00 {:#X} with registers {}, {}, {}", special_opcode, rs, rt, rd);

            match special_opcode {
                0x0A => self.MOVZ(rs, rt, rd),
                0x0B => self.MOVN(rs, rt, rd),
                0x24 => self.AND(rs, rt, rd),
                0x2A => self.SLT(rs, rt, rd),
                0x08 => self.JR(rs),
                0x20 => self.ADD(rs, rt, rd),
                0x21 => self.ADDU(rs, rt, rd),
                0x22 => self.SUB(rs, rt, rd),
                0x23 => self.SUBU(rs, rt, rd),
                0x25 => self.OR(rs, rt, rd),
                0x26 => self.XOR(rs, rt, rd),
                0x27 => self.NOR(rs, rt, rd),
                0x0C => self.SYSCALL(),
                _ => ()
            }
        } 
        //another set of opcodes, SPECIAL2
        else if opcode == 0x1C {
            //same as with opcode 0x00
            let rd: u8 = ((imm & 0xF800) >> 11).try_into().unwrap();
            let special_opcode: u8 = ((imm & 0x003F)).try_into().unwrap();

            //debug printing
            println!("Found Opcode 0x1C {:#X} with registers {}, {}, {}", special_opcode, rs, rt, rd);

            match special_opcode {
                0x1C => self.CLZ(rs, rt, rd),
                0x21 => self.CLO(rs, rt, rd),
                _ => ()
            }
        }
        //normal opcodes here
        else {
            //debug printing
            println!("Found Opcode {:#X} with registers {}, {} and immediate {}", opcode, rs, rt, imm);

            match opcode {
                0x03 => self.JAL((word & 0x03FF_FFFF).try_into().unwrap()),
                0x07 => self.BGTZ(rs, imm),
                0x08 => self.ADDI(rs, rt, imm),
                0x09 => self.ADDIU(rs, rt, imm),
                0x0A => self.SLTI(rs, rt, imm),
                0x0B => self.SLTIU(rs, rt, imm),
                0x0C => self.ANDI(rs, rt, imm),
                0x0D => self.ORI(rs, rt, imm),
                0x0E => self.XORI(rs, rt, imm),
                0x0F => self.LUI(rt, imm),
                0x20 => self.LB(rs, rt, imm),
                0x21 => self.LH(rs, rt, imm),
                0x23 => self.LW(rs, rt, imm),
                0x2B => self.SW(rs, rt, imm),
                _ => ()
            }
        }

        //increment the program counter by 32 bit (4 * 8 addressable bit)
        self.PC += 4;
    }

    //print all kinds of information about the CPU
    pub fn print_reg(&self, format_hex: bool) {
        println!("\t----- PROCESSOR STATE -----\t");
        
        if format_hex {
            println!("Registers:");

            for i in 1..=31 {
                println!("${:0>2}: 0x{:0>8X}", i, self.GPR[i]);
            }
        }
        else {
            println!("Registers:");

            for i in 1..=31 {
                println!("${:0>2}: {:0>8}", i, self.GPR[i]);
            }
        }
        
        println!("\nHI/LO: {}/{}\n
                  \rProgram Counter: {:#X}",
                self.HI, self.LO, self.PC);
    }
    
    //print a portion of the main memory
    pub fn print_mem(&self, start: u32, end: u32) {
        self.MEM.print_mem(start, end);
    }

    //print the current instruction +- an offset in whole instructions in binary
    pub fn print_instruction(&self, offset: i16) {
        println!("{:0>32b}", self.MEM.read_word(self.PC.wrapping_add((offset * 4)as u32)));
    }

    //reset the cpu to a known state
    pub fn reset(&mut self) {
        self.GPR = [0; 32];         //null all registers - not needed but nice
        self.PC = 0x0040_0000;      //.text segment base address
        self.GPR[29] = 0x7fffeffc;  //stack pointer $sp base address

    }

    //read data from a register
    fn read_reg(&self, number: u8) -> u32 {
        match number {
            0       => 0,   //register 0 is hardwired to logic 0
            1..=31   => self.GPR[number as usize],
            32      => self.HI,
            33      => self.LO,
            34      => self.PC,
            _       => 0 //error handle later?
        }
    }

    //write data to a register
    fn write_reg(&mut self, number: u8, value: u32) {
        match number {
            1..=31   => self.GPR[number as usize] = value,
            34      => self.PC = value,
            _       => ()
        }
    }

    ///////////////
    //
    //
    // INSTRUCTIONS
    // 
    // 
    ///////////////

    #[allow(non_snake_case)]
    fn AND(&mut self, rs: u8, rt: u8, rd: u8) {
        self.write_reg(rd, self.read_reg(rs) & self.read_reg(rt))
    }
    
    #[allow(non_snake_case)]
    fn ANDI(&mut self, rs: u8, rt: u8, imm: u16) {
        self.write_reg(rt, self.read_reg(rs) & ((imm as u32) << 16))
    }

    #[allow(non_snake_case)]
    fn OR(&mut self, rs: u8, rt: u8, rd: u8) {
        self.write_reg(rd, self.read_reg(rs) | self.read_reg(rt));
    }

    #[allow(non_snake_case)]
    fn ORI(&mut self, rs: u8, rt: u8, imm: u16) {
        self.write_reg(rt, self.read_reg(rs) | (imm as u32));
    }

    #[allow(non_snake_case)]
    fn NOR(&mut self, rs: u8, rt: u8, rd: u8) {
        self.write_reg(rd, !(self.read_reg(rs) | self.read_reg(rt)));
    }

    #[allow(non_snake_case)]
    fn XOR(&mut self, rs: u8, rt: u8, rd: u8) {
        self.write_reg(rd, self.read_reg(rs) ^ self.read_reg(rt));
    }

    #[allow(non_snake_case)]
    fn XORI(&mut self, rs: u8, rt: u8, imm: u16) {
        self.write_reg(rt, self.read_reg(rs) ^ (imm as u32));
    }

    #[allow(non_snake_case)]
    fn SLT(&mut self, rs: u8, rt: u8, rd: u8) {
        self.write_reg(rd, if self.read_reg(rs) < self.read_reg(rt) { 1 } else { 0 });
    }

    #[allow(non_snake_case)]
    fn SLTI(&mut self, rs: u8, rt: u8, imm: u16) {
        //sign extend the immediate
        let signed_imm = imm as i16 as i32;
        self.write_reg(rt, if (self.read_reg(rs) as i32) < (signed_imm) { 1 } else { 0 });
    }

    #[allow(non_snake_case)]
    fn JR(&mut self, rs: u8) {
        self.write_reg(34, self.read_reg(rs) - 4);
    }

    #[allow(non_snake_case)]
    fn ADDU(&mut self, rs: u8, rt: u8, rd: u8) {
        let (result, _overflow_flag) = self.read_reg(rs).overflowing_add(self.read_reg(rt));

        self.write_reg(rd, result);
    }

    #[allow(non_snake_case)]
    fn ADDIU(&mut self, rs: u8, rt: u8, imm: u16) {
        //sign extend the immediate
        let signed_imm = imm as i16 as i32;

        let (result, _overflow_flag) = self.read_reg(rs).overflowing_add(signed_imm as u32);

        self.write_reg(rt, result);
    }

    #[allow(non_snake_case)]
    fn CLZ(&mut self, rs: u8, _rt: u8, rd: u8) {
        //in the original design rt and rd have to be equal!
        self.write_reg(rd, self.read_reg(rs).count_ones());
    }

    #[allow(non_snake_case)]
    fn CLO(&mut self, rs: u8, _rt: u8, rd: u8) {
        //in the original design rt and rd have to be equal!
        self.write_reg(rd, self.read_reg(rs).count_zeros());
    }

    #[allow(non_snake_case)]
    fn LUI(&mut self, rt: u8, imm: u16) {
        self.write_reg(rt, (imm as u32) << 16);
    }

    #[allow(non_snake_case)]
    fn SYSCALL(&mut self) {
        println!("System call registered! Too bad I don't handle exceptions ye(e)t."); //TODO: implement this
    }
    
    #[allow(non_snake_case)]
    fn LB(&mut self, base: u8, rt: u8, imm: u16) {
        //sign extend the immediate
        let signed_imm = imm as i16 as i32;
        //compute the address as a sum of i32, then cast back to u32
        let address = ((self.read_reg(base) as i32) + signed_imm) as u32;

        //read a byte as u8, then cast to i8 and i32 to sign extend to i32, then back to u32 to write it into a register
        self.write_reg(rt, self.MEM.read_byte(address) as i8 as i32 as u32);
    }

    #[allow(non_snake_case)]
    fn LH(&mut self, base: u8, rt: u8, imm: u16) {
        //sign extend the immediate
        let signed_imm = imm as i16 as i32;
        //compute the address as a sum of i32, then cast back to u32
        let address = ((self.read_reg(base) as i32) + signed_imm) as u32;

        //TODO: throw an exception if the address isn't aligned properly, LSB != 0 => Address Error exception
        if address % 2 != 0 {
            self.CP0.throw_exception();
        }

        //read a byte as u16, then cast it to i16 and i32 to sign extend to i32, then back to u32 to write it into a register
        self.write_reg(rt, self.MEM.read_half(address) as i16 as i32 as u32);
    }

    #[allow(non_snake_case)]
    fn LW(&mut self, base: u8, rt: u8, imm: u16) { 
        //sign extend the immediate
        let signed_imm = imm as i16 as i32;
        //compute the address as a sum of i32, then cast back to u32
        let address = ((self.read_reg(base) as i32) + signed_imm) as u32;

        //TODO: throw an exception if the address isn't aligned properly, 2 LSB != 0 => Address Error exception
        if address % 4 != 0 {
            self.CP0.throw_exception();
        }

        //read a word and write it into a register
        self.write_reg(rt, self.MEM.read_word(address));
    }

    #[allow(non_snake_case)]
    fn BGTZ(&mut self, rs: u8, imm: u16) {
        //execute the branch delay slot
        println!("Executing branch delay slot:");
        self.PC += 4;
        self.clock();

        //interpret the immediate as signed and lshift it by 2 because instructions are 4-aligned
        let signed_imm: i32 = ((imm as i16) << 2) as i32; 

        //if the register value is > 0 add the offset to the PC
        if self.read_reg(rs) > 0 {
            //debug prints!
            println!("Attempting to add {} to the current PC {}", signed_imm, self.PC);

            //off by two instructions, because of the branch delay slot and the PC increment at the end of clock()
            self.PC = ((self.PC as i32 + signed_imm) - 8) as u32;
        }
    }

    #[allow(non_snake_case)]
    fn ADD(&mut self, rs: u8, rt: u8, rd: u8) {
        let (result, overflow_flag) = (self.read_reg(rs) as i32).overflowing_add(self.read_reg(rt) as i32);

        if overflow_flag {
            //TODO: Call CP0 that we have an Integer Overflow exception
            self.CP0.throw_exception();
        }
        else {
            self.write_reg(rd, result as u32);
        }
    }

    #[allow(non_snake_case)]
    fn ADDI(&mut self, rs: u8, rt: u8, imm: u16) {
        let signed_imm = imm as i16 as i32;

        let (result, overflow_flag) = (self.read_reg(rs) as i32).overflowing_add(signed_imm as i32);

        if overflow_flag {
            //TODO: Call CP0 that we have an Integer Overflow exception
            self.CP0.throw_exception();
        }
        else {
            self.write_reg(rt, result as u32);
        }
    }

    #[allow(non_snake_case)]
    fn SUB(&mut self, rs: u8, rt: u8, rd: u8) {
        let (result, overflow_flag) = (self.read_reg(rs) as i32).overflowing_sub(self.read_reg(rt) as i32);

        if overflow_flag {
            //TODO: Call CP0 that we have an Integer Overflow exception
            self.CP0.throw_exception();
        }
        else {
            self.write_reg(rd, result as u32);
        }
    }

    #[allow(non_snake_case)]
    fn SUBU(&mut self, rs: u8, rt: u8, rd: u8) {
        let (result, _overflow_flag) = (self.read_reg(rs) as i32).overflowing_sub(self.read_reg(rt) as i32);

        self.write_reg(rd, result as u32);

    }

    #[allow(non_snake_case)]
    fn JAL(&mut self, instr_index: u32) {
        //execute the branch delay slot
        println!("Executing branch delay slot:");
        self.PC += 4;
        self.clock();

        //store the return address in the $ra register
        self.write_reg(31, self.PC + 4);

        //isolate the upper 2 bits of the PC
        let pc_part = self.PC & 0xC000_000;

        //left shift the index as instructions are 4 aligned
        let instr_index = instr_index << 2;

        //update the program counter to the new value - 8 to account for the PC increment in clock() and the branch delay slot
        self.PC = pc_part & instr_index; 
    }

    #[allow(non_snake_case)]
    fn SW(&mut self, base: u8, rt: u8, offset: u16) {
         //sign extend the offset
        let signed_offset = offset as i16 as i32;
        //compute the address as a sum of i32, then cast back to u32
        let address = ((self.read_reg(base) as i32) + signed_offset) as u32;

        //TODO: throw an exception if the address isn't aligned properly, 2 LSB != 0 => Address Error exception
        //there are also all kinds of other exceptions that can occur here but who the hell knows what TLB Refill means
        if address % 4 != 0 {
            self.CP0.throw_exception();
        }

        //store the contents of rt in memory
        self.MEM.write_word(address, self.read_reg(rt));
    }

    #[allow(non_snake_case)]
    fn MOVN(&mut self, rs: u8, rt: u8, rd: u8) {
        if self.read_reg(rt) != 0 {
            self.write_reg(rd, self.read_reg(rs));
        }
    }

    #[allow(non_snake_case)]
    fn MOVZ(&mut self, rs: u8, rt: u8, rd: u8) {
        if self.read_reg(rt) == 0 {
            self.write_reg(rd, self.read_reg(rs));
        }
    }

    #[allow(non_snake_case)]
    fn SLTIU(&mut self, rs: u8, rt: u8, imm: u16) {
        let signed_imm = imm as i16 as i32;
        //sign extend the immediate first, but then do an unsigned comparison
        self.write_reg(rt, if self.read_reg(rs) < (signed_imm as u32) { 1 } else { 0 })
    }
}