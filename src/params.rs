use std::sync::Arc;
use nih_plug::prelude::*;
use vizia_plug::ViziaState;
use crate::editor;

#[derive(Params)]
pub struct VelocityMapperParams {
    #[persist = "editor-state"]
    pub editor_state: Arc<ViziaState>,

    // TODO can we have a flexible amount of ranges??
    // #[id = "how_many"]
    // pub how_many: IntParam,

    #[nested(array, group = "ranges")]
    pub ranges: Vec<RangeParams>,
}

#[derive(Params)]
pub struct RangeParams {
    #[id = "bypass"]
    pub bypass: BoolParam,

    #[id = "range_min"]
    pub range_min: IntParam,

    #[id = "range_max"]
    pub range_max: IntParam,

    #[id = "pitch"]
    pub pitch: IntParam,
}

impl Default for VelocityMapperParams {
    fn default() -> Self {
        Self {
            editor_state: editor::default_state(),

            // how_many: IntParam::new(
            //     "how many",
            //     4,
            //     IntRange::Linear { min: 1, max: 10 }
            // ).non_automatable(),

            ranges: vec![RangeParams::default(), RangeParams::default(), RangeParams::default(), RangeParams::default()],
        }
    }
}

impl Default for RangeParams {
    fn default() -> Self {
        Self {
            bypass: BoolParam::new(
                "Bypass",
                true
            ),
            range_min: IntParam::new(
                "Velocity Minimum",
                0,
                IntRange::Linear { min: 0, max: 127 }
                ),
            range_max: IntParam::new(
                "Velocity Maximum",
                127,
                IntRange::Linear { min: 0, max: 127 }
            ),
            pitch: IntParam::new(
                "Output Pitch",
                60,
                IntRange::Linear { min: 0, max: 127 }
            ),
        }
    }
}