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

const fn div_ceil(x: u32, y: u32) -> u32 {
    (x + (y - 1)) / y
}

const CLOCK_PERIOD_NS: u32 = 6;

const T_RAS_MIN_NS: u32 = 48;
const T_RAS_MIN_CYCLES: u32 = div_ceil(T_RAS_MIN_NS, CLOCK_PERIOD_NS);
const T_RAS_MAX_NS: u32 = 100000;
const T_RAS_MAX_CYCLES: u32 = div_ceil(T_RAS_MAX_NS, CLOCK_PERIOD_NS);

const T_RC_NS: u32 = 60;
const T_RC_CYCLES: u32 = div_ceil(T_RC_NS, CLOCK_PERIOD_NS);

const T_RCD_NS: u32 = 18;
pub const T_RCD_CYCLES: u32 = div_ceil(T_RCD_NS, CLOCK_PERIOD_NS);

const T_RP_NS: u32 = 18;
pub const T_RP_CYCLES: u32 = div_ceil(T_RP_NS, CLOCK_PERIOD_NS);

const T_RRD_NS: u32 = 12;
const T_RRD_CYCLES: u32 = div_ceil(T_RRD_NS, CLOCK_PERIOD_NS);

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
struct TRasTester {
    is_active: bool,
    cycles_since_activation: u32,
}

impl TRasTester {
    fn new() -> TRasTester {
        TRasTester {
            is_active: false,
            cycles_since_activation: 0,
        }
    }

    fn clk(&mut self) {
        if !self.is_active {
            return;
        }

        self.cycles_since_activation += 1;

        // The datasheet claims a row can be active for an "indefinite period" after tRAS
        //  min is met, but it still lists a max value, and hitting that is probably
        //  indicative of a refresh logic error anyways, so let's still test for it.
        // TODO: Double-check that the max value is inclusive (and adjust if not)
        if self.cycles_since_activation >= T_RAS_MAX_CYCLES {
            panic!("tRAS max violated.");
        }
    }

    fn active(&mut self) {
        self.is_active = true;
        self.cycles_since_activation = 0;
    }

    fn precharge(&mut self) {
        if self.cycles_since_activation < T_RAS_MIN_CYCLES {
            // TODO: Test(s)
            panic!("tRAS min violated.");
        }

        self.is_active = false;
    }
}

#[derive(Clone)]
struct TRcTester {
    is_active: bool,
    cycles_since_activation: u32,
}

impl TRcTester {
    fn new() -> TRcTester {
        TRcTester {
            is_active: false,
            cycles_since_activation: 0,
        }
    }

    fn clk(&mut self) {
        if !self.is_active {
            return;
        }

        self.cycles_since_activation += 1;

        if self.cycles_since_activation >= T_RC_CYCLES {
            self.is_active = false;
        }
    }

    fn active(&mut self) {
        if self.is_active {
            // TODO: Test(s)
            panic!("tRC violated.");
        }

        self.is_active = true;
        self.cycles_since_activation = 0;
    }
}

#[derive(Clone)]
struct TRcdTester {
    is_active: bool,
    cycles_since_activation: u32,
}

impl TRcdTester {
    fn new() -> TRcdTester {
        TRcdTester {
            is_active: false,
            cycles_since_activation: 0,
        }
    }

    fn clk(&mut self) {
        if !self.is_active {
            return;
        }

        self.cycles_since_activation += 1;

        if self.cycles_since_activation >= T_RCD_CYCLES {
            self.is_active = false;
        }
    }

    fn active(&mut self) {
        self.is_active = true;
        self.cycles_since_activation = 0;
    }

    fn read_or_write(&mut self) {
        if !self.is_active {
            return;
        }

        // TODO: Test(s)
        panic!("tRCD violated.");
    }
}

#[derive(Clone)]
struct TRpTester {
    is_active: bool,
    cycles_since_activation: u32,
}

impl TRpTester {
    fn new() -> TRpTester {
        TRpTester {
            is_active: false,
            cycles_since_activation: 0,
        }
    }

    fn clk(&mut self) {
        if !self.is_active {
            return;
        }

        self.cycles_since_activation += 1;

        if self.cycles_since_activation >= T_RP_CYCLES {
            self.is_active = false;
        }
    }

    fn precharge(&mut self) {
        self.test();

        self.is_active = true;
        self.cycles_since_activation = 0;
    }

    fn active_or_read_or_write(&mut self) {
        self.test();
    }

    fn test(&mut self) {
        if !self.is_active {
            return;
        }

        // TODO: Test(s)
        panic!("tRP violated.");
    }
}

#[derive(Clone)]
struct Bank {
    rows: Box<[Row]>,
    active_row: Option<usize>,

    t_ras_tester: TRasTester,
    t_rc_tester: TRcTester,
    t_rcd_tester: TRcdTester,
    t_rp_tester: TRpTester,
}

impl Bank {
    fn new() -> Bank {
        Bank {
            rows: vec![Row::new(); NUM_ROWS as usize].into(),
            active_row: None,

            t_ras_tester: TRasTester::new(),
            t_rc_tester: TRcTester::new(),
            t_rcd_tester: TRcdTester::new(),
            t_rp_tester: TRpTester::new(),
        }
    }

    fn active(&mut self, row_addr: u32) {
        if self.active_row.is_some() {
            panic!("Attempted to activate a row in a bank which already has an active row.");
        }

        self.active_row = Some(row_addr as _);

        self.t_ras_tester.active();
        self.t_rc_tester.active();
        self.t_rcd_tester.active();
        self.t_rp_tester.active_or_read_or_write();
    }

    fn precharge(&mut self) {
        if self.active_row.is_none() {
            return;
        }

        self.active_row = None;

        self.t_ras_tester.precharge();
        self.t_rp_tester.precharge();
    }

    fn read(&mut self, col_addr: u32) -> u16 {
        self.t_rcd_tester.read_or_write();
        self.t_rp_tester.active_or_read_or_write();

        // TODO: Test(s)
        let active_row = self.active_row.expect("Attempted to read from a column in a bank which does not currently have an active row.");
        self.rows[active_row as usize].cols[col_addr as usize]
    }

    fn write(&mut self, col_addr: u32, data: u16) {
        self.t_rcd_tester.read_or_write();
        self.t_rp_tester.active_or_read_or_write();

        // TODO: Test(s)
        let active_row = self.active_row.expect("Attempted to write to a column in a bank which does not currently have an active row.");
        self.rows[active_row as usize].cols[col_addr as usize] = data;
    }

    fn clk(&mut self) {
        self.t_ras_tester.clk();
        self.t_rc_tester.clk();
        self.t_rcd_tester.clk();
        self.t_rp_tester.clk();
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
    Read { bank: IoBank, data: u128, num_cycles: u32 },
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

struct TRrdTester {
    is_active: bool,
    cycles_since_activation: u32,
}

impl TRrdTester {
    fn new() -> TRrdTester {
        TRrdTester {
            is_active: false,
            cycles_since_activation: 0,
        }
    }

    fn clk(&mut self) {
        if !self.is_active {
            return;
        }

        self.cycles_since_activation += 1;

        if self.cycles_since_activation >= T_RRD_CYCLES {
            self.is_active = false;
        }
    }

    fn active(&mut self) {
        if self.is_active {
            // TODO: Test(s)
            panic!("tRRD violated");
        }

        self.is_active = true;
        self.cycles_since_activation = 0;
    }
}

// TODO: Mode registers
pub struct Sdram {
    banks: Box<[Bank]>,

    state: State,

    t_rrd_tester: TRrdTester,
}

impl Sdram {
    pub fn new() -> Sdram {
        Sdram {
            banks: vec![Bank::new(); NUM_BANKS as usize].into(),

            state: State::Idle,

            t_rrd_tester: TRrdTester::new(),
        }
    }

    pub fn clk(&mut self, io: &mut Io) {
        for bank in &mut *self.banks {
            bank.clk();
        }
        self.t_rrd_tester.clk();

        match io.command {
            Command::Active => {
                self.t_rrd_tester.active();

                self.banks[io.bank.index()].active(io.a as u32 & ROW_ADDR_MASK);
            }
            Command::Nop => (), // Do nothing
            Command::Precharge => {
                self.banks[io.bank.index()].precharge();
            }
            Command::Read => {
                self.state = State::Read { bank: io.bank, data: 0, num_cycles: 0 };
            }
            Command::Write => {
                self.state = State::Write { bank: io.bank, num_cycles: 0 };
            }
            _ => todo!()
        }

        match &mut self.state {
            State::Idle => (), // Do nothing
            State::Read { bank, data, num_cycles } => {
                // Perform read immediately to test that required timing is met
                if *num_cycles == 0 {
                    for i in 0..BURST_LEN {
                        *data |= (self.banks[bank.index()].read((io.a as u32).wrapping_add(i) & COL_ADDR_MASK) as u128) << (i * 16);
                    }
                }
                if *num_cycles >= CAS_LATENCY {
                    // TODO: Test(s)
                    if io.dq.is_some() {
                        panic!("Expected no data to be provided for read cycle.");
                    }
                    io.dq = Some((*data >> ((*num_cycles - CAS_LATENCY) * 16)) as _);
                }
                *num_cycles += 1;
                if *num_cycles == CAS_LATENCY + BURST_LEN {
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
    fn one_active_precharge() {
        let mut sdram = Sdram::new();

        // TODO: Initialization

        let mut io = Io::new();
        io.command = Command::Active;
        for _ in 0..T_RAS_MIN_CYCLES {
            sdram.clk(&mut io);
            assert!(io.dq.is_none());
            io.command = Command::Nop;
        }
        io.command = Command::Precharge;
        sdram.clk(&mut io);
        assert!(io.dq.is_none());
    }

    #[test]
    #[should_panic(expected = "Attempted to activate a row in a bank which already has an active row.")]
    fn one_active_active() {
        let mut sdram = Sdram::new();

        // TODO: Initialization

        let mut io = Io::new();
        io.command = Command::Active;
        for _ in 0..T_RRD_CYCLES {
            sdram.clk(&mut io);
            assert!(io.dq.is_none());
            io.command = Command::Nop;
        }
        io.command = Command::Active;
        sdram.clk(&mut io);
    }

    #[test]
    fn two_actives_separate_banks() {
        let mut sdram = Sdram::new();

        // TODO: Initialization

        let mut io = Io::new();
        io.command = Command::Active;
        io.bank = IoBank::Bank0;
        for _ in 0..T_RRD_CYCLES {
            sdram.clk(&mut io);
            assert!(io.dq.is_none());
            io.command = Command::Nop;
        }
        io.command = Command::Active;
        io.bank = IoBank::Bank1;
        sdram.clk(&mut io);
    }

    #[test]
    #[should_panic(expected = "tRAS max violated.")]
    fn violates_t_ras_max() {
        let mut sdram = Sdram::new();

        // TODO: Initialization

        let mut io = Io::new();
        io.command = Command::Active;
        io.bank = IoBank::Bank0;
        for _ in 0..T_RAS_MAX_CYCLES + 1 {
            sdram.clk(&mut io);
            assert!(io.dq.is_none());
            io.command = Command::Nop;
        }
    }
}
