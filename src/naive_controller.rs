use crate::sdram;

use std::io;

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
    pub fn new(sdram: sdram::Sdram) -> NaiveController {
        NaiveController {
            sdram,
            io: sdram::Io::new(),
        }

        // TODO: Initialization
    }

    pub fn execute(&mut self, command: Command) -> io::Result<(Option<u128>, u64)> {
        let mut ret_data = None;
        let mut num_cycles = 0;

        match command {
            Command::Write { addr, data } => {
                let element_addr = addr << sdram::NUM_BURST_ADDR_BITS;
                let bank_addr = element_addr
                    >> (sdram::NUM_ROW_ADDR_BITS + sdram::NUM_COL_ADDR_BITS)
                    & sdram::NUM_BANK_ADDR_BITS;
                self.io.bank = sdram::IoBank::from_index(bank_addr as _).unwrap();

                self.io.command = sdram::Command::Active;
                let row_addr =
                    ((element_addr >> sdram::NUM_COL_ADDR_BITS) & sdram::ROW_ADDR_MASK) as _;
                self.io.a = row_addr;
                for _ in 0..sdram::T_RCD_CYCLES {
                    self.sdram.clk(&mut self.io)?;
                    num_cycles += 1;
                    self.io.command = sdram::Command::Nop;
                }

                self.io.command = sdram::Command::Write;
                self.io.a = (element_addr & sdram::COL_ADDR_MASK) as _;
                for i in 0..sdram::BURST_LEN {
                    self.io.dq_in =
                        sdram::OptionalBytePair::some((data >> (i * sdram::NUM_ELEMENT_BITS)) as _);
                    self.sdram.clk(&mut self.io)?;
                    num_cycles += 1;
                    self.io.command = sdram::Command::Nop;
                }

                self.io.dq_in = sdram::OptionalBytePair::none();
                for _ in 0..sdram::T_WR_CYCLES - 1 {
                    self.sdram.clk(&mut self.io)?;
                    num_cycles += 1;
                }

                // TODO: Auto-precharge instead of explicit precharge command
                self.io.command = sdram::Command::Precharge;
                for _ in 0..sdram::T_RP_CYCLES {
                    self.sdram.clk(&mut self.io)?;
                    num_cycles += 1;
                    self.io.command = sdram::Command::Nop;
                }
            }
            Command::Read { addr } => {
                let element_addr = addr << sdram::NUM_BURST_ADDR_BITS;
                let bank_addr = element_addr
                    >> (sdram::NUM_ROW_ADDR_BITS + sdram::NUM_COL_ADDR_BITS)
                    & sdram::NUM_BANK_ADDR_BITS;
                self.io.bank = sdram::IoBank::from_index(bank_addr as _).unwrap();

                self.io.command = sdram::Command::Active;
                let row_addr =
                    ((element_addr >> sdram::NUM_COL_ADDR_BITS) & sdram::ROW_ADDR_MASK) as _;
                self.io.a = row_addr;
                for _ in 0..sdram::T_RCD_CYCLES {
                    self.sdram.clk(&mut self.io)?;
                    num_cycles += 1;
                    self.io.command = sdram::Command::Nop;
                }

                self.io.command = sdram::Command::Read;
                self.io.a = (element_addr & sdram::COL_ADDR_MASK) as _;
                for _ in 0..sdram::CAS_LATENCY {
                    self.sdram.clk(&mut self.io)?;
                    num_cycles += 1;
                    self.io.command = sdram::Command::Nop;
                }
                let mut data = 0;
                for i in 0..sdram::BURST_LEN {
                    data |= (self.io.dq().expect("No data returned for read cycle.") as u128)
                        << (i * sdram::NUM_ELEMENT_BITS);
                    self.sdram.clk(&mut self.io)?;
                    num_cycles += 1;
                }
                ret_data = Some(data);

                // TODO: Auto-precharge instead of explicit precharge command
                self.io.command = sdram::Command::Precharge;
                for _ in 0..sdram::T_RP_CYCLES {
                    self.sdram.clk(&mut self.io)?;
                    num_cycles += 1;
                    self.io.command = sdram::Command::Nop;
                }
            }
        }

        Ok((ret_data, num_cycles))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_write() -> io::Result<()> {
        let mut c = NaiveController::new(sdram::Sdram::new(Some("NaiveController__one_write"))?);

        let (ret_data, num_cycles) = c.execute(Command::Write {
            addr: 0,
            data: 0xfadebabedeadbeefabad1deacafef00d,
        })?;
        assert!(ret_data.is_none());

        println!("Test successful after {} cycles", num_cycles);

        Ok(())
    }

    #[test]
    fn two_writes() -> io::Result<()> {
        let mut c = NaiveController::new(sdram::Sdram::new(Some("NaiveController__two_writes"))?);

        let mut num_cycles = 0;

        for addr in 0..2 {
            let (ret_data, command_cycles) = c.execute(Command::Write {
                addr,
                data: 0xfadebabedeadbeefabad1deacafef00d,
            })?;
            assert!(ret_data.is_none());
            num_cycles += command_cycles;
        }

        println!("Test successful after {} cycles", num_cycles);

        Ok(())
    }

    #[test]
    fn one_write_read() -> io::Result<()> {
        let mut c =
            NaiveController::new(sdram::Sdram::new(Some("NaiveController__one_write_read"))?);

        let addr = 0;
        let expected_data = 0xfadebabedeadbeefabad1deacafef00d;

        let mut num_cycles = 0;

        let (ret_data, command_cycles) = c.execute(Command::Write {
            addr,
            data: expected_data,
        })?;
        assert!(ret_data.is_none());
        num_cycles += command_cycles;

        let (ret_data, command_cycles) = c.execute(Command::Read { addr })?;
        assert_eq!(
            ret_data.expect("No data returned from read command."),
            expected_data
        );
        num_cycles += command_cycles;

        println!("Test successful after {} cycles", num_cycles);

        Ok(())
    }

    #[test]
    fn two_writes_reads() -> io::Result<()> {
        let mut c = NaiveController::new(sdram::Sdram::new(Some(
            "NaiveController__two_writes_reads",
        ))?);

        let expected_data = 0xfadebabedeadbeefabad1deacafef00d;

        let mut num_cycles = 0;

        for addr in 0..2 {
            let (ret_data, command_cycles) = c.execute(Command::Write {
                addr,
                data: expected_data,
            })?;
            assert!(ret_data.is_none());
            num_cycles += command_cycles;

            let (ret_data, command_cycles) = c.execute(Command::Read { addr })?;
            assert_eq!(
                ret_data.expect("No data returned from read command."),
                expected_data
            );
            num_cycles += command_cycles;
        }

        println!("Test successful after {} cycles", num_cycles);

        Ok(())
    }
}
