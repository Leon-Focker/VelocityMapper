use nih_plug::prelude::{Editor, Param};
use vizia_plug::vizia::prelude::*;
use vizia_plug::{create_vizia_editor, ViziaState, ViziaTheming};
use std::sync::Arc;
use vizia_plug::vizia::style::FontWeightKeyword::Bold;
use vizia_plug::widgets::{ParamButton, ParamSlider};
use crate::params::{RangeParams, VelocityMapperParams};
use crate::gui::dropdown_param::DropDownParam;

const NEW_STYLE: &str = r#"
    .red_button:checked {
        background-color: #ac3535;
    }
"#;

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
    ViziaState::new(|| (300, 570))
}

pub(crate) fn create(
    params: Arc<VelocityMapperParams>,
    editor_state: Arc<ViziaState>,
) -> Option<Box<dyn Editor>> {
    create_vizia_editor(editor_state, ViziaTheming::Custom, move |cx, _| {
        // add new styling
        let _ = cx.add_stylesheet(NEW_STYLE);

        Data {
            params: params.clone(),
        }
            .build(cx);

        VStack::new(cx, |cx| {
            Label::new(cx, "Velocity Mapper")
                .font_weight(Bold)
                .font_size(25.0);

            Element::new(cx).height(Pixels(15.0));

            // TODO this might(!) be abstractable when all ranges are in a vector

            range_selector(cx, Data::params, |params| &params.range1, &1.to_string());

            Element::new(cx).height(Pixels(10.0));

            range_selector(cx, Data::params, |params| &params.range2, &2.to_string());

            Element::new(cx).height(Pixels(10.0));

            range_selector(cx, Data::params, |params| &params.range3, &3.to_string());

            Element::new(cx).height(Pixels(10.0));

            range_selector(cx, Data::params, |params| &params.range4, &4.to_string());

        })
            .alignment(Alignment::TopCenter);
    })
}

fn range_selector<L, Params, FMap>(cx: &mut Context, params: L, params_to_param: FMap, label_suffix: &str)
where
    L: Lens<Target = Params> + Clone,
    Params: 'static,
    FMap: Fn(&Params) -> &RangeParams + 'static + Copy,
{
    HStack::new(cx, move |cx| {
        Element::new(cx).width(Stretch(0.1));

        VStack::new(cx, |cx| {

            Element::new(cx).height(Pixels(5.0));

            Label::new(cx, format!("Mapping {}:", label_suffix))
                .font_weight(Bold)
                .font_size(15.0);

            Element::new(cx).height(Pixels(5.0));

            HStack::new(cx, |cx| {

                Element::new(cx).width(Pixels(10.0));

                // Bypass
                VStack::new(cx, |cx| {
                    Element::new(cx).height(Pixels(37.0));

                    ParamButton::new(cx, params, move |params| &params_to_param(params).bypass)
                        .class("red_button")
                        .font_size(10.0);
                })
                    .alignment(Alignment::TopCenter);

                VStack::new(cx, |cx| {
                    Label::new(cx, "Velocity Range:")
                        .font_size(15.0);

                    Element::new(cx).height(Pixels(5.0));

                    // Min
                    DropDownParam::new(
                        cx,
                        params,
                        move |params| {
                            &params_to_param(params).range_min
                        },
                    );

                    Element::new(cx).height(Pixels(5.0));

                    // Max
                    DropDownParam::new(
                        cx,
                        params,
                        move |params| {
                            &params_to_param(params).range_max
                        },
                    );
                })
                    .width(Stretch(1.0))
                    .alignment(Alignment::TopCenter);

                // Arrow
                VStack::new(cx, |cx| {
                    Element::new(cx).height(Pixels(37.0));

                    // const APPLE: &[u8] = include_bytes!("../assets/apple.jpg");
                    // cx.load_image("appley", APPLE, ImageRetentionPolicy::Forever);
                    // Image::new(cx, "appley");

                    Svg::new(cx, include_str!("../assets/arrow_right.svg"))
                        .height(Pixels(30.0))
                        .width(Stretch(1.0));
                })
                    .width(Stretch(1.0))
                    .alignment(Alignment::TopCenter);

                // Pitch
                VStack::new(cx, |cx| {

                    Element::new(cx).height(Pixels(15.0));

                    Label::new(cx, "Pitch:")
                        .font_size(15.0);

                    Element::new(cx).height(Pixels(5.0));

                    DropDownParam::new(
                        cx,
                        params,
                        move |params| {
                            &params_to_param(params).pitch
                        },
                    );
                })
                    .width(Stretch(1.0))
                    .alignment(Alignment::TopCenter);

                Element::new(cx).width(Pixels(10.0));
            })
                .height(Pixels(90.0))
                .alignment(Alignment::Center);
        })
            .border_color(Color::black())
            .border_width(Pixels(1.0))
            .alignment(Alignment::Center)
            .width(Stretch(2.0))
            .height(Pixels(120.0));

        Element::new(cx).width(Stretch(0.1));
    })
        .height(Pixels(120.0));
}