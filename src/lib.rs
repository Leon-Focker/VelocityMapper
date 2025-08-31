use std::sync::Arc;
use nih_plug::prelude::*;
use vizia_plug::vizia::vg::luma_color_filter::new;
use crate::params::{RangeParams, VelocityMapperParams};

mod editor;
mod params;
mod gui;

const MAX_CHANNELS: usize = 16;
const MAX_PITCHES: usize = 128;
const MAX_VOICES: usize = 4; // max overlapping notes per pitch

struct VelocityMapper {
    params: Arc<VelocityMapperParams>,
    // This stores NoteOns to match with NoteOffs by channel, by pitch, with Option<new_pitch>
    note_stack: Vec<Vec<Vec<Option<u8>>>>,
}

impl Default for VelocityMapper {
    fn default() -> Self {
        let default_params = Arc::new(VelocityMapperParams::default());
        Self {
            params: default_params.clone(),
            note_stack: vec![vec![vec![None; MAX_VOICES]; MAX_PITCHES]; MAX_CHANNELS],
        }
    }
}

impl VelocityMapper {
    fn get_remapped_pitch(&mut self, velocity: f32) -> Option<u8> {
        if let Some(pitch) = matches_range(velocity, &self.params.range1) {
            Some(pitch)
        } else if let Some(pitch) = matches_range(velocity, &self.params.range2) {
            Some(pitch)
        } else if let Some(pitch) = matches_range(velocity, &self.params.range3) {
            Some(pitch)
        } else if let Some(pitch) = matches_range(velocity, &self.params.range4) {
            Some(pitch)
        } else { return None }
    }

    fn next_free_index(&self, channel: u8, old_pitch: u8) -> Option<usize> {
        self.note_stack[channel as usize][old_pitch as usize]
            .iter()
            .position(|note| note.is_none())
    }

    fn push_note_on(&mut self, channel: u8, old_pitch: u8, new_pitch: u8) {
        let idx = self.next_free_index(channel, old_pitch).unwrap_or(MAX_VOICES - 1);
        self.note_stack[channel as usize][old_pitch as usize][idx] = Some(new_pitch);
    }

    fn pop_note_on(&mut self, channel: u8, old_pitch: u8) -> Option<u8> {
        let idx = self.next_free_index(channel, old_pitch).unwrap_or(MAX_VOICES);
        if idx == 0 {
            None
        } else {
            let new_pitch = self.note_stack[channel as usize][old_pitch as usize][idx-1];
            self.note_stack[channel as usize][old_pitch as usize][idx-1] = None;
            new_pitch
        }
    }
}

fn matches_range(velocity: f32, range_params: &RangeParams) -> Option<u8> {
    let lo = range_params.range_min.unmodulated_normalized_value();
    let hi = range_params.range_max.unmodulated_normalized_value();

    if !range_params.bypass.value()
        && (velocity >= lo.min(hi) && velocity <= lo.max(hi)) {
        Some(range_params.pitch.value() as u8)
    } else { return None }
}

impl Plugin for VelocityMapper {
    const NAME: &'static str = "VelocityMapper";
    const VENDOR: &'static str = "Leon Focker";
    const URL: &'static str = "https://youtu.be/dQw4w9WgXcQ";
    const EMAIL: &'static str = "contact@leonfocker.de";

    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[
    ];

    const MIDI_INPUT: MidiConfig = MidiConfig::MidiCCs;
    const MIDI_OUTPUT: MidiConfig = MidiConfig::MidiCCs;
    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        editor::create(
            self.params.clone(),
            self.params.editor_state.clone(),
        )
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        _buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        true
    }

    fn process(
        &mut self,
        _buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {

        while let Some(event) = context.next_event() {
            match event {
                NoteEvent::NoteOn {
                    timing,
                    voice_id,
                    channel,
                    note,
                    velocity,
                } => {
                    let new_pitch = self.get_remapped_pitch(velocity);
                    // remember this NoteOn and which pitch it was mapped to
                    self.push_note_on(channel, note, new_pitch.unwrap_or(note));

                    // Send and maybe remap the NoteOn
                    if let Some(new_pitch) = new_pitch {
                        context.send_event(NoteEvent::NoteOn {
                            timing,
                            voice_id,
                            channel,
                            note: new_pitch,
                            velocity,
                        })
                    } else { context.send_event(event) }
                },
                NoteEvent::NoteOff {
                    timing,
                    voice_id,
                    channel,
                    note,
                    velocity,
                } => {
                    // if we remapped the last NoteOn on this channel with this pitch, get new pitch
                    let new_pitch = self.pop_note_on(channel, note).unwrap_or(note);

                    context.send_event(NoteEvent::NoteOff {
                        timing,
                        voice_id,
                        channel,
                        note: new_pitch,
                        velocity,
                    })
                },
                _ => (),
            }
        }

        ProcessStatus::Normal
    }
}

impl ClapPlugin for VelocityMapper {
    const CLAP_ID: &'static str = "leonfocker.velocitymapper";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::AudioEffect,
        ClapFeature::Stereo,
        ClapFeature::Mono,
        ClapFeature::Utility,
    ];
}

impl Vst3Plugin for VelocityMapper {
    const VST3_CLASS_ID: [u8; 16] = *b"VelocityMapperrr";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Tools];
}

nih_export_clap!(VelocityMapper);
nih_export_vst3!(VelocityMapper);