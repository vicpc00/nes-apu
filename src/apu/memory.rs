
//TODO: implement banks
pub struct Memory {
    pub memory: [u8; 0x10000],
}

impl Memory {
    pub fn new() -> Memory {
        Memory { memory: [0; 0x10000] }
    }

    pub fn get_byte(&self, addr: u16) -> u8 {
        self.memory[addr as usize]
    }

    pub fn load_byte(&mut self, addr: u16, byte: u8) {
        self.memory[addr as usize] = byte;
    }

    pub fn load_bytes(&mut self, start: u16, bytes: &Vec<u8>) {
        let start = start as usize;
        for (i, byte) in bytes.iter().enumerate() {
            self.memory[start+i] = *byte;
        }
        
    }
    
}