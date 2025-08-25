use nih_plug::prelude::{Editor};
use vizia_plug::vizia::prelude::*;
use vizia_plug::{create_vizia_editor, ViziaState, ViziaTheming};
use std::sync::Arc;
use std::sync::atomic::Ordering::SeqCst;
use vizia_plug::vizia::prelude::Anchor::Center;
use vizia_plug::vizia::style::FontWeightKeyword::Bold;
use vizia_plug::widgets::{ParamButton, ParamSlider};
use crate::params::VelocityMapperParams;

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
    ViziaState::new(|| (600, 500))
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

            range_selector(cx, 1);
            range_selector(cx, 2);
            range_selector(cx, 3);
            range_selector(cx, 4);

        })
            .alignment(Alignment::TopCenter);
    })
}

// TODO this "generic" is no really working. Make it a real Gui Element?
fn range_selector(cx: &mut Context, nr: u8) {
    VStack::new(cx, |cx| {
        Label::new(cx, format!("Range {}", nr))
            .font_weight(Bold)
            .font_size(15.0);

        HStack::new(cx, |cx| {
            match nr {
                1 => ParamButton::new(cx, Data::params, |params| &params.range1.bypass),
                2 => ParamButton::new(cx, Data::params, |params| &params.range2.bypass),
                3 => ParamButton::new(cx, Data::params, |params| &params.range3.bypass),
                _ => ParamButton::new(cx, Data::params, |params| &params.range4.bypass),
            };
            VStack::new(cx, |cx| {
                match nr {
                    1 => ParamSlider::new(cx, Data::params, |params| &params.range1.range_min),
                    2 => ParamSlider::new(cx, Data::params, |params| &params.range2.range_min),
                    3 => ParamSlider::new(cx, Data::params, |params| &params.range3.range_min),
                    _ => ParamSlider::new(cx, Data::params, |params| &params.range4.range_min),
                };
                match nr {
                    1 => ParamSlider::new(cx, Data::params, |params| &params.range1.range_max),
                    2 => ParamSlider::new(cx, Data::params, |params| &params.range2.range_max),
                    3 => ParamSlider::new(cx, Data::params, |params| &params.range3.range_max),
                    _ => ParamSlider::new(cx, Data::params, |params| &params.range4.range_max),
                };
            });
            match nr {
                1 => ParamSlider::new(cx, Data::params, |params| &params.range1.pitch),
                2 => ParamSlider::new(cx, Data::params, |params| &params.range2.pitch),
                3 => ParamSlider::new(cx, Data::params, |params| &params.range3.pitch),
                _ => ParamSlider::new(cx, Data::params, |params| &params.range4.pitch),
            };
        });
    })
        .alignment(Alignment::Center)
        .height(Pixels(100.0));
}
