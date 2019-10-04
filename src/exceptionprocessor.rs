use crate::cpu::CPU;

pub struct ExceptionProcessor {
    BadVAddr: u32,  //Memory address where exception occured
    Status: u32,    //Interrupt mask, enable bits and status when exception occured
    Cause: u32,     //Type of exception and pending interrupt bits
    EPC: u32        //Address of instruction that caused exception

                    //Note: the exception handler itself usually resides in 0x8000_0080
}

impl ExceptionProcessor {
    //construct a new ExceptionProcessor
    pub fn new() -> ExceptionProcessor {
        ExceptionProcessor {BadVAddr: 0, Status: 0, Cause: 0, EPC: 0}
    }

    pub fn throw_exception(&self) {
        println!("Oh no! An exception occured and I simply cannot vibe with it.");
    }

    pub fn syscall(&self, cpu: &mut CPU) {
        let syscall_number = cpu.GPR[2]; //read the syscall number from $v0


    }
}