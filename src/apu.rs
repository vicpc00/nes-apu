pub mod registers;
pub mod frame_counter;
pub mod pulse;


use crate::apu::frame_counter::FrameCounter;
use crate::apu::registers::{MappedAddress, Registers};
use crate::apu::pulse::Pulse;

//twice the CPU freq of 1.789773 MHz because of the pulse
const MAIN_FREQ: u32 = 21_477_270/6; 

const LENGTH_TABLE: [u8; 0x20] = [
     10, 254,  20,   2,  40,   4,  80,   6, //00-07
    160,   8,  60,  10,  14,  12,  26,  14, //08-0F
     12,  16,  24,  18,  48,  20,  96,  22, //10-17
    192,  24,  72,  26,  16,  28,  32,  30, //18-1F
];

pub enum Channel {
    Pulse1,
    Pulse2,
    Triangle,
    Noise,
    DMC
}

pub struct APU {
    registers: Registers,

    frame_counter: FrameCounter,

    pulse1: Pulse,
    pulse2: Pulse,

    clock_counter: u32,
}

impl APU {
    pub fn new() -> APU {
        APU { 
            registers: Registers::new(),

            frame_counter: FrameCounter::new(),

            pulse1: Pulse::new(true), 
            pulse2: Pulse::new(false),

            clock_counter: 0

        }

    }

    pub fn write_opp(&mut self, address: u16, byte: u8) {
        if address < 0x4000 || address > 0x4017 {
            panic!("Address aout of range")
        }
        let addr = address.to_be_bytes()[1]; 
        let mapped_addr = self.registers.write_register(addr, byte);
        match mapped_addr {
            MappedAddress::Pulse1(reg) => {
                self.pulse1.load_register(reg, byte);
            }
            MappedAddress::Pulse2(reg) => {
                self.pulse2.load_register(reg, byte);
            }
            MappedAddress::FrameRegister => {
                self.frame_counter.load_reguister(byte);
            }
            MappedAddress::StatusRegister => {
                self.pulse1.set_enabled(byte & 0b0000_0001 > 0);
                self.pulse2.set_enabled(byte & 0b0000_0010 > 0);
            }
            _ => {}
        }
    }

    pub fn clock(&mut self) {
        self.clock_counter += 1;
        self.clock_counter %= 6*MAIN_FREQ;

        //println!("cpu clock {}", self.clock_counter);
        // divided by 4 because freq is twice the normal
        if self.clock_counter % 4 == 0 { 
            self.pulse1.clock_timer();
            self.pulse2.clock_timer();
        }
        

        if self.frame_counter.clock() {
            if self.frame_counter.sequencer_signals.envelope_clock {
                self.pulse1.clock_envelop();
                self.pulse2.clock_envelop();
            }
            if self.frame_counter.sequencer_signals.length_clock {
                self.pulse1.clock_length();
                self.pulse2.clock_length();
                self.pulse1.clock_sweep();
                self.pulse2.clock_sweep();
            }

        };
    }

    pub fn output(&self) -> f32 {
        let pulse1 = self.pulse1.output() as f32;
        let pulse2 = self.pulse2.output() as f32;

        let pulse_out: f32 = 95.88 / (8128. / (pulse1 + pulse2) + 100.);
        
        pulse_out
    }

    fn is_channel_enabled(&self, channel: Channel) -> bool {
        let status = self.registers.get_status_register();
        match channel {
            Channel::Pulse1   => status & 0b0000_0001 > 0,
            Channel::Pulse2   => status & 0b0000_0010 > 0,
            Channel::Triangle => status & 0b0000_0100 > 0,
            Channel::Noise    => status & 0b0000_1000 > 0,
            Channel::DMC      => status & 0b0001_0000 > 0,
        }
    }



}

impl APU {
    
}