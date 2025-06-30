pub mod registers;
pub mod frame_counter;
pub mod envelope;
pub mod pulse;
pub mod triangle;
pub mod noise;
pub mod dmc;
pub mod memory;


use crate::apu::frame_counter::FrameCounter;
use crate::apu::registers::{MappedAddress, Registers};
use crate::apu::memory::Memory;
use crate::apu::pulse::Pulse;
use crate::apu::triangle::Triangle;
use crate::apu::noise::Noise;
use crate::apu::dmc::DMC;

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
    sample_memory: Memory,

    frame_counter: FrameCounter,

    pub pulse1: Pulse,
    pub pulse2: Pulse,
    pub triangle: Triangle,
    pub noise: Noise,
    pub dmc: DMC,

    clock_counter: u32,
}

impl APU {
    pub fn new() -> APU {
        APU { 
            registers: Registers::new(),
            sample_memory: Memory::new(),

            frame_counter: FrameCounter::new(),

            pulse1: Pulse::new(true), 
            pulse2: Pulse::new(false),
            triangle: Triangle::new(),
            noise: Noise::new(),
            dmc: DMC::new(),


            clock_counter: 0

        }

    }

    pub fn write_opp(&mut self, address: u16, byte: u8) {
        match address {
            0x4000..=0x4017 => {
                let addr = address.to_be_bytes()[1]; 
                let mapped_addr = self.registers.write_register(addr, byte);
                match mapped_addr {
                    MappedAddress::Pulse1(reg) => {
                        self.pulse1.load_register(reg, byte);
                    }
                    MappedAddress::Pulse2(reg) => {
                        self.pulse2.load_register(reg, byte);
                    }
                    MappedAddress::Triangle(reg) => {
                        self.triangle.load_register(reg, byte);
                    }
                    MappedAddress::Noise(reg) => {
                        self.noise.load_register(reg, byte);
                    }
                    MappedAddress::DMC(reg) => {
                        self.dmc.load_register(reg, byte);
                    }
                    MappedAddress::FrameRegister => {
                        self.frame_counter.load_reguister(byte);
                    }
                    MappedAddress::StatusRegister => {
                        self.pulse1.set_enabled(byte & 0b0000_0001 > 0);
                        self.pulse2.set_enabled(byte & 0b0000_0010 > 0);
                        self.triangle.set_enabled(byte & 0b0000_0100 > 0);
                        self.noise.set_enabled(byte & 0b0000_1000 > 0);
                        self.dmc.set_enabled(byte & 0b0001_0000 > 0);
                        self.dmc_fetch();
                    }
                    _ => {}
                }
            }
            0xC000..=0xCFFF => {
                self.sample_memory.load_byte(address, byte);
            }
            _ => {
                panic!("Address out of range")
            }
        }
        
    }

    pub fn load_sample(&mut self, address: u16, sample: &Vec<u8>) {
        self.sample_memory.load_bytes(address, sample);
    }

    pub fn clock(&mut self) {
        self.clock_counter += 1;
        self.clock_counter %= 6*MAIN_FREQ;

        // divided by 4 because freq is twice the normal
        if self.clock_counter % 4 == 0 { 
            self.pulse1.clock_timer();
            self.pulse2.clock_timer();
        }
        if self.clock_counter % 2 == 0 { 
            self.triangle.clock_timer();
            self.noise.clock_timer();
            self.dmc.clock_timer();
        }

        if self.frame_counter.clock() {
            if self.frame_counter.sequencer_signals.envelope_clock {
                self.pulse1.clock_envelop();
                self.pulse2.clock_envelop();
                self.triangle.clock_linear();
                self.noise.clock_envelop();
            }
            if self.frame_counter.sequencer_signals.length_clock {
                self.pulse1.clock_length();
                self.pulse2.clock_length();
                self.triangle.clock_length();
                self.noise.clock_length();
                self.pulse1.clock_sweep();
                self.pulse2.clock_sweep();
            }

        };
        self.dmc_fetch();
    }

    pub fn output(&self) -> f32 {
        let pulse1 = self.pulse1.output() as f32;
        let pulse2 = self.pulse2.output() as f32;

        let pulse_out: f32 = 95.88 / (8128. / (pulse1 + pulse2) + 100.);

        let triangle = self.triangle.output() as f32;
        let noise = self.noise.output() as f32;
        let dmc = self.dmc.output() as f32;

        let tnd_out: f32 = 159.79 / (100. + 1. / (triangle / 8227. + noise / 12241. + dmc / 22638.));
        
        pulse_out + tnd_out
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

    fn dmc_fetch(&mut self) {
        if self.dmc.request_fetch {
            self.dmc.fetch_sample(&self.sample_memory);
        }
    }



}

impl APU {
    
}