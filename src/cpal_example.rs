
use std::f32::consts::PI;
use std::collections::VecDeque;
use std::iter::zip;
use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    SizedSample, I24,
    FromSample, Sample
};


fn main() {
    
    let host = cpal::default_host();
    let device = host.default_output_device().expect("no output device");

    let config = device.default_output_config().expect("no default config");

    println!("{:?}", config);

    let stream = match config.sample_format() {
        cpal::SampleFormat::F32 => build_stream::<f32>(&device, &config.into()),
        _ => panic!("Unsuported format")
    };
    stream.play().expect("Error playing");
    std::thread::sleep(std::time::Duration::from_millis(4000));
}

pub struct Oscillator {
    pub sample_rate: u32,
    pub sample_index: u32,
    pub frequency_hz: f32,
}

impl Oscillator {
    fn next_sample(&mut self) -> f32 {
        let out = self.get_sampel_value_square();
        self.sample_index = (self.sample_index + 1) % self.sample_rate;
        out
    }

    fn get_sampel_value_sin(&self) -> f32 {
        let tau = 2.0 * std::f32::consts::PI;
        let t = (self.sample_index as f32) / (self.sample_rate as f32);
        (tau * self.frequency_hz * t).sin()
    }
    fn get_sampel_value_square(&self) -> f32 {
        let t = (self.sample_index as f32) / (self.sample_rate as f32);
        if (t * self.frequency_hz) % 1.0 < 0.5 { 0.7 }
        else { -0.7}
    }
    fn get_sampel_value_square_alt(&self) -> f32 {
        let tau = 2.0 * std::f32::consts::PI;
        let t = (self.sample_index as f32) / (self.sample_rate as f32);
        let mut out = 0.0;
        let mut i = 1;
        while 2.0 * (i as f32) * self.frequency_hz < (self.sample_rate as f32) {
            let gain = 1.0 / (i as f32);
            let freq = (i as f32) * self.frequency_hz;
            out += gain * (tau * t * freq).sin();
            i += 2
        }        
        out
    }

}

fn build_stream<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig
) -> cpal::Stream
where T: SizedSample + FromSample<f32> 
{
    let mut oscillator = Oscillator{
        sample_rate: config.sample_rate.0 as u32,
        frequency_hz: 440.0,
        sample_index: 0
    };
    let mut filter = Filter::lowpass_blackman(127, 20000.0, config.sample_rate.0 as f32);
    let num_channels = config.channels as usize;

    let stream = device.build_output_stream(
        config, 
        move |output: &mut [T], _: &cpal::OutputCallbackInfo|{
            for frame in output.chunks_mut(num_channels) {
                let mut sample = oscillator.next_sample();
                sample = filter.step(sample);
                let value: T = T::from_sample(sample);
                for sample in frame.iter_mut() {
                    *sample = value;
                }
            }
        }, 
        |err| eprintln!("Error building output sound stream: {}", err), 
        None
    ).expect("Failed to build stream");
    stream
}

pub struct Filter {
    direct_coeff: Vec<f32>,
    recursive_coeff: Vec<f32>,

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
    pub fn lowpass_blackman(length: usize, cutoff_freq: f32, sample_freq: f32) -> Filter{
        let recursive_coeff: Vec<f32> = vec![];
        let mut direct_coeff: Vec<f32> = vec![];

        let fc = 2.*cutoff_freq/sample_freq;
        let m = length as i32;

        for n in -m/2..m/2+1 {
            let t = n as f32;
            let sinc = (PI*fc*t).sin()/(PI*t + 1e-8);
            let x = 2.0*PI*t/(m as f32);
            let win = 0.42 + 0.5*(x).cos() + 0.08*(2.0*x).cos();
            direct_coeff.push(sinc*win);
        }

        Filter::new(direct_coeff, recursive_coeff)
    }
}