
use crate::apu::LENGTH_TABLE;
use crate::apu::envelope::Envelope;



pub enum NoiseRegister {
    R0, R1, R2, R3
}

#[derive(Debug)]
pub struct Noise {
    enabled: bool,

    length_counter_halt: bool,
    envelope: Envelope,

    mode_flag: bool,
    period_value: u16,

    length_value: u8,

    length_counter: u8,
    period_counter: u16,
    shift_register: u16,
}

impl Noise {

    const RATE_TABLE: [[u16; 16]; 2] = [
        [4, 8, 16, 32, 64, 96, 128, 160, 202, 254, 380, 508, 762, 1016, 2034, 4068], //NTSC
        [4, 8, 14, 30, 60, 88, 118, 148, 188, 236, 354, 472, 708,  944, 1890, 3778], //PAL
    ];

    pub fn new() -> Noise {
        Noise {
            enabled: false,

            length_counter_halt: false,
            envelope: Envelope::new(),

            mode_flag: false,
            period_value: 0,

            length_value: 0,

            length_counter: 0,
            period_counter: 0,
            shift_register: 1,
        }   
    }

    pub fn load_registers(&mut self, bytes: [u8; 4]) {
        self.load_register(NoiseRegister::R0, bytes[0]);
        self.load_register(NoiseRegister::R1, bytes[1]);
        self.load_register(NoiseRegister::R2, bytes[2]);
        self.load_register(NoiseRegister::R3, bytes[3]);
    }

    pub fn load_register(&mut self, register: NoiseRegister, byte: u8) {
        match register {
            NoiseRegister::R0 => {
                self.length_counter_halt = byte & 0b0010_0000 > 0;

                self.envelope.load_register(byte);
            },
            NoiseRegister::R1 => {
            },
            NoiseRegister::R2 => {
                self.mode_flag = byte & 0b1000_0000 > 0;
                let index = (byte & 0b0000_1111) as usize;
                self.period_value = Noise::RATE_TABLE[0][index];
                self.period_counter = self.period_value
            },
            NoiseRegister::R3 => {
                self.length_value = byte >> 3;

                self.length_counter = if self.enabled {
                    LENGTH_TABLE[self.length_value as usize]
                } else {0};
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
            
            let b0 = self.shift_register & 0b0000_0001;
            let b1 = if self.mode_flag {
                (self.shift_register & 0b0100_0000) >> 6
            } else {
                (self.shift_register & 0b0000_0010) >> 1
            };
            let feedback = b0 ^ b1;
            self.shift_register = (self.shift_register >> 1) | (feedback << 14)

        } else {
            self.period_counter -= 1;
        }
    }

    pub fn clock_length(&mut self) {
        if !self.length_counter_halt && self.length_counter > 0 {
            self.length_counter -= 1
        }

    }

    pub fn clock_envelop(&mut self) {
        self.envelope.clock();
    }

    pub fn output(&self) -> u8 {
        if self.length_counter == 0 {
            0
        } else {
            self.envelope.volume() * ((self.shift_register & 0b0000_0001) as u8)
        }
    }
    
}