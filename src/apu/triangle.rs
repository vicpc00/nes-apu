use crate::apu::LENGTH_TABLE;

pub enum TriangleRegister {
    R0, R1, R2, R3
}

#[derive(Debug)]
pub struct Triangle {
    //Variables set by registers
    enabled: bool,

    control_flag: bool,
    linear_counter_value: u8,

    period_value: u16,
    length_value: u8,

    //Internal Variables
    linear_counter_reload: bool,
    period_counter: u16,
    linear_counter: u8,
    sequence_counter: u8,
    length_counter: u8,

}

impl Triangle {

    const SEQUENCE: [u8; 32] = [
        15, 14, 13, 12, 11, 10,  9,  8,  7,  6,  5,  4,  3,  2,  1,  0,
        0,  1,  2,  3,  4,  5,  6,  7,  8,  9, 10, 11, 12, 13, 14, 15
    ];

    pub fn new() -> Triangle {
        Triangle { 
            enabled: false, 
            control_flag: false, 
            linear_counter_value: 0, 
            period_value: 0, 
            length_value: 0, 
            linear_counter_reload: false,
            period_counter: 0, 
            linear_counter: 0, 
            sequence_counter: 0, 
            length_counter: 0 
        }
    }
    
    pub fn load_registers(&mut self, bytes: [u8; 4]) {
        self.load_register(TriangleRegister::R0, bytes[0]);
        self.load_register(TriangleRegister::R1, bytes[1]);
        self.load_register(TriangleRegister::R2, bytes[2]);
        self.load_register(TriangleRegister::R3, bytes[3]);
    }

    pub fn load_register(&mut self, register: TriangleRegister, byte: u8) {
        match register {
            TriangleRegister::R0 => {
                self.control_flag = byte & 0b1000_0000 > 0;
                self.linear_counter_value = byte & 0b0111_1111;
            },
            TriangleRegister::R1 => {
            },
            TriangleRegister::R2 => {
                self.period_value = (self.period_value & 0xFF00) + (byte as u16);
            },
            TriangleRegister::R3 => {
                self.period_value = (self.period_value & 0x00FF) + (((byte & 0b0000_0111) as u16) << 8 );
                self.length_value = byte >> 3;

                self.length_counter = if self.enabled {
                    LENGTH_TABLE[self.length_value as usize]
                } else {0};
                self.linear_counter_reload = true;
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
            if self.linear_counter > 0 && self.length_counter > 0 {
                self.sequence_counter = (self.sequence_counter + 1) % 32;
            }
        } else {
            self.period_counter -= 1;
        }
        //println!("pulse clock {}, {}", self.period_counter, self.duty_sequencer);
    }

    pub fn clock_linear(&mut self) {
        if self.linear_counter_reload {
            self.linear_counter = self.linear_counter_value;
        } else {
            self.linear_counter_value = self.linear_counter_value.saturating_sub(1);
        }
        if !self.control_flag {
            self.linear_counter_reload = false;
        }
    }

    pub fn clock_length(&mut self) {
        if !self.control_flag && self.length_counter > 0 {
            self.length_counter -= 1
        }
    }

    pub fn output(&self) -> u8 {
        //println!("{}, {}", self.sequence_counter, Triangle::SEQUENCE[self.sequence_counter as usize]);
        Triangle::SEQUENCE[self.sequence_counter as usize]
    }
}