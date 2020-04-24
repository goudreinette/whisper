#[macro_use]
extern crate vst;
extern crate rand;
extern crate lerp;
extern crate noise;

use lerp::Lerp;
use vst::plugin::{Info, Plugin, Category, HostCallback};
use vst::buffer::AudioBuffer;
use rand::random;
use vst::api::Events;
use vst::event::Event;
use std::f64::consts::PI;
use noise::{NoiseFn, Perlin, Worley, Point2, Billow, Cylinders, OpenSimplex, RidgedMulti, SuperSimplex, Value, HybridMulti, BasicMulti};
use std::sync::Arc;


const TAU: f64 = PI * 2.0;


struct Whisper {
    notes: u8,
    note: u8,
    sample_rate: f32,
    time: f64,
    note_duration: f64,
    decay_time: f64,
    alpha: f64,

    // Amounts
    a_perlin: f32,
    a_value: f32,
    a_worley: f32,
    a_ridged_multi: f32,
    a_open_simplex: f32,
    a_billow: f32,
    a_cylinders: f32,
    a_hybrid_multi: f32,
    a_basic_multi: f32,

    // Noise functions
    fn_perlin: Perlin,
    fn_value: Value,
    fn_worley: Worley,
    fn_ridged_multi: RidgedMulti,
    fn_open_simplex: OpenSimplex,
    fn_billow: Billow,
    fn_cylinders: Cylinders,
    fn_hybrid_multi: HybridMulti,
    fn_basic_multi: BasicMulti

}


impl Default for Whisper {
    fn default() -> Whisper {
        Whisper {
            notes: 0,
            note: 0,
            sample_rate: 44100.0,
            time: 0.0,
            note_duration: 0.0,
            decay_time: 0.0,
            alpha: 0.0,

            // Amounts
            a_perlin: 100.0,
            a_value: 0.0,
            a_worley: 0.0,
            a_ridged_multi: 0.0,
            a_open_simplex: 0.0,
            a_billow: 0.0,
            a_cylinders: 0.0,
            a_hybrid_multi: 0.0,
            a_basic_multi: 0.0,

            // Noise functions
            fn_perlin: Perlin::new(),
            fn_value: Value::new(),
            fn_worley: Worley::new(),
            fn_ridged_multi: RidgedMulti::new(),
            fn_open_simplex: OpenSimplex::new(),
            fn_billow: Billow::new(),
            fn_cylinders: Cylinders::new(),
            fn_hybrid_multi: HybridMulti::new(),
            fn_basic_multi: BasicMulti::new()
        }
    }
}


impl Plugin for Whisper {
    fn get_parameter(&self, index: i32) -> f32 {
        match index {
            0 => self.size,
            1 => self.dry_wet,
            _ => 0.0,
        }
    }

    fn get_parameter_text(&self, index: i32) -> String {
        match index {
            0 => format!("{}", (self.size * 1000.0) as isize),
            1 => format!("{:.1}%", self.dry_wet * 100.0),
            2 => format!("{:.1}%", self.dry_wet * 100.0),
            3 => format!("{:.1}%", self.dry_wet * 100.0),
            4 => format!("{:.1}%", self.dry_wet * 100.0),
            5 => format!("{:.1}%", self.dry_wet * 100.0),
            6 => format!("{:.1}%", self.dry_wet * 100.0),
            7 => format!("{:.1}%", self.dry_wet * 100.0),
            8 => format!("{:.1}%", self.dry_wet * 100.0),
            _ => "".to_string(),
        }
    }

    fn get_parameter_name(&self, index: i32) -> String {
        match index {
            0 => "Perlin",
            1 => "Value",
            2 => "Worley",
            3 => "RidgedMulti",
            4 => "OpenSimplex",
            5 => "Billow",
            6 => "Cylinders",
            7 => "HybridMulti",
            8 => "BasicMulti",
            _ => "",
        }.to_string()
    }

    fn set_parameter(&mut self, index: i32, val: f32) {
        match index {
            0 => self.size = val,
            1 => self.dry_wet = val,
            _ => (),
        }
    }

    fn get_info(&self) -> Info {
        Info {
            name: "Whisper".to_string(),
            unique_id: 1337,

            inputs: 0,
            outputs: 2,
            parameters: 2,

            category: Category::Synth,

            ..Default::default()
        }
    }



    fn process_events(&mut self, events: &Events) {
        for event in events.events() {
            match event {
                Event::Midi(ev) => {
                    match ev.data[0] {
                        144 => {
                            self.notes += 1;
                            self.note = ev.data[1];
                        },
                        128 => {
                            self.notes -= 1;
                        },
                        _ => ()
                    }

                }

                _ => ()
            }
        }
    }

    fn set_sample_rate(&mut self, rate: f32) {
        self.sample_rate = rate;
    }


    fn process(&mut self, buffer: &mut AudioBuffer<f32>) {
        let samples = buffer.samples();
        let (_, mut outputs) = buffer.split();
        let output_count = outputs.len();
        let per_sample = (1.0 / self.sample_rate) as f64;

        let mut output_sample;
        for sample_idx in 0..samples {
            let time = self.time;


            if self.notes > 0 {
                self.alpha = self.alpha.lerp(1.0, 0.000125)
            } else {
                self.alpha = self.alpha.lerp(0.0, 0.000125)
            }


            if self.notes != 0 || self.alpha > 0.0001 {
                let sin =  (time * midi_pitch_to_freq(self.note) * TAU).sin();
                let noise =  random::<f64>() - 0.5;

                let signal = self.perlin.get([0.0, time * midi_pitch_to_freq(self.note)]);

                output_sample = (signal * self.alpha) as f32;

                self.time += per_sample;
            } else {
                output_sample = 0.0;
            }

            for buf_idx in 0..output_count {
                let buff = outputs.get_mut(buf_idx);
                buff[sample_idx] = output_sample;
            }
        }
    }
}

plugin_main!(Whisper);



fn midi_pitch_to_freq(pitch: u8) -> f64 {
    const A4_PITCH: i8 = 69;
    const A4_FREQ: f64 = 440.0;

    // Midi notes can be 0-127
    ((f64::from(pitch as i8 - A4_PITCH)) / 12.).exp2() * A4_FREQ
}
