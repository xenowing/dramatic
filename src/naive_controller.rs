use crate::sdram::{self, NUM_COL_ADDR_BITS, ROW_ADDR_MASK, COL_ADDR_MASK};

pub enum Command {
    // TODO: Mask bits
    Write { addr: u32, data: u128 },
    Read { addr: u32 },
}

pub struct NaiveController {
    sdram: sdram::Sdram,
    io: sdram::Io,
}

impl NaiveController {
    pub fn new() -> NaiveController {
        NaiveController {
            sdram: sdram::Sdram::new(),
            io: sdram::Io::new(),
        }
    }

    // TODO: Return data on read
    pub fn execute(&mut self, command: Command) -> u64 {
        let mut num_cycles = 0;

        match command {
            Command::Write { addr, data } => {
                let element_addr = addr << sdram::NUM_BURST_ADDR_BITS;

                self.io.command = sdram::Command::Active;
                let row_addr = ((element_addr >> NUM_COL_ADDR_BITS) & ROW_ADDR_MASK) as _;
                self.io.a = row_addr;
                self.sdram.clk(&mut self.io);
                num_cycles += 1;
                assert!(self.io.dq.is_none());

                self.io.command = sdram::Command::Write;
                self.io.a = (element_addr & COL_ADDR_MASK) as _;
                for i in 0..sdram::BURST_LEN {
                    self.io.dq = Some((data >> (i * sdram::NUM_ELEMENT_BITS)) as _);
                    self.sdram.clk(&mut self.io);
                    num_cycles += 1;
                    self.io.command = sdram::Command::Nop;
                }

                // TODO: Auto-precharge instead of explicit precharge command
                self.io.command = sdram::Command::Precharge;
                self.io.dq = None;
                self.sdram.clk(&mut self.io);
                num_cycles += 1;
                assert!(self.io.dq.is_none());
            }
            Command::Read { addr } => {
                todo!()
            }
        }

        num_cycles
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_write() {
        let mut c = NaiveController::new();

        let num_cycles = c.execute(Command::Write { addr: 0, data: 0xfadebabedeadbeefabad1deacafef00d });

        println!("Test successful after {} cycles", num_cycles);
    }

    #[test]
    fn two_writes() {
        let mut c = NaiveController::new();

        let mut num_cycles = 0;

        for addr in 0..2 {
            num_cycles += c.execute(Command::Write { addr, data: 0xfadebabedeadbeefabad1deacafef00d });
        }

        println!("Test successful after {} cycles", num_cycles);
    }
}
