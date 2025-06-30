use crate::apu::memory::Memory;


pub enum DMCRegister {
    R0, R1, R2, R3
}

#[derive(Debug)]
pub struct DMC {
    //Variables set by registers
    enabled: bool,

    irq_enabled: bool,
    loop_flag: bool,
    rate_value: u16,

    length_value: u16,

    start_address: u16,

    current_level: u8,

    //Internal variables
    irq_interrupt: bool,
    rate_counter: u16,
    length_counter: u16,
    address_pointer: u16,

    sample_buffer: u8,
    sample_buffer_empty: bool,
    shift_register: u8,
    bits_counter: u8,

    silence_flag: bool,

    pub request_fetch: bool,

}


impl DMC {

    const RATE_TABLE: [[u16; 16]; 2] = [
        [428, 380, 340, 320, 286, 254, 226, 214, 190, 160, 142, 128, 106,  84,  72,  54], //NTSC
        [398, 354, 316, 298, 276, 236, 210, 198, 176, 148, 132, 118,  98,  78,  66,  50], //PAL
    ];

    pub fn new() -> DMC {
        DMC {
            enabled: false,

            irq_enabled: false,
            loop_flag: false,
            rate_value: DMC::RATE_TABLE[0][0],

            length_value: 0,

            start_address: 0,

            current_level: 0,

            irq_interrupt: false,

            rate_counter: DMC::RATE_TABLE[0][0],
            length_counter: 0,
            address_pointer: 0,

            sample_buffer: 0,
            sample_buffer_empty: true,
            shift_register: 0,
            bits_counter: 8,

            silence_flag: true,

            request_fetch: false,

        }
    }

    pub fn load_registers(&mut self, bytes: [u8; 4]) {
        self.load_register(DMCRegister::R0, bytes[0]);
        self.load_register(DMCRegister::R1, bytes[1]);
        self.load_register(DMCRegister::R2, bytes[2]);
        self.load_register(DMCRegister::R3, bytes[3]);
    }

    pub fn load_register(&mut self, register: DMCRegister, byte: u8) {
        match register {
            DMCRegister::R0 => {
                self.irq_enabled = byte & 0b1000_0000 > 0;
                self.loop_flag = byte & 0b0100_0000 > 0;
                self.rate_value = DMC::RATE_TABLE[0][(byte & 0x0F) as usize];
                self.rate_counter = self.rate_value;
            },
            DMCRegister::R1 => {
                self.current_level = byte & 0x7F;
            },
            DMCRegister::R2 => {
                self.start_address = 0xC000 + ((byte as u16) << 6)
            },
            DMCRegister::R3 => {
                self.length_value = ((byte as u16) << 4) + 1;
            },
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        self.irq_enabled = false;
        if !enabled {
            self.length_counter = 0;
        } else {
            self.start_sample();
        }
    }

    pub fn clock_timer(&mut self) {
        self.rate_counter -= 1;
        if self.rate_counter == 0 {
            self.rate_counter = self.rate_value;
            self.clock_level();
        }

    }

    fn start_sample(&mut self) {
        if self.length_counter == 0 {
            self.address_pointer = self.start_address;
            self.length_counter = self.length_value;
            if self.sample_buffer_empty {
                self.request_fetch = true;
            }
        }
    }

    pub fn fetch_sample(&mut self, memory: &Memory) {
        self.request_fetch = false;
        if self.length_counter > 0 {
            self.sample_buffer = memory.get_byte(self.address_pointer);
            self.sample_buffer_empty = false;
            let (ptr, overflow) = self.address_pointer.overflowing_add(1);
            self.address_pointer = if overflow {0x8000} else {ptr};
            self.length_counter -= 1;
        }

        if self.length_counter == 0 {
            if self.loop_flag {
                self.start_sample();
            } else if self.irq_enabled {
                self.irq_interrupt = true;
            }
        }
    }

    fn clock_level(&mut self) {
        if !self.silence_flag {
            if self.shift_register & 0x01 == 1  && self.current_level < 126 {
                self.current_level += 2
            }
            if self.shift_register & 0x01 == 0  && self.current_level > 1 {
                self.current_level -= 2
            }
        }
        self.shift_register >>= 1;
        self.bits_counter -= 1;
        
        if self.bits_counter == 0 {
            self.bits_counter = 8;
            if !self.sample_buffer_empty {
                self.silence_flag = false;
                self.shift_register = self.sample_buffer;
                self.sample_buffer_empty = true;
                self.request_fetch = true;
            } else {
                self.silence_flag = true;
            }
        }
    }

    pub fn output(&self) -> u8 {
        if self.silence_flag {
            0
        } else {
            self.current_level
        }
    }
}