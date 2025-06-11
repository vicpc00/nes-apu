

enum Mode {
    FourStep, 
    FiveStep
}

const FRAME_DIVIDER: u32 = 89_490/6;

pub struct FrameCounter
{
    mode: Mode,
    interrupt_inhibit: bool,

    divider_counter: u32,
    sequence_counter: u8,

    pub sequencer_signals: FrameCounterClocks
}

pub struct FrameCounterClocks {
    pub length_clock: bool,
    pub envelope_clock: bool,
    pub interrupt: bool
}

impl FrameCounterClocks {
    pub fn any(&self) -> bool {
        self.envelope_clock || self.length_clock || self.interrupt
    }
}

impl FrameCounter
{
    pub fn new() -> Self
    
    {
        FrameCounter {
            mode: Mode::FourStep,
            interrupt_inhibit: false,
            divider_counter: 0,
            sequence_counter: 0,
            sequencer_signals: FrameCounterClocks {
                length_clock: false, 
                envelope_clock: false, 
                interrupt: false
            }
        }
    }
    

    pub fn load_reguister(&mut self, byte: u8) {
        self.mode = if byte & 0b1000_0000 > 0 {
            Mode::FourStep
        } else{
            Mode::FiveStep
        };

        self.interrupt_inhibit = byte & 0b0100_0000 > 0;

        //TODO Wait 3 or 4 cycles to reset counter
        //https://www.nesdev.org/wiki/APU_Frame_Counter
        self.divider_counter = FRAME_DIVIDER-1;
        self.sequence_counter = 0;
        if let Mode::FiveStep = self.mode {
            self.clock_sequencer();
        }

    }

    pub fn clock(&mut self) -> bool{
        if self.divider_counter == FRAME_DIVIDER {
            self.divider_counter = 0;
            self.clock_sequencer();
            true
        } else {
            self.divider_counter += 1;
            false
        }
    }

    pub fn clock_sequencer(&mut self){
        self.sequence_counter += 1;

        match self.mode {
            Mode::FourStep => {
                self.sequence_counter %= 4;
                self.sequencer_signals.envelope_clock = true;
                self.sequencer_signals.length_clock = self.sequence_counter == 1 || self.sequence_counter == 3;
                self.sequencer_signals.interrupt = !self.interrupt_inhibit && self.sequence_counter == 3;
            }
            Mode::FiveStep => {
                self.sequence_counter %= 5;
                self.sequencer_signals.envelope_clock = self.sequence_counter < 4;
                self.sequencer_signals.length_clock = self.sequence_counter == 0 || self.sequence_counter == 2;
                self.sequencer_signals.interrupt = false;
            }
        };

    }
}