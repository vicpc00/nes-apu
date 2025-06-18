use crate::apu::LENGTH_TABLE;
use crate::apu::envelope::Envelope;

pub enum PulseRegister {
    R0, R1, R2, R3
}

#[derive(Debug)]
pub struct Pulse {
    //Variables set by registers
    enabled: bool,

    duty_cycle: u8,
    length_counter_halt: bool,
    constant_flag: bool,
    envelope_divider: u8,

    sweep_enabled: bool,
    sweep_period_value: u8,
    sweep_negate: bool,
    sweep_shift: u8,

    period_value: u16,
    length_value: u8,

    //Internal Variables
    envelope: Envelope,

    period_counter: u16,
    duty_sequencer: u8,
    length_counter: u8,
    sweep_counter: u8,
    sweep_target_period: u16,
    sweep_use_two_complemet: bool,
    sweep_reload: bool,
}


impl Pulse {

    const DUTY_SEQUENCE: [[u8; 8]; 4] = [
        [0,0,0,0,0,0,0,1],
        [0,0,0,0,0,0,1,1],
        [0,0,0,0,1,1,1,1],
        [1,1,1,1,1,1,0,0],
    ];

    pub fn new(is_channel_1: bool) -> Pulse {
        Pulse {
            enabled: false,

            duty_cycle: 0,
            length_counter_halt: false,
            constant_flag: false,
            envelope_divider: 0,

            sweep_enabled: false,
            sweep_period_value: 0,
            sweep_negate: false,
            sweep_shift: 0,

            period_value: 0,
            length_value: 0,
            length_counter: 0,
            
            envelope: Envelope::new(),

            period_counter: 0,
            duty_sequencer: 0,

            sweep_counter: 0,
            sweep_target_period: 0,
            sweep_use_two_complemet: !is_channel_1,
            sweep_reload: false,
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

                self.envelope.load_register(byte);
            },
            PulseRegister::R1 => {
                self.sweep_enabled = byte & 0b1000_0000 > 0;
                self.sweep_period_value = (byte & 0b0111_0000) >> 4;
                self.sweep_negate = byte & 0b0000_1000 > 0;
                self.sweep_shift = byte & 0b0000_0111;

                self.update_sweep_target_period();
                self.sweep_counter = self.sweep_period_value;
                self.sweep_reload = true 
            },
            PulseRegister::R2 => {
                self.period_value = (self.period_value & 0xFF00) + (byte as u16);
                self.update_sweep_target_period();
            },
            PulseRegister::R3 => {
                self.period_value = (self.period_value & 0x00FF) + (((byte & 0b0000_0111) as u16) << 8 );
                self.length_value = byte >> 3;

                self.length_counter = if self.enabled {
                    LENGTH_TABLE[self.length_value as usize]
                } else {0};
                self.duty_sequencer = 0;
                self.update_sweep_target_period();
                self.envelope.set_start();
            },
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            self.length_counter = 0;
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
        if !self.length_counter_halt && self.length_counter > 0 {
            self.length_counter -= 1
        }

    }

    pub fn clock_sweep(&mut self) {
        if self.sweep_counter == 0{
            self.sweep_counter = self.sweep_period_value;
            if self.sweep_enabled && self.sweep_shift > 0 && !self.is_mute_sweep() {
                self.period_value = self.sweep_target_period;
                self.update_sweep_target_period();
                //println!("{}", 1789773./(16.*(1. + self.period_value as f32)))
            }
        } else if self.sweep_reload {
            self.sweep_counter = self.sweep_period_value;
            self.sweep_reload = false;
        } else {
            self.sweep_counter -= 1;
        }
    }

    pub fn clock_envelop(&mut self) {
        self.envelope.clock();
    }

    pub fn update_sweep_target_period(&mut self) {
        let delta: u16 = self.period_value >> self.sweep_shift;
        if self.sweep_negate {
            self.sweep_target_period = self.period_value.saturating_sub(delta);
            if !self.sweep_use_two_complemet {
                self.sweep_target_period = self.period_value.saturating_sub(1);
            }
        }
        else {
            self.sweep_target_period += delta;
        }
    }

    fn is_mute_sweep(&self) -> bool{
        self.period_value < 8 || self.sweep_target_period > 0x7FF
    }

    fn is_mute(&self) -> bool {
        !self.enabled || self.length_counter == 0 || self.is_mute_sweep()
    }

    pub fn output(&self) -> u8 {
        if self.is_mute() {
            0
        } else {
            self.envelope.volume() * Pulse::DUTY_SEQUENCE[self.duty_cycle as usize][self.duty_sequencer as usize]
        }
    }
}