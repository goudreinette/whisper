#[macro_use]
extern crate vst;
extern crate rand;
extern crate noise;


use vst::prelude::HostCallback;
use vst::plugin::{Info, Plugin, Category, PluginParameters};
use vst::buffer::AudioBuffer;
use rand::random;
use vst::api::Events;
use vst::event::Event;
use noise::{NoiseFn, Perlin, Worley, Billow, Cylinders, OpenSimplex, RidgedMulti, Value, HybridMulti, BasicMulti};
use std::sync::{Arc};
use std::ops::Deref;

mod parameters;
mod util;

use parameters::WhisperParameters;
use util::midi_pitch_to_freq;


#[derive(Debug, Copy, Clone)]
struct Note {
    alpha: f64,
    note: u8,
    is_released: bool,
}


struct Whisper {
    notes: Vec<Note>,

    sample_rate: f32,
    time: f64,

    // Noise functions
    fn_perlin: Perlin,
    fn_value: Value,
    fn_worley: Worley,
    fn_ridged_multi: RidgedMulti,
    fn_open_simplex: OpenSimplex,
    fn_billow: Billow,
    fn_cylinders: Cylinders,
    fn_hybrid_multi: HybridMulti,
    fn_basic_multi: BasicMulti,

    params: Arc<WhisperParameters>
}



impl Default for Whisper {
    fn default() -> Whisper {
        Whisper {
            notes: vec![],
            sample_rate: 44100.0,
            time: 0.0,

            // Amounts
            params: Arc::new(WhisperParameters::default()),

            // Noise functions
            fn_perlin: Perlin::new(),
            fn_value: Value::new(),
            fn_worley: Worley::new(),
            fn_ridged_multi: RidgedMulti::new(),
            fn_open_simplex: OpenSimplex::new(),
            fn_billow: Billow::new(),
            fn_cylinders: Cylinders::new(),
            fn_hybrid_multi: HybridMulti::new(),
            fn_basic_multi: BasicMulti::new(),
        }
    }
}


impl Plugin for Whisper {
    fn new(_host: HostCallback) -> Whisper {
        return Whisper::default()
    }
    
    fn get_info(&self) -> Info {
        Info {
            name: "Whisper".to_string(),
            unique_id: 1337,

            inputs: 0,
            outputs: 2,
            parameters: 12,

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
                            self.notes.push(Note { note: ev.data[1], alpha: 0.0, is_released: false });
                        }
                        128 => {
                            for note in self.notes.iter_mut() {
                                if note.note == ev.data[1] {
                                    note.is_released = true;
                                }
                            }
                        }
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
        let attack_per_sample = per_sample * (1.0 / self.params.attack_duration.get() as f64);
        let release_per_sample = per_sample * (1.0 / self.params.release_duration.get() as f64);


        let mut output_sample;
        for sample_idx in 0..samples {
            // Update the alpha of each note...
            for note in self.notes.iter_mut() {
                if !note.is_released && note.alpha < 1.0 {
                    note.alpha += attack_per_sample;
                }

                if note.is_released {
                    note.alpha -= release_per_sample;
                }
            }

            // ...and remove finished notes.
            self.notes.retain(|n| n.alpha > 0.0);


            // Sum up all the different notes and noise types
            if !self.notes.is_empty() {
                let mut signal = 0.0;
                let params = self.params.deref();


                for note in &self.notes {
                    let point = [0.0, self.time * midi_pitch_to_freq(note.note)];

                    if params.a_white_noise.get() > 0.0 && note.alpha > 0.0001 {
                        signal += ((random::<f64>() - 0.5) * 2.0) * params.a_white_noise.get() as f64 * note.alpha;
                    }

                    if params.a_perlin.get() > 0.0 && note.alpha > 0.0001 {
                        signal += self.fn_perlin.get(point) * params.a_perlin.get() as f64 * note.alpha;
                    }

                    if params.a_value.get() > 0.0 && note.alpha > 0.0001 {
                        signal += self.fn_value.get(point) * params.a_value.get() as f64 * note.alpha;
                    }

                    if params.a_worley.get() > 0.0 && note.alpha > 0.0001 {
                        signal += self.fn_worley.get(point) * params.a_worley.get() as f64 * note.alpha;
                    }

                    if params.a_ridged_multi.get() > 0.0 && note.alpha > 0.0001 {
                        signal += self.fn_ridged_multi.get(point) * params.a_ridged_multi.get() as f64 * note.alpha;
                    }

                    if params.a_open_simplex.get() > 0.0 && note.alpha > 0.0001 {
                        signal += self.fn_open_simplex.get(point) * params.a_open_simplex.get() as f64 * note.alpha;
                    }

                    if params.a_billow.get() > 0.0 && note.alpha > 0.0001 {
                        signal += self.fn_billow.get(point) * params.a_billow.get() as f64 * note.alpha;
                    }

                    if params.a_cylinders.get() > 0.0 && note.alpha > 0.0001 {
                        signal += self.fn_cylinders.get(point) * params.a_cylinders.get() as f64 * note.alpha;
                    }

                    if params.a_hybrid_multi.get() > 0.0 && note.alpha > 0.0001 {
                        signal += self.fn_hybrid_multi.get(point) * params.a_hybrid_multi.get() as f64 * note.alpha;
                    }

                    if params.a_basic_multi.get() > 0.0 && note.alpha > 0.0001 {
                        signal += self.fn_basic_multi.get(point) * params.a_basic_multi.get() as f64 * note.alpha;
                    }
                }


                output_sample = signal as f32;
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

    fn get_parameter_object(&mut self) -> Arc<dyn PluginParameters> {
        Arc::clone(&self.params) as Arc<dyn PluginParameters>
    }
}


plugin_main!(Whisper);
