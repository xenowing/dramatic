use dramatic::sdram::*;

use kaze::*;

// TODO: De-dupe with xenowing rtl equivalent
pub struct ReplicaPort<'a> {
    pub bus_enable: &'a Input<'a>,
    pub bus_addr: &'a Input<'a>,
    pub bus_write: &'a Input<'a>,
    pub bus_write_data: &'a Input<'a>,
    pub bus_write_byte_enable: &'a Input<'a>,
    pub bus_ready: &'a Output<'a>,
    pub bus_read_data: &'a Output<'a>,
    pub bus_read_data_valid: &'a Output<'a>,
}

const NUM_DATA_BITS: u32 = 128;
const NUM_TRANSACTION_ADDR_BITS: u32 = NUM_ROW_ADDR_BITS + NUM_COL_ADDR_BITS - NUM_BURST_ADDR_BITS;

// TODO: Consider a name that reflects the target part(s)
pub struct SdramController<'a> {
    pub m: &'a Module<'a>,
    pub client_port: ReplicaPort<'a>,
    // TODO: SDRAM IOs
}

impl<'a> SdramController<'a> {
    pub fn new(
        instance_name: impl Into<String>,
        p: &'a impl ModuleParent<'a>,
    ) -> SdramController<'a> {
        let m = p.module(instance_name, "SdramController");

        let bus_addr_bits = NUM_BANK_ADDR_BITS + NUM_TRANSACTION_ADDR_BITS;
        let replica_bus_enable = m.input("replica_bus_enable", 1);
        let replica_bus_addr = m.input("replica_bus_addr", bus_addr_bits);
        let replica_bus_write = m.input("replica_bus_write", 1);
        let replica_bus_write_data = m.input("replica_bus_write_data", NUM_DATA_BITS);
        let replica_bus_write_byte_enable =
            m.input("replica_bus_write_byte_enable", NUM_DATA_BITS / 8);

        // TODO: Some kind of "split" operator where input is an index and return is a (max, index] and (index, 0] Signal tuple
        let bank_addr = replica_bus_addr.bits(
            NUM_BANK_ADDR_BITS + NUM_TRANSACTION_ADDR_BITS,
            NUM_TRANSACTION_ADDR_BITS,
        );
        let transaction_addr = replica_bus_addr.bits(NUM_TRANSACTION_ADDR_BITS, 0);

        let bank_machines = (0..NUM_BANKS)
            .map(|i| BankMachine::new(format!("bank_machine_{}", i), m))
            .collect::<Vec<_>>();

        // TODO: Move transactions into bank machines
        // TODO: Move transactions from bank machines to sequencers
        // TODO: Sequencers
        // TODO: Refresh

        SdramController {
            m,
            client_port: ReplicaPort {
                bus_enable: replica_bus_enable,
                bus_addr: replica_bus_addr,
                bus_write: replica_bus_write,
                bus_write_data: replica_bus_write_data,
                bus_write_byte_enable: replica_bus_write_byte_enable,
                bus_ready: m.output("bus_ready", replica_bus_ready),
                bus_read_data: m.output("bus_read_data", replica_bus_read_data),
                bus_read_data_valid: m.output("bus_read_data_valid", replica_bus_read_data_valid),
            },
        }
    }
}

struct BankMachine<'a> {
    m: &'a Module<'a>,

    pub current_transaction_ingress_ready: &'a Output<'a>,
    pub current_transaction_ingress_valid: &'a Input<'a>,
    pub current_transaction_ingress_addr: &'a Input<'a>,
    pub current_transaction_ingress_write: &'a Input<'a>,
    pub current_transaction_ingress_write_data: &'a Input<'a>,
    pub current_transaction_ingress_write_byte_enable: &'a Input<'a>,

    pub current_transaction_egress_ready: &'a Input<'a>,
    pub current_transaction_egress_valid: &'a Output<'a>,
    pub current_transaction_egress_addr: &'a Output<'a>,
    pub current_transaction_egress_write: &'a Output<'a>,
    pub current_transaction_egress_write_data: &'a Output<'a>,
    pub current_transaction_egress_write_byte_enable: &'a Output<'a>,

    pub current_bank_valid: &'a Output<'a>,
    pub current_bank_row: &'a Output<'a>,
}

impl<'a> BankMachine<'a> {
    pub fn new(instance_name: impl Into<String>, p: &'a impl ModuleParent<'a>) -> BankMachine<'a> {
        let m = p.module(instance_name, "BankMachine");

        let current_transaction_ingress_valid = m.input("current_transaction_ingress_valid", 1);

        let current_transaction_egress_ready = m.input("current_transaction_egress_ready", 1);

        let current_transaction_valid_reg = m.reg("current_transaction_valid_reg", 1);
        current_transaction_valid_reg.default_value(false);
        current_transaction_valid_reg.drive_next(
            if_(
                current_transaction_egress_ready,
                current_transaction_ingress_valid,
            )
            .else_(current_transaction_valid_reg),
        );
        let current_transaction_addr_reg =
            m.reg("current_transaction_addr_reg", NUM_TRANSACTION_ADDR_BITS);
        let current_transaction_write_reg = m.reg("current_transaction_write_reg", 1);
        let current_transaction_write_data_reg =
            m.reg("current_transaction_write_data_reg", NUM_DATA_BITS);
        let current_transaction_write_byte_enable_reg = m.reg(
            "current_transaction_write_byte_enable_reg",
            NUM_DATA_BITS / 8,
        );

        let current_transaction_ingress_ready =
            !current_transaction_valid_reg | current_transaction_egress_ready;

        let current_bank_valid_reg = m.reg("current_bank_valid_reg", 1);
        current_bank_valid_reg.default_value(false);
        let current_bank_row_reg = m.reg("current_bank_row_reg", NUM_ROW_ADDR_BITS);

        BankMachine {
            m,

            current_transaction_egress_ready,
            current_transaction_egress_valid: m.output(
                "current_transaction_egress_valid",
                current_transaction_valid_reg,
            ),
            current_transaction_egress_addr: m.output(
                "current_transaction_egress_addr",
                current_transaction_addr_reg,
            ),
            current_transaction_egress_write: m.output(
                "current_transaction_egress_write",
                current_transaction_write_reg,
            ),
            current_transaction_egress_write_data: m.output(
                "current_transaction_egress_write_data",
                current_transaction_write_data_reg,
            ),
            current_transaction_egress_write_byte_enable: m.output(
                "current_transaction_egress_write_byte_enable",
                current_transaction_write_byte_enable_reg,
            ),

            current_bank_valid: m.output("current_bank_valid", current_bank_valid_reg),
            current_bank_row: m.output("current_bank_row", current_bank_row_reg),
        }
    }
}
