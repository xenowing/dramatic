// Alliance AS4C32M16MSA-6BIN
//  8M x 16bits x 4 banks (64MBytes)
//  Assumes 166MHz operation

pub const NUM_ROW_ADDR_BITS: usize = 13;
pub const NUM_COL_ADDR_BITS: usize = 10;
pub const NUM_ROWS: usize = 1 << NUM_ROW_ADDR_BITS;
pub const ROW_ADDR_MASK: usize = NUM_ROWS - 1;
pub const NUM_COLS: usize = 1 << NUM_COL_ADDR_BITS;
pub const COL_ADDR_MASK: usize = NUM_COLS - 1;
pub const NUM_BANKS: usize = 4;
pub const CAS_LATENCY: u32 = 3;
pub const BURST_LEN: u32 = 8; // 128-bit effective word size

#[derive(Clone)]
struct Row {
    cols: Box<[u16]>,
}

impl Row {
    fn new() -> Row {
        Row {
            cols: vec![0; NUM_COLS].into_boxed_slice(),
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
            rows: vec![Row::new(); NUM_ROWS].into_boxed_slice(),
            active_row: None,
        }
    }

    fn active(&mut self, row: usize) {
        if self.active_row.is_some() {
            panic!("Attempted to activate a row in a bank which already has an active row.");
        }

        self.active_row = Some(row);
    }

    fn precharge(&mut self) {
        if self.active_row.is_none() {
            panic!("Attempted to precharge a row in a bank which does not have an active row.");
        }

        self.active_row = None;
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

pub struct Sdram {
    banks: Box<[Bank]>,

    // TODO: Mode registers
}

impl Sdram {
    pub fn new() -> Sdram {
        Sdram {
            banks: vec![Bank::new(); NUM_BANKS].into_boxed_slice(),
        }
    }

    pub fn clk(&mut self, io: &mut Io) {
        match io.command {
            Command::Active => {
                self.banks[io.bank.index()].active(io.a as usize & ROW_ADDR_MASK);
            }
            Command::Precharge => {
                self.banks[io.bank.index()].precharge();
            }
            _ => todo!()
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

    #[test]
    #[should_panic(expected = "Attempted to precharge a row in a bank which does not have an active row.")]
    fn single_precharge() {
        let mut sdram = Sdram::new();

        // TODO: Initialization

        let mut io = Io::new();
        io.command = Command::Precharge;
        sdram.clk(&mut io);
    }
}
