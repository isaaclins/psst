use druid::{widget::Controller, Data, Env, Event, EventCtx, MouseButton, MouseEvent, Widget};

pub struct DoubleClick<T> {
    button: Option<MouseButton>,
    action: Box<dyn Fn(&mut EventCtx, &MouseEvent, &mut T, &Env)>,
}

impl<T: Data> DoubleClick<T> {
    pub fn new(
        button: Option<MouseButton>,
        action: impl Fn(&mut EventCtx, &MouseEvent, &mut T, &Env) + 'static,
    ) -> Self {
        Self {
            button,
            action: Box::new(action),
        }
    }
}

impl<T: Data, W: Widget<T>> Controller<T, W> for DoubleClick<T> {
    fn event(&mut self, child: &mut W, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env) {
        if let Event::MouseDown(mouse_event) = event {
            if mouse_event.count >= 2
                && mouse_event.button == self.button.unwrap_or(mouse_event.button)
            {
                (self.action)(ctx, mouse_event, data, env);
            }
        }

        child.event(ctx, event, data, env);
    }
}
