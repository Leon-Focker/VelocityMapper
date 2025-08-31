use std::collections::HashMap;
use std::ops::Rem;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::SeqCst;
use nih_plug::prelude::*;
use vizia_plug::ViziaState;
use nih_plug::prelude::SmoothingStyle::Linear;
use crate::editor;

#[derive(Params)]
pub struct VelocityMapperParams {
    #[persist = "editor-state"]
    pub editor_state: Arc<ViziaState>,

    // #[nested(id_prefix = "range1", group = "range1")]
    // pub range1: RangeParams,
    //
    // #[nested(id_prefix = "range2", group = "range3")]
    // pub range2: RangeParams,
    //
    // #[nested(id_prefix = "range3", group = "range3")]
    // pub range3: RangeParams,
    //
    // #[nested(id_prefix = "range4", group = "range4")]
    // pub range4: RangeParams,

    // TODO can we have a flexible amount of ranges??
    // pub how_many: u8,

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

            // range1: Default::default(),
            // range2: Default::default(),
            // range3: Default::default(),
            // range4: Default::default(),

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