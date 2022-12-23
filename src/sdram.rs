// Alliance AS4C32M16MSA-6BIN
//  8M x 16bits x 4 banks (64MBytes)
//  Assumes 166MHz operation

pub const NUM_ELEMENT_BITS: u32 = 16;
pub const ELEMENT_MASK: u32 = 1 << NUM_ELEMENT_BITS - 1;
pub const NUM_ROW_ADDR_BITS: u32 = 13;
pub const NUM_COL_ADDR_BITS: u32 = 10;
pub const NUM_ROWS: u32 = 1 << NUM_ROW_ADDR_BITS;
pub const ROW_ADDR_MASK: u32 = NUM_ROWS - 1;
pub const NUM_COLS: u32 = 1 << NUM_COL_ADDR_BITS;
pub const COL_ADDR_MASK: u32 = NUM_COLS - 1;
pub const NUM_BANK_ADDR_BITS: u32 = 2;
pub const NUM_BANKS: u32 = 1 << NUM_BANK_ADDR_BITS;
pub const BANK_ADDR_MASK: u32 = NUM_BANKS - 1;
pub const CAS_LATENCY: u32 = 3;
pub const NUM_BURST_ADDR_BITS: u32 = 3;
pub const BURST_LEN: u32 = 1 << NUM_BURST_ADDR_BITS; // 128-bit effective word size

#[derive(Clone)]
struct Row {
    cols: Box<[u16]>,
}

impl Row {
    fn new() -> Row {
        Row {
            cols: vec![0; NUM_COLS as usize].into(),
        }
    }
}

#[derive(Clone)]
struct Bank {
    rows: Box<[Row]>,
    active_row: Option<usize>,
}

impl Bank {
    fn new() -> Bank {
        Bank {
            rows: vec![Row::new(); NUM_ROWS as usize].into(),
            active_row: None,
        }
    }

    fn active(&mut self, row_addr: u32) {
        if self.active_row.is_some() {
            panic!("Attempted to activate a row in a bank which already has an active row.");
        }

        self.active_row = Some(row_addr as _);
    }

    fn precharge(&mut self) {
        self.active_row = None;
    }

    fn read(&mut self, col_addr: u32) -> u16 {
        // TODO: Test(s)
        let active_row = self.active_row.expect("Attempted to read from a column in a bank which does not currently have an active row.");
        self.rows[active_row as usize].cols[col_addr as usize]
    }

    fn write(&mut self, col_addr: u32, data: u16) {
        // TODO: Test(s)
        let active_row = self.active_row.expect("Attempted to write to a column in a bank which does not currently have an active row.");
        self.rows[active_row as usize].cols[col_addr as usize] = data;
    }
}

// TODO: Add LoadModeRegister command
pub enum Command {
    Active,
    AutoRefresh,
    Nop,
    Precharge,
    Read,
    Write,
}

#[derive(Clone, Copy)]
pub enum IoBank {
    Bank0,
    Bank1,
    Bank2,
    Bank3,
}

impl IoBank {
    fn index(&self) -> usize {
        match *self {
            IoBank::Bank0 => 0,
            IoBank::Bank1 => 1,
            IoBank::Bank2 => 2,
            IoBank::Bank3 => 3,
        }
    }
}

// TODO: More specific name?
enum State {
    Idle,
    Read { bank: IoBank, num_cycles: u32 },
    Write { bank: IoBank, num_cycles: u32 },
}

pub struct Io {
    pub command: Command,
    // TODO: Verify correct polarity
    pub ldqm: bool,
    pub udqm: bool,
    pub bank: IoBank,
    pub a: u16,
    pub dq: Option<u16>,
}

impl Io {
    pub fn new() -> Io {
        Io {
            command: Command::Nop,
            ldqm: false,
            udqm: false,
            bank: IoBank::Bank0,
            a: 0,
            dq: None,
        }
    }
}

// TODO: Mode registers
pub struct Sdram {
    banks: Box<[Bank]>,

    state: State,
}

impl Sdram {
    pub fn new() -> Sdram {
        Sdram {
            banks: vec![Bank::new(); NUM_BANKS as usize].into(),

            state: State::Idle,
        }
    }

    pub fn clk(&mut self, io: &mut Io) {
        match io.command {
            Command::Active => {
                self.banks[io.bank.index()].active(io.a as u32 & ROW_ADDR_MASK);
            }
            Command::Nop => (), // Do nothing
            Command::Precharge => {
                self.banks[io.bank.index()].precharge();
            }
            Command::Read => {
                self.state = State::Read { bank: io.bank, num_cycles: 0 };
            }
            Command::Write => {
                self.state = State::Write { bank: io.bank, num_cycles: 0 };
            }
            _ => todo!()
        }

        match &mut self.state {
            State::Idle => (), // Do nothing
            State::Read { bank, num_cycles } => {
                // TODO: Test(s)
                if io.dq.is_some() {
                    panic!("Expected no data to be provided for read cycle.");
                }
                io.dq = Some(self.banks[bank.index()].read((io.a as u32).wrapping_add(*num_cycles) & COL_ADDR_MASK));
                *num_cycles += 1;
                if *num_cycles == BURST_LEN {
                    self.state = State::Idle;
                }
            }
            State::Write { bank, num_cycles } => {
                // TODO: Test(s)
                let data = io.dq.expect("No data provided for write cycle.");
                self.banks[bank.index()].write((io.a as u32).wrapping_add(*num_cycles) & COL_ADDR_MASK, data);
                *num_cycles += 1;
                if *num_cycles == BURST_LEN {
                    self.state = State::Idle;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_active_precharge() {
        let mut sdram = Sdram::new();

        // TODO: Initialization

        let mut io = Io::new();
        io.command = Command::Active;
        sdram.clk(&mut io);
        assert!(io.dq.is_none());
        io.command = Command::Precharge;
        sdram.clk(&mut io);
        assert!(io.dq.is_none());
    }

    #[test]
    #[should_panic(expected = "Attempted to activate a row in a bank which already has an active row.")]
    fn single_active_active() {
        let mut sdram = Sdram::new();

        // TODO: Initialization

        let mut io = Io::new();
        io.command = Command::Active;
        sdram.clk(&mut io);
        assert!(io.dq.is_none());
        io.command = Command::Active;
        sdram.clk(&mut io);
    }
}
