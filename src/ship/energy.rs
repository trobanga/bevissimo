use std::time::Duration;

use bevy::prelude::*;

#[derive(Component, Reflect)]
pub struct Energy {
    pub max: f32,
    pub current: f32,
}

#[derive(Component, Reflect)]
pub struct EnergyDecay(pub f32);

#[derive(Component, Reflect)]
pub struct EnergyTimer(pub Timer);

impl Default for EnergyTimer {
    fn default() -> Self {
        Self(Timer::new(Duration::from_secs(1), true))
    }
}

#[derive(Bundle)]
pub struct EnergyBundle {
    pub energy: Energy,
    pub decay: EnergyDecay,
    pub timer: EnergyTimer,
}

impl Energy {
    pub fn increase(&mut self, amount: f32) {
        self.current += amount;
        if self.current > self.max {
            self.current = self.max;
        }
    }

    #[inline(always)]
    pub fn current(&self) -> f32 {
        self.current
    }

    #[inline(always)]
    pub fn current_percentage(&self) -> f32 {
        self.current / self.max
    }
}

fn decay(energy: &mut Energy, decay: &EnergyDecay) {
    if energy.current > 0.0 {
        energy.current -= decay.0;
        if energy.current < 0.0 {
            energy.current = 0.0;
        }
    }
}
pub struct EnergyPlugin;

impl Plugin for EnergyPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(tick).add_system(handle_timer);
    }
}

fn tick(mut q: Query<(&mut Energy, &EnergyDecay, &mut EnergyTimer)>, time: Res<Time>) {
    for (mut e, d, mut t) in q.iter_mut() {
        t.0.tick(time.delta());
        if t.0.just_finished() {
            decay(&mut e, &d);
        }
    }
}
