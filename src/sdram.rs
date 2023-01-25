// Alliance AS4C32M16MSA-6BIN
//  8M x 16bits x 4 banks (64MBytes)
//  Assumes 166MHz operation

extern crate vcd;

use std::{fs, io};

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
#[derive(Debug)]
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
    pub fn from_index(index: usize) -> Option<IoBank> {
        match index {
            0 => Some(IoBank::Bank0),
            1 => Some(IoBank::Bank1),
            2 => Some(IoBank::Bank2),
            3 => Some(IoBank::Bank3),
            _ => None
        }
    }

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
    pub dq_in: Option<u16>,
    dq_out: Option<u16>,
}

impl Io {
    pub fn new() -> Io {
        Io {
            command: Command::Nop,
            ldqm: false,
            udqm: false,
            bank: IoBank::Bank0,
            a: 0,
            dq_in: None,
            dq_out: None,
        }
    }

    pub fn dq(&self) -> Option<u16> {
        self.check_dq_bus_conflict();

        self.dq_in.or(self.dq_out)
    }

    fn check_dq_bus_conflict(&self) {
        if self.dq_in.is_some() && self.dq_out.is_some() {
            // TODO: Test(s)
            panic!("DQ bus conflict occurred.");
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
            panic!("tRRD violated.");
        }

        self.is_active = true;
        self.cycles_since_activation = 0;
    }
}

struct Trace {
    w: vcd::Writer<io::BufWriter<fs::File>>,

    clk_id: vcd::IdCode,
    command_id: vcd::IdCode,
    ldqm_id: vcd::IdCode,
    udqm_id: vcd::IdCode,
    bank_id: vcd::IdCode,
    a_id: vcd::IdCode,
    dq_id: vcd::IdCode,

    time_stamp: u64,
}

// TODO: Mode registers
pub struct Sdram {
    banks: Box<[Bank]>,

    state: State,
    dq_out_pipeline: Box<[Option<u16>]>,

    t_rrd_tester: TRrdTester,

    trace: Option<Trace>,
}

trait Bits {
    fn bits(&self) -> Box<[vcd::Value]>;
}

impl Bits for u16 {
    fn bits(&self) -> Box<[vcd::Value]> {
        let mut ret = vec![vcd::Value::X; 16];
        for (i, value) in ret.iter_mut().enumerate() {
            *value = match (self >> (15 - i)) & 1 {
                0 => vcd::Value::V0,
                1 => vcd::Value::V1,
                _ => unreachable!()
            };
        }
        ret.into()
    }
}

impl Bits for IoBank {
    fn bits(&self) -> Box<[vcd::Value]> {
        let index = self.index();
        let mut ret = vec![vcd::Value::X; 2];
        for (i, value) in ret.iter_mut().enumerate() {
            *value = match (index >> (1 - i)) & 1 {
                0 => vcd::Value::V0,
                1 => vcd::Value::V1,
                _ => unreachable!()
            };
        }
        ret.into()
    }
}

impl Sdram {
    pub fn new(trace_file_name_prefix: Option<&str>) -> io::Result<Sdram> {
        Ok(Sdram {
            banks: vec![Bank::new(); NUM_BANKS as usize].into(),

            state: State::Idle,
            dq_out_pipeline: vec![None; CAS_LATENCY as usize - 1].into(),

            t_rrd_tester: TRrdTester::new(),

            trace: if let Some(prefix) = trace_file_name_prefix {
                let path = format!("vcd/{}.vcd", prefix);
                println!("Writing trace to {}", path);
                let file = fs::File::create(path)?;
                let mut w = vcd::Writer::new(io::BufWriter::new(file));

                w.timescale(CLOCK_PERIOD_NS / 2, vcd::TimescaleUnit::NS)?;

                w.add_module("sdram")?;

                let clk_id = w.add_wire(1, "clk")?;
                let command_id = w.add_var(vcd::VarType::String, 4 /* TODO: Verify correct width */, "command", None)?;
                let ldqm_id = w.add_wire(1, "ldqm")?;
                let udqm_id = w.add_wire(1, "udqm")?;
                let bank_id = w.add_wire(2, "bank")?;
                let a_id = w.add_wire(NUM_ROW_ADDR_BITS, "a")?;
                let dq_id = w.add_wire(16, "dq")?;

                w.upscope()?;
                w.enddefinitions()?;

                let time_stamp = 0;
                w.timestamp(time_stamp)?;

                Some(Trace {
                    w,

                    clk_id,
                    command_id,
                    ldqm_id,
                    udqm_id,
                    bank_id,
                    a_id,
                    dq_id,

                    time_stamp,
                })
            } else {
                None
            },
        })
    }

    pub fn clk(&mut self, io: &mut Io) -> io::Result<()> {
        io.check_dq_bus_conflict();

        if let Some(trace) = &mut self.trace {
            trace.w.change_scalar(trace.clk_id, false)?;

            trace.w.change_string(trace.command_id, &format!("{:?}", io.command))?;
            trace.w.change_scalar(trace.ldqm_id, io.ldqm)?;
            trace.w.change_scalar(trace.udqm_id, io.udqm)?;
            trace.w.change_vector(trace.bank_id, &io.bank.bits())?;
            trace.w.change_vector(trace.a_id, &io.a.bits())?;
            trace.w.change_vector(trace.dq_id, &io.dq().map_or_else(
                || vec![vcd::Value::Z; 16].into(),
                |dq| dq.bits(),
            ))?;

            trace.time_stamp += 1;
            trace.w.timestamp(trace.time_stamp)?;
            trace.w.change_scalar(trace.clk_id, true)?;
            trace.time_stamp += 1;
            trace.w.timestamp(trace.time_stamp)?;
        }

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
                self.state = State::Read { bank: io.bank, num_cycles: 0 };
            }
            Command::Write => {
                self.state = State::Write { bank: io.bank, num_cycles: 0 };
            }
            _ => todo!()
        }

        let mut next_dq_out = None;

        match &mut self.state {
            State::Idle => (), // Do nothing
            State::Read { bank, num_cycles } => {
                // TODO: Technically we only need to test timings for the first read cycle, but doing them each time doesn't hurt
                let data = self.banks[bank.index()].read((io.a as u32).wrapping_add(*num_cycles) & COL_ADDR_MASK);
                next_dq_out = Some(data);
                *num_cycles += 1;
                if *num_cycles == BURST_LEN {
                    self.state = State::Idle;
                }
            }
            State::Write { bank, num_cycles } => {
                // TODO: Test(s)
                let data = io.dq_in.expect("No data provided for write cycle.");
                self.banks[bank.index()].write((io.a as u32).wrapping_add(*num_cycles) & COL_ADDR_MASK, data);
                *num_cycles += 1;
                if *num_cycles == BURST_LEN {
                    self.state = State::Idle;
                }
            }
        }

        let last = self.dq_out_pipeline[self.dq_out_pipeline.len() - 1];
        io.dq_out = last;
        for i in (1..self.dq_out_pipeline.len()).rev() {
            self.dq_out_pipeline[i] = self.dq_out_pipeline[i - 1];
        }
        self.dq_out_pipeline[0] = next_dq_out;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_active_precharge() -> io::Result<()> {
        let mut sdram = Sdram::new(Some("Sdram__one_active_precharge"))?;

        // TODO: Initialization

        let mut io = Io::new();
        io.command = Command::Active;
        for _ in 0..T_RAS_MIN_CYCLES {
            sdram.clk(&mut io)?;
            assert!(io.dq().is_none());
            io.command = Command::Nop;
        }
        io.command = Command::Precharge;
        sdram.clk(&mut io)?;
        assert!(io.dq().is_none());

        Ok(())
    }

    #[test]
    #[should_panic(expected = "Attempted to activate a row in a bank which already has an active row.")]
    fn one_active_active() {
        let mut sdram = Sdram::new(Some("Sdram__one_active_active")).unwrap();

        // TODO: Initialization

        let mut io = Io::new();
        io.command = Command::Active;
        for _ in 0..T_RRD_CYCLES {
            sdram.clk(&mut io).unwrap();
            assert!(io.dq().is_none());
            io.command = Command::Nop;
        }
        io.command = Command::Active;
        sdram.clk(&mut io).unwrap();
    }

    #[test]
    fn two_actives_separate_banks() -> io::Result<()> {
        let mut sdram = Sdram::new(Some("Sdram__two_actives_separate_banks"))?;

        // TODO: Initialization

        let mut io = Io::new();
        io.command = Command::Active;
        io.bank = IoBank::Bank0;
        for _ in 0..T_RRD_CYCLES {
            sdram.clk(&mut io)?;
            assert!(io.dq().is_none());
            io.command = Command::Nop;
        }
        io.command = Command::Active;
        io.bank = IoBank::Bank1;
        sdram.clk(&mut io)?;

        Ok(())
    }

    #[test]
    #[should_panic(expected = "tRAS min violated.")]
    fn violate_t_ras_min() {
        let mut sdram = Sdram::new(Some("Sdram__violate_t_ras_min")).unwrap();

        // TODO: Initialization

        let mut io = Io::new();
        io.command = Command::Active;
        sdram.clk(&mut io).unwrap();
        assert!(io.dq().is_none());
        io.command = Command::Precharge;
        sdram.clk(&mut io).unwrap();
    }

    #[test]
    #[should_panic(expected = "tRAS max violated.")]
    fn violate_t_ras_max() {
        let mut sdram = Sdram::new(Some("Sdram__violate_t_ras_max")).unwrap();

        // TODO: Initialization

        let mut io = Io::new();
        io.command = Command::Active;
        for _ in 0..T_RAS_MAX_CYCLES + 1 {
            sdram.clk(&mut io).unwrap();
            assert!(io.dq().is_none());
            io.command = Command::Nop;
        }
    }

    #[test]
    #[should_panic(expected = "tRC violated.")]
    fn violate_t_rc() {
        let mut sdram = Sdram::new(Some("Sdram__violate_t_rc")).unwrap();

        // TODO: Initialization

        let mut io = Io::new();
        io.command = Command::Active;
        for _ in 0..T_RAS_MIN_CYCLES {
            sdram.clk(&mut io).unwrap();
            assert!(io.dq().is_none());
            io.command = Command::Nop;
        }
        io.command = Command::Precharge;
        sdram.clk(&mut io).unwrap();
        assert!(io.dq().is_none());
        io.command = Command::Active;
        sdram.clk(&mut io).unwrap();
    }

    #[test]
    #[should_panic(expected = "tRCD violated.")]
    fn violate_t_rcd_read() {
        let mut sdram = Sdram::new(Some("Sdram__violate_t_rcd_read")).unwrap();

        // TODO: Initialization

        let mut io = Io::new();
        io.command = Command::Active;
        sdram.clk(&mut io).unwrap();
        assert!(io.dq().is_none());
        io.command = Command::Read;
        sdram.clk(&mut io).unwrap();
    }

    #[test]
    #[should_panic(expected = "tRCD violated.")]
    fn violate_t_rcd_write() {
        let mut sdram = Sdram::new(Some("Sdram__violate_t_rcd_write")).unwrap();

        // TODO: Initialization

        let mut io = Io::new();
        io.command = Command::Active;
        sdram.clk(&mut io).unwrap();
        assert!(io.dq().is_none());
        io.command = Command::Write;
        io.dq_in = Some(0xbeef);
        sdram.clk(&mut io).unwrap();
    }

    #[test]
    #[should_panic(expected = "tRP violated.")]
    fn violate_t_rp() {
        let mut sdram = Sdram::new(Some("Sdram__violate_t_rp")).unwrap();

        // TODO: Initialization

        let mut io = Io::new();
        io.command = Command::Active;
        for _ in 0..T_RC_CYCLES {
            sdram.clk(&mut io).unwrap();
            assert!(io.dq().is_none());
            io.command = Command::Nop;
        }
        io.command = Command::Precharge;
        sdram.clk(&mut io).unwrap();
        assert!(io.dq().is_none());
        io.command = Command::Active;
        sdram.clk(&mut io).unwrap();
    }

    #[test]
    #[should_panic(expected = "tRRD violated.")]
    fn violate_t_rrd() {
        let mut sdram = Sdram::new(Some("Sdram__violate_t_rrd")).unwrap();

        // TODO: Initialization

        let mut io = Io::new();
        io.command = Command::Active;
        io.bank = IoBank::Bank0;
        sdram.clk(&mut io).unwrap();
        assert!(io.dq().is_none());
        io.command = Command::Active;
        io.bank = IoBank::Bank1;
        sdram.clk(&mut io).unwrap();
    }
}
