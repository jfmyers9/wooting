use crate::effects::EffectKind;
use crate::extensions::KeyboardExtension;
use crate::render::{Frame, RenderContext};
use std::sync::atomic::AtomicBool;

#[derive(Clone, Debug)]
pub struct StaticEffectExtension {
    effect: EffectKind,
}

impl StaticEffectExtension {
    pub fn new(effect: EffectKind) -> Self {
        Self { effect }
    }
}

impl KeyboardExtension for StaticEffectExtension {
    fn tick(&mut self, _interrupted: &AtomicBool) {}

    fn render(&self, ctx: &RenderContext<'_>) -> Frame {
        self.effect.render(ctx)
    }

    fn finished(&self) -> bool {
        false
    }

    fn shutdown(&mut self, _interrupted: bool) {}
}
