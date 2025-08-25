use std::collections::HashMap;
use std::ops::Rem;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::SeqCst;
use nih_plug::prelude::*;
use vizia_plug::ViziaState;
use nih_plug::prelude::SmoothingStyle::Linear;

mod editor;

#[derive(Params)]
pub struct VelocityMapperParams {
    /// The editor state, saved together with the parameter state so the custom scaling can be
    /// restored.
    #[persist = "editor-state"]
    pub editor_state: Arc<ViziaState>,
}

impl Default for VelocityMapperParams {
    fn default() -> Self {
        Self {
            editor_state: editor::default_state(),
        }
    }
}

struct VelocityMapper {
    params: Arc<VelocityMapperParams>,
}

impl Default for VelocityMapper {
    fn default() -> Self {
        let default_params = Arc::new(VelocityMapperParams::default());
        Self {
            params: default_params.clone(),
        }
    }
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
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {

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