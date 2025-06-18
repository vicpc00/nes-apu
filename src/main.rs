
use hound;

use nes_apu::{apu::APU, dac::filter::Filter};

fn main() {

    let mut apu = APU::new();

    apu.write_opp(0x4015, 0b0000_0100);
    apu.write_opp(0x4017, 0b0000_0000);

    //apu.write_opp(0x4000, 0b1000_0011);
    apu.write_opp(0x4000, 0b1001_1111);
    //apu.write_opp(0x4001, 0b1000_1001);
    apu.write_opp(0x4001, 0b0000_1000);
    apu.write_opp(0x4002, 0xFD); //440Hz = 0x0FD
    apu.write_opp(0x4003, (0x16 << 3) + 0x00);

    apu.write_opp(0x4008, 0b0111_1111);
    apu.write_opp(0x400A, 0x7E);
    apu.write_opp(0x400B, (0x14 << 3) + 0x00);

    println!("{:?}", apu.pulse1);
    println!("{:?}", apu.pulse2);
    println!("{:?}", apu.triangle);

    let final_fs = 44100;
    let apu_downsample = 2;
    let apu_fs = apu_downsample*final_fs;

    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: final_fs,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };

    let cps = (21_477_270/6)/apu_fs;

    let mut aa_filter = Filter::lowpass_blackman(256, 20000., apu_fs as f32);

    let mut writer = hound::WavWriter::create("out.wav", spec).unwrap();

    for sample_count in 0 .. 2*44100 {
        for _ in 0..cps {
            apu.clock();
        }
        let sample = apu.output();
        let filtered_sample = aa_filter.step(sample);
        if sample_count % apu_downsample == 0 {
            writer.write_sample(filtered_sample).unwrap();
        }
    }
}