
use crate::apu::Channel;
use crate::apu::pulse::PulseRegister;
use crate::apu::triangle::TriangleRegister;
use crate::apu::noise::NoiseRegister;

pub enum MappedAddress {
    Pulse1(PulseRegister),
    Pulse2(PulseRegister),
    Triangle(TriangleRegister),
    Noise(NoiseRegister),
    DMC,
    StatusRegister,
    NotUsed,
    FrameRegister
} 

pub struct Registers {
    pub memory: [u8; 0x18]
}

impl Registers {
    pub fn new() -> Registers {
        Registers{
            memory: [0; 0x18]
        }
    }

    pub fn write_register(&mut self, address: u8, byte: u8) -> MappedAddress{
        self.memory[address as usize] = byte;
        match address {
            0x00 => MappedAddress::Pulse1(PulseRegister::R0),
            0x01 => MappedAddress::Pulse1(PulseRegister::R1),
            0x02 => MappedAddress::Pulse1(PulseRegister::R2),
            0x03 => MappedAddress::Pulse1(PulseRegister::R3),
            0x04 => MappedAddress::Pulse2(PulseRegister::R0),
            0x05 => MappedAddress::Pulse2(PulseRegister::R1),
            0x06 => MappedAddress::Pulse2(PulseRegister::R2),
            0x07 => MappedAddress::Pulse2(PulseRegister::R3),
            0x08 => MappedAddress::Triangle(TriangleRegister::R0),
            0x09 => MappedAddress::Triangle(TriangleRegister::R1),
            0x0A => MappedAddress::Triangle(TriangleRegister::R2),
            0x0B => MappedAddress::Triangle(TriangleRegister::R3),
            0x0C => MappedAddress::Noise(NoiseRegister::R0),
            0x0D => MappedAddress::Noise(NoiseRegister::R1),
            0x0E => MappedAddress::Noise(NoiseRegister::R2),
            0x0F => MappedAddress::Noise(NoiseRegister::R3),
            0x10 => MappedAddress::NotUsed,
            0x11 => MappedAddress::NotUsed,
            0x12 => MappedAddress::NotUsed,
            0x13 => MappedAddress::NotUsed,
            0x14 => MappedAddress::NotUsed,
            0x15 => MappedAddress::StatusRegister,
            0x16 => MappedAddress::NotUsed,
            0x17 => MappedAddress::FrameRegister,
            _    => panic!("Address out of range")
        }
    }

    fn get_register_address(&self, channel: Channel) -> usize {
        match channel {
            Channel::Pulse1   => 0x00,
            Channel::Pulse2   => 0x04,
            Channel::Triangle => 0x08,
            Channel::Noise    => 0x0C,
            Channel::DMC      => 0x10,
        }
    }

    pub fn get_channel_registers(&self, channel: Channel) -> [u8; 4] {
        let mut bytes: [u8; 4] = [0;4];
        let addr = self.get_register_address(channel);
        for i in 0..4 {
            bytes[i] = self.memory[addr+i];
        }
        bytes
    }
    pub fn set_channel_registers(&mut self, channel: Channel, bytes: [u8; 4] ) {
        let addr = self.get_register_address(channel);
        for i in 0..4 {
            self.memory[addr+i] = bytes[i]
        }
    }

    pub fn get_status_register(&self) -> u8 {
        self.memory[0x15]
    }
    pub fn set_status_register(&mut self, value: u8) {
        self.memory[0x15] = value;
    }
    pub fn get_frame_register(&self) -> u8 {
        self.memory[0x17]
    }
    pub fn set_frame_register(&mut self, value: u8) {
        self.memory[0x17] = value;
    }

}
