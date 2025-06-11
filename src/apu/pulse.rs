
pub enum PulseRegister {
    R0, R1, R2, R3
}

#[derive(Debug)]
pub struct Pulse {
    duty_cycle: u8,
    length_counter_halt: bool,
    constant_flag: bool,
    envelope_divider: u8,

    sweep_enabled: bool,
    sweep_period: u8,
    sweep_negate: bool,
    sweep_shift: u8,

    period_value: u16,
    length_value: u8,

    length_counter_enabled: bool,

    period_counter: u16,
    duty_sequencer: u8
}


impl Pulse {

    const DUTY_SEQUENCE: [[u8; 8]; 4] = [
        [0,0,0,0,0,0,0,1],
        [0,0,0,0,0,0,1,1],
        [0,0,0,0,1,1,1,1],
        [1,1,1,1,1,1,0,0],
    ];

    pub fn new() -> Pulse {
        Pulse {
            duty_cycle: 0,
            length_counter_halt: false,
            constant_flag: false,
            envelope_divider: 0,

            sweep_enabled: false,
            sweep_period: 0,
            sweep_negate: false,
            sweep_shift: 0,

            period_value: 0,
            length_value: 0,

            length_counter_enabled: false,

            period_counter: 0,
            duty_sequencer: 0,
        }
    }

    pub fn load_registers(&mut self, bytes: [u8; 4]) {
        self.load_register(PulseRegister::R0, bytes[0]);
        self.load_register(PulseRegister::R1, bytes[1]);
        self.load_register(PulseRegister::R2, bytes[2]);
        self.load_register(PulseRegister::R3, bytes[3]);
    }

    pub fn load_register(&mut self, register: PulseRegister, byte: u8) {
        match register {
            PulseRegister::R0 => {
                self.duty_cycle = byte >> 6;
                self.length_counter_halt = byte & 0b0010_0000 > 0;
                self.constant_flag = byte & 0b0001_0000 > 0;
                self.envelope_divider = byte & 0b0000_1111;
            },
            PulseRegister::R1 => {
                self.sweep_enabled = byte & 0b1000_0000 > 0;
                self.sweep_period = (byte & 0b0111_0000) >> 4;
                self.sweep_negate = byte & 0b0000_1000 > 0;
                self.sweep_shift = byte & 0b0000_0111;
            },
            PulseRegister::R2 => {
                self.period_value = (self.period_value & 0xFF00) + (byte as u16);
            },
            PulseRegister::R3 => {
                self.period_value = (self.period_value & 0x00FF) + (((byte & 0b0000_0111) as u16) << 8 );
                self.length_value = byte >> 3;

                self.duty_sequencer = 0;
            },
        }
    }

    pub fn clock_timer(&mut self) {
        if self.period_counter == 0 {
            self.period_counter = self.period_value;
            self.duty_sequencer = self.duty_sequencer.wrapping_sub(1) % 8
        } else {
            self.period_counter -= 1;
        }
        //println!("pulse clock {}, {}", self.period_counter, self.duty_sequencer);
    }

    pub fn clock_length(&mut self) {

    }

    pub fn clock_sweep(&mut self) {

    }

    pub fn clock_envelop(&mut self) {

    }

    pub fn output(&self) -> u8 {
        if self.period_value < 8 {
            0
        } else {
            self.envelope_divider * Pulse::DUTY_SEQUENCE[self.duty_cycle as usize][self.duty_sequencer as usize]
        }
    }
}