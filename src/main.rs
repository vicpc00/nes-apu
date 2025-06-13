
use hound;

use nes_apu::apu::APU;

fn main() {

    let mut apu = APU::new();

    apu.write_opp(0x4000, 0b1000_1111);
    apu.write_opp(0x4001, 0b0000_1000);
    apu.write_opp(0x4002, 0xFD); //440Hz = 0x0FD
    apu.write_opp(0x4003, 0x00);

    apu.write_opp(0x4015, 0b0000_0001);


    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 44100,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };

    let cps = (21_477_270/6)/44100;

    let mut writer = hound::WavWriter::create("out.wav", spec).unwrap();

    for _ in 0 .. 5*44100 {
        for _ in 0..cps {
            apu.clock();
        }
        let sample = apu.output();
        writer.write_sample(sample).unwrap();
    }
}