use nih_plug::prelude::{Editor};
use vizia_plug::vizia::prelude::*;
use vizia_plug::{create_vizia_editor, ViziaState, ViziaTheming};
use std::sync::Arc;
use std::sync::atomic::Ordering::SeqCst;
use vizia_plug::vizia::style::FontWeightKeyword::Bold;
use crate::VelocityMapperParams;

#[derive(Lens, Clone)]
pub(crate) struct Data {
    pub(crate) params: Arc<VelocityMapperParams>,
}

impl Model for Data {
    fn event(&mut self, _: &mut EventContext, event: &mut Event) {
    }
}

// Makes sense to also define this here, makes it a bit easier to keep track of
pub(crate) fn default_state() -> Arc<ViziaState> {
    ViziaState::new(|| (300, 100))
}

pub(crate) fn create(
    params: Arc<VelocityMapperParams>,
    editor_state: Arc<ViziaState>,
) -> Option<Box<dyn Editor>> {
    create_vizia_editor(editor_state, ViziaTheming::Custom, move |cx, _| {

        Data {
            params: params.clone(),
        }
            .build(cx);

        VStack::new(cx, |cx| {
            Label::new(cx, "Velocity Mapper")
                .font_weight(Bold)
                .font_size(25.0);
        })
            .alignment(Alignment::TopCenter);
    })
}