use nih_plug::prelude::Param;
use vizia_plug::vizia::prelude::*;
use vizia_plug::widgets::param_base::ParamWidgetBase;
use vizia_plug::widgets::util::{self, ModifiersExt};

/// When shift+dragging a parameter, one pixel dragged corresponds to this much change in the
/// normalized parameter.
const GRANULAR_DRAG_MULTIPLIER: f32 = 0.1;

#[derive(Lens)]
pub struct DropDownParam {
    param_base: ParamWidgetBase,

    /// Will be set to `true` if we're dragging the parameter. Resetting the parameter or entering a
    /// text value should not initiate a drag.
    drag_active: bool,
    /// We keep track of the start coordinate and normalized value when holding down Shift while
    /// dragging for higher precision dragging. This is a `None` value when granular dragging is not
    /// active.
    granular_drag_status: Option<GranularDragStatus>,
    /// The number of (fractional) scrolled lines that have not yet been turned into parameter
    /// change events. This is needed to support trackpads with smooth scrolling.
    scrolled_lines: f32,

    // TODO have an offset for displayed numbers
}

#[derive(Debug, Clone, Copy)]
pub struct GranularDragStatus {
    /// The mouse's Y-coordinate when the granular drag was started.
    pub starting_coordinate: f32,
    /// The normalized value when the granular drag was started.
    pub starting_value: f32,
}

impl DropDownParam {
    /// Creates a new [`DropDownParam`] for the given parameter. To accommodate VIZIA's mapping system,
    /// you'll need to provide a lens containing your `Params` implementation object (check out how
    /// the `Data` struct is used in `gain_gui_vizia`) and a projection function that maps the
    /// `Params` object to the parameter you want to display a widget for. Parameter changes are
    /// handled by emitting [`ParamEvent`][super::ParamEvent]s which are automatically handled by
    /// the VIZIA wrapper.
    pub fn new<L, Params, P, FMap>(
        cx: &mut Context,
        params: L,
        params_to_param: FMap,
    ) -> Handle<Self>
        where
            L: Lens<Target = Params> + Clone,
            Params: 'static,
            P: Param + 'static,
            FMap: Fn(&Params) -> &P + Copy + 'static,
    {
        // We'll visualize the difference between the current value and the default value if the
        // default value lies somewhere in the middle and the parameter is continuous. Otherwise
        // this approach looks a bit jarring.
        Self {
            param_base: ParamWidgetBase::new(cx, params, params_to_param),
            drag_active: false,
            granular_drag_status: None,
            scrolled_lines: 0.0,
        }
            .build(
                cx,
                ParamWidgetBase::build_view(params, params_to_param, move |cx, param_data| {

                    // Can't use `.to_string()` here as that would include the modulation.
                    let unmodulated_normalized_value_lens =
                        param_data.make_lens(|param| param.unmodulated_normalized_value());
                    let display_value_lens = param_data.make_lens(|param| {
                        param.normalized_value_to_string(param.unmodulated_normalized_value(), true)
                    });
                    let steps = param_data.param().step_count().unwrap_or(1);

                    Dropdown::new(
                        cx,
                        // This is the 'Preview'
                        move |cx| {
                            Self::build_preview(cx, display_value_lens);
                        },
                        // POPUP
                        move |cx| {
                            Self::build_popup(cx, steps, unmodulated_normalized_value_lens);
                        },
                    )
                        .on_press(move |cx| cx.emit(PopupEvent::Open))
                        .alignment(Alignment::Center)
                        .width(Stretch(1.0))
                        .height(Stretch(1.0));
                }),
            )
            .width(Pixels(50.0))
            .height(Pixels(25.0))
            .border_color(Color::black())
            .border_width(Pixels(1.0))
    }

    fn build_preview(
        cx: &mut Context,
        display_value_lens: impl Lens<Target = String>,
    ) {
        Label::new(cx, display_value_lens)
            .alignment(Alignment::Center)
            .hoverable(false);
    }

    fn build_popup(
        cx: &mut Context,
        steps: usize,
        val_lens: impl Lens<Target = f32>,
    ) {
        let current_step = (val_lens.get(cx) * steps as f32).round() as usize;

        ScrollView::new(cx, move|cx| {
            for i in 0..=steps {
                Label::new(cx, i)
                    .background_color(if i == current_step { Color::gray() } else { Color::white() })
                    .width(Stretch(1.0))
                    .on_press(move |cx| {
                        cx.emit(DropDownEvent::SetValue(i as f32 / steps as f32));
                        cx.emit(PopupEvent::Close);
                    });
            }
        })
            .scroll_y(val_lens)
            .show_horizontal_scrollbar(false)
            .show_vertical_scrollbar(false)
            .width(Stretch(1.0))
            .height(Pixels(80.0));
    }

    fn set_value(&self, cx: &mut EventContext, value: f32) {
        self.param_base.begin_set_parameter(cx);
        self.param_base.set_normalized_value(cx, value);
        self.param_base.end_set_parameter(cx);
    }

    /// `self.param_base.set_normalized_value()`, but resulting from a mouse drag.
    /// This still needs to be wrapped in a parameter automation gesture.
    fn set_normalized_value_drag(&self, cx: &mut EventContext, normalized_value: f32) {
        let normalized_value =  normalized_value;

        self.param_base.set_normalized_value(cx, normalized_value);
    }
}

enum DropDownEvent {
    SetValue(f32),
}

impl View for DropDownParam {
    fn element(&self) -> Option<&'static str> {
        Some("dropdown-param")
    }

    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|window_event, meta| match window_event {
            DropDownEvent::SetValue(value) => {
                self.set_value(cx, *value);
            }
        });
        event.map(|window_event, meta| match window_event {
            WindowEvent::MouseScroll(_scroll_x, scroll_y) => {
                // With a regular scroll wheel `scroll_y` will only ever be -1 or 1, but with smooth
                // scrolling trackpads being a thing `scroll_y` could be anything.
                self.scrolled_lines += scroll_y;

                if self.scrolled_lines.abs() >= 1.0 {
                    let use_finer_steps = cx.modifiers().shift();

                    // Scrolling while dragging needs to be taken into account here
                    if !self.drag_active {
                        self.param_base.begin_set_parameter(cx);
                    }

                    let mut current_value = self.param_base.unmodulated_normalized_value();

                    while self.scrolled_lines >= 1.0 {
                        current_value = self
                            .param_base
                            .next_normalized_step(current_value, use_finer_steps);
                        self.param_base.set_normalized_value(cx, current_value);
                        self.scrolled_lines -= 1.0;
                    }

                    while self.scrolled_lines <= -1.0 {
                        current_value = self
                            .param_base
                            .previous_normalized_step(current_value, use_finer_steps);
                        self.param_base.set_normalized_value(cx, current_value);
                        self.scrolled_lines += 1.0;
                    }

                    if !self.drag_active {
                        self.param_base.end_set_parameter(cx);
                    }
                }

                meta.consume();
            }
            _ => ()
        });
    }

    // fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
    //     event.map(|window_event, meta| match window_event {
    //         // Vizia always captures the third mouse click as a triple click. Treating that triple
    //         // click as a regular mouse button makes double click followed by another drag work as
    //         // expected, instead of requiring a delay or an additional click. Double double click
    //         // still won't work.
    //         WindowEvent::MouseDown(MouseButton::Left)
    //         | WindowEvent::MouseTripleClick(MouseButton::Left) => {
    //
    //             if cx.modifiers().command() {
    //                 // Ctrl+Click, double click, and right clicks should reset the parameter instead
    //                 // of initiating a drag operation
    //                 self.param_base.begin_set_parameter(cx);
    //                 self.param_base
    //                     .set_normalized_value(cx, self.param_base.default_normalized_value());
    //                 self.param_base.end_set_parameter(cx);
    //             } else {
    //                 // The `!self.text_input_active` check shouldn't be needed, but the textbox does
    //                 // not consume the mouse down event. So clicking on the textbox to move the
    //                 // cursor would also change the slider.
    //                 self.drag_active = true;
    //                 cx.capture();
    //                 // NOTE: Otherwise we don't get key up events
    //                 cx.focus();
    //                 cx.set_active(true);
    //
    //                 // When holding down shift while clicking on a parameter we want to granuarly
    //                 // edit the parameter without jumping to a new value
    //                 self.param_base.begin_set_parameter(cx);
    //                 if cx.modifiers().shift() {
    //                     self.granular_drag_status = Some(GranularDragStatus {
    //                         starting_coordinate: cx.mouse().cursor_x,
    //                         starting_value: self.param_base.unmodulated_normalized_value(),
    //                     });
    //                 } else {
    //                     self.granular_drag_status = None;
    //                     self.set_normalized_value_drag(
    //                         cx,
    //                         1.0 - util::remap_current_entity_y_coordinate(cx, cx.mouse().cursor_y)
    //                     );
    //                 }
    //             }
    //
    //             meta.consume();
    //         }
    //         WindowEvent::MouseDoubleClick(MouseButton::Left)
    //         | WindowEvent::MouseDown(MouseButton::Right)
    //         | WindowEvent::MouseDoubleClick(MouseButton::Right)
    //         | WindowEvent::MouseTripleClick(MouseButton::Right) => {
    //             // TODO all of this should also just open the dropdown right?
    //             // Ctrl+Click, double click, and right clicks should reset the parameter instead of
    //             // initiating a drag operation
    //             self.param_base.begin_set_parameter(cx);
    //             self.param_base
    //                 .set_normalized_value(cx, self.param_base.default_normalized_value());
    //             self.param_base.end_set_parameter(cx);
    //
    //             meta.consume();
    //         }
    //         WindowEvent::MouseUp(MouseButton::Left) => {
    //             // TODO does dragging do the same as scrolling??
    //             if self.drag_active {
    //                 self.drag_active = false;
    //                 cx.release();
    //                 cx.set_active(false);
    //
    //                 self.param_base.end_set_parameter(cx);
    //
    //                 meta.consume();
    //             }
    //         }
    //         WindowEvent::MouseMove(x, y) => {
    //             // TODO does dragging do the same as scrolling??
    //             if self.drag_active {
    //                 // If shift is being held then the drag should be more granular instead of
    //                 // absolute
    //                 if cx.modifiers().shift() {
    //                     let granular_drag_status =
    //                         *self
    //                             .granular_drag_status
    //                             .get_or_insert_with(|| GranularDragStatus {
    //                                 starting_coordinate: *y,
    //                                 starting_value: self.param_base.unmodulated_normalized_value(),
    //                             });
    //
    //                     // These positions should be compensated for the DPI scale so it remains
    //                     // consistent
    //                     let start_y =
    //                         util::remap_current_entity_y_t(cx, granular_drag_status.starting_value);
    //                     let delta_y = ((*y - granular_drag_status.starting_coordinate)
    //                         * GRANULAR_DRAG_MULTIPLIER)
    //                         * cx.scale_factor();
    //
    //                     self.set_normalized_value_drag(
    //                         cx,
    //                         1.0 - util::remap_current_entity_y_coordinate(cx, start_y + delta_y),
    //                     );
    //
    //                 } else {
    //                     self.granular_drag_status = None;
    //
    //                     self.set_normalized_value_drag(
    //                         cx,
    //                         1.0 - util::remap_current_entity_y_coordinate(cx, *y),
    //                     );
    //                 }
    //             }
    //         }
    //         WindowEvent::KeyUp(_, Some(Key::Shift)) => {
    //             // If this happens while dragging, snap back to reality uh I mean the current screen
    //             // position
    //             if self.drag_active && self.granular_drag_status.is_some() {
    //                 self.granular_drag_status = None;
    //                 self.param_base.set_normalized_value(
    //                     cx,
    //                     1.0 - util::remap_current_entity_y_coordinate(cx, cx.mouse().cursor_y),
    //                 );
    //             }
    //         }
    //
    //         _ => {}
    //     });
    // }
}