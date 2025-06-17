
#[derive(Debug)]
pub struct Envelope {
    loop_flag: bool,
    constant_vol_flag: bool,
    volume_value: u8,

    divider_counter: u8,
    decay_counter: u8,
    start_flag: bool
}

impl Envelope {
    pub fn new() -> Envelope {
        Envelope { 
            loop_flag: false, 
            constant_vol_flag: false,
            volume_value: 0, 
            divider_counter: 0, 
            decay_counter: 0, 
            start_flag: false 
        }
    }

    pub fn load_register(&mut self, byte: u8) {
        self.loop_flag = byte & 0b0010_0000 > 0;
        self.constant_vol_flag = byte & 0b0001_0000 > 0;
        self.volume_value = byte & 0b0000_1111;
    }

    pub fn set_start(&mut self) {
        self.start_flag = true
    }

    pub fn clock(&mut self) {
        if self.start_flag {
            self.start_flag = false;
            self.decay_counter = 15;
            self.divider_counter = self.volume_value;
        } else {
            if self.divider_counter == 0 {
                self.divider_counter = self.volume_value;
                if self.decay_counter == 0 && self.loop_flag {
                    self.decay_counter = 15;
                } else {
                    self.decay_counter = self.decay_counter.saturating_sub(1);
                }
            } else {
                self.divider_counter -= 1
            }
        }
    }

    pub fn volume(&self) -> u8 {
        if self.constant_vol_flag {
            self.volume_value
        } else {
            self.decay_counter
        }
    }
}