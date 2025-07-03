use std::{
    f32::consts::PI,
    collections::VecDeque, 
    iter::zip,
};

pub struct Filter {
    pub direct_coeff: Vec<f32>,
    pub recursive_coeff: Vec<f32>,

    input_buffer: VecDeque<f32>,
    output_buffer: VecDeque<f32>,
}

impl Filter {
    pub fn new(direct_coeff: Vec<f32>, recursive_coeff: Vec<f32>) -> Filter {
        let mut ret = Filter {
            direct_coeff: direct_coeff,
            recursive_coeff: recursive_coeff,

            input_buffer: VecDeque::new(),
            output_buffer: VecDeque::new(),
        };

        for _ in 0..ret.direct_coeff.len() {
            ret.input_buffer.push_back(0.0);
        }
        for _ in 0..ret.recursive_coeff.len()+1 {
            ret.output_buffer.push_back(0.0);
        }
        ret
    }

    pub fn step(&mut self, sample: f32) -> f32{
        self.output_buffer.pop_back();
        self.input_buffer.push_front(sample);
        self.input_buffer.pop_back();

        let mut out:f32 = 0.0;

        for (b, x) in zip(self.direct_coeff.iter(), self.input_buffer.iter()) {
            out += b*x;
        }
        for (a, y) in zip(self.recursive_coeff.iter(), self.output_buffer.iter()) {
            out += a*y;
        }

        self.output_buffer.push_front(out);
        
        out
    }

    pub fn output(&self) -> f32{
        self.output_buffer[0]
    }

}

impl Filter {
    pub fn lowpass_blackman(length: u32, cutoff_freq: f32, sample_freq: f32) -> Filter{
        let recursive_coeff: Vec<f32> = vec![];
        let mut direct_coeff: Vec<f32> = vec![];

        let fc = 2.*cutoff_freq/sample_freq;
        let m = length as i32;

        for n in -m/2..m/2+1 {
            let t = n as f32;
            let sinc = if n != 0 {(PI*fc*t).sin()/(PI*t + 1e-8)} else {fc};
            let x = 2.0*PI*t/(m as f32);
            let win = 0.42 + 0.5*(x).cos() + 0.08*(2.0*x).cos();
            direct_coeff.push(sinc*win);
        }
        Filter::new(direct_coeff, recursive_coeff)
    }
}