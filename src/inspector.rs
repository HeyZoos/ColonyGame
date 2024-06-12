use bevy::prelude::*;
use big_brain::prelude::*;

pub struct InspectorPlugin;

impl Plugin for InspectorPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<ActionState>()
            .register_type::<Actor>()
            .register_type::<Thinker>();
    }
}
