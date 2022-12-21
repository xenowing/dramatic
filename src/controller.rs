use crate::sdram;

pub enum Command {
    Write { addr: u32, data: u128 },
    Read { addr: u32 },
}

pub struct Controller {
    sdram: sdram::Sdram,
    io: sdram::Io,
}

impl Controller {
    pub fn new() -> Controller {
        Controller {
            sdram: sdram::Sdram::new(),
            io: sdram::Io::new(),
        }
    }

    pub fn execute(&mut self, command: Command) -> u64 {
        match command {
            Command::Write { addr, data } => {
                self.io.command = sdram::Command::Active;
                self.io.a = (addr << 3) as _;
                self.sdram.clk(&mut self.io);
                assert!(self.io.dq.is_none());

                todo!()
            }
            Command::Read { addr } => {
                todo!()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_write() {
        let mut c = Controller::new();

        let _num_cycles = c.execute(Command::Write { addr: 0, data: 0xfadebabedeadbeefabad1deacafef00d });

        // TODO: Assert expected cycle count (if this makes sense?)
    }
}
