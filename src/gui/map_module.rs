use nih_plug::prelude::Param;
use vizia_plug::vizia::prelude::*;
use vizia_plug::widgets::param_base::ParamWidgetBase;
use vizia_plug::widgets::util::{self, ModifiersExt};

/// When shift+dragging a parameter, one pixel dragged corresponds to this much change in the
/// normalized parameter.
const GRANULAR_DRAG_MULTIPLIER: f32 = 0.1;

#[derive(Lens)]
pub struct MapModule {
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
}

#[derive(Debug, Clone, Copy)]
pub struct GranularDragStatus {
    /// The mouse's Y-coordinate when the granular drag was started.
    pub starting_coordinate: f32,
    /// The normalized value when the granular drag was started.
    pub starting_value: f32,
}

impl MapModule {
    /// Creates a new [`MapModule`] for the given parameter. To accommodate VIZIA's mapping system,
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

                    // The resulting tuple `(start_t, delta)` corresponds to the start and the
                    // signed width of the bar. `start_t` is in `[0, 1]`, and `delta` is in
                    // `[-1, 1]`.
                    let fill_start_delta_lens =
                        unmodulated_normalized_value_lens.map(move |current_value| {
                            Self::compute_fill_start_delta(
                                *current_value,
                            )
                        });

                    ZStack::new(cx, |cx| {

                    })
                        .hoverable(false);
                }),
            )
            // To override the css styling:
            .border_color(RGBA::rgba(250, 250, 250, 0))
            .background_color(RGBA::rgba(250, 250, 250, 0))
            .width(Pixels(20.0))
            .height(Pixels(180.0))
    }

    fn compute_fill_start_delta(
        current_value: f32,
    ) -> (f32, f32) {

        (
            0.0,
            current_value,
        )
    }

    /// `self.param_base.set_normalized_value()`, but resulting from a mouse drag.
    /// This still needs to be wrapped in a parameter automation gesture.
    fn set_normalized_value_drag(&self, cx: &mut EventContext, normalized_value: f32) {
        let normalized_value =  normalized_value;

        self.param_base.set_normalized_value(cx, normalized_value);
    }
}

impl View for MapModule {
    fn element(&self) -> Option<&'static str> {
        Some("map-module")
    }

    // TODO this handles all events for the entire block, how do I handle just the dropdowns?
    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|window_event, meta| match window_event {
            // Vizia always captures the third mouse click as a triple click. Treating that triple
            // click as a regular mouse button makes double click followed by another drag work as
            // expected, instead of requiring a delay or an additional click. Double double click
            // still won't work.
            WindowEvent::MouseDown(MouseButton::Left)
            | WindowEvent::MouseTripleClick(MouseButton::Left) => {
                if cx.modifiers().command() {
                    // Ctrl+Click, double click, and right clicks should reset the parameter instead
                    // of initiating a drag operation
                    self.param_base.begin_set_parameter(cx);
                    self.param_base
                        .set_normalized_value(cx, self.param_base.default_normalized_value());
                    self.param_base.end_set_parameter(cx);
                } else {
                    // The `!self.text_input_active` check shouldn't be needed, but the textbox does
                    // not consume the mouse down event. So clicking on the textbox to move the
                    // cursor would also change the slider.
                    self.drag_active = true;
                    cx.capture();
                    // NOTE: Otherwise we don't get key up events
                    cx.focus();
                    cx.set_active(true);

                    // When holding down shift while clicking on a parameter we want to granuarly
                    // edit the parameter without jumping to a new value
                    self.param_base.begin_set_parameter(cx);
                    if cx.modifiers().shift() {
                        self.granular_drag_status = Some(GranularDragStatus {
                            starting_coordinate: cx.mouse().cursor_y,
                            starting_value: self.param_base.unmodulated_normalized_value(),
                        });
                    } else {
                        self.granular_drag_status = None;
                        self.set_normalized_value_drag(
                            cx,
                            1.0 - util::remap_current_entity_y_coordinate(cx, cx.mouse().cursor_y)
                        );
                    }
                }

                meta.consume();
            }
            WindowEvent::MouseDoubleClick(MouseButton::Left)
            | WindowEvent::MouseDown(MouseButton::Right)
            | WindowEvent::MouseDoubleClick(MouseButton::Right)
            | WindowEvent::MouseTripleClick(MouseButton::Right) => {
                // Ctrl+Click, double click, and right clicks should reset the parameter instead of
                // initiating a drag operation
                self.param_base.begin_set_parameter(cx);
                self.param_base
                    .set_normalized_value(cx, self.param_base.default_normalized_value());
                self.param_base.end_set_parameter(cx);

                meta.consume();
            }
            WindowEvent::MouseUp(MouseButton::Left) => {
                if self.drag_active {
                    self.drag_active = false;
                    cx.release();
                    cx.set_active(false);

                    self.param_base.end_set_parameter(cx);

                    meta.consume();
                }
            }
            WindowEvent::MouseMove(x, y) => {
                if self.drag_active {
                    // If shift is being held then the drag should be more granular instead of
                    // absolute
                    if cx.modifiers().shift() {
                        let granular_drag_status =
                            *self
                                .granular_drag_status
                                .get_or_insert_with(|| GranularDragStatus {
                                    starting_coordinate: *y,
                                    starting_value: self.param_base.unmodulated_normalized_value(),
                                });

                        // These positions should be compensated for the DPI scale so it remains
                        // consistent
                        let start_y =
                            util::remap_current_entity_y_t(cx, granular_drag_status.starting_value);
                        let delta_y = ((*y - granular_drag_status.starting_coordinate)
                            * GRANULAR_DRAG_MULTIPLIER)
                            * cx.scale_factor();

                        self.set_normalized_value_drag(
                            cx,
                            1.0 - util::remap_current_entity_y_coordinate(cx, start_y + delta_y),
                        );

                    } else {
                        self.granular_drag_status = None;

                        self.set_normalized_value_drag(
                            cx,
                            1.0 - util::remap_current_entity_y_coordinate(cx, *y),
                        );
                    }
                }
            }
            WindowEvent::KeyUp(_, Some(Key::Shift)) => {
                // If this happens while dragging, snap back to reality uh I mean the current screen
                // position
                if self.drag_active && self.granular_drag_status.is_some() {
                    self.granular_drag_status = None;
                    self.param_base.set_normalized_value(
                        cx,
                        1.0 - util::remap_current_entity_y_coordinate(cx, cx.mouse().cursor_y),
                    );
                }
            }
            WindowEvent::MouseScroll(_scroll_x, scroll_y) => {
                // TODO Scrollwheel should change options up or down.
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
            _ => {}
        });
    }
}