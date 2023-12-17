use bevy::prelude::*;
use rand::prelude::*;
use rand_chacha::ChaCha20Rng;

#[allow(dead_code)]
#[derive(Component)]
pub struct RandomDeterministic {
    // TO support reflect, we need https://github.com/bevyengine/bevy/pull/7575
    //#[reflect(skip_serializing)]
    //#[reflect(default = "get_default_random")]
    pub random: ChaCha20Rng,
    seed: u64,
}
fn get_default_random() -> ChaCha20Rng {
    ChaCha20Rng::seed_from_u64(0)
}

impl Default for RandomDeterministic {
    fn default() -> Self {
        let seed = 0; //thread_rng().gen::<u64>();
        Self::new_from_seed(seed)
    }
}

impl RandomDeterministic {
    pub fn new_from_seed(seed: u64) -> RandomDeterministic {
        Self {
            random: ChaCha20Rng::seed_from_u64(seed),
            seed,
        }
    }
    pub fn _reset(&mut self) {
        *self = Self::new_from_seed(self.seed);
    }
    pub fn _get_seed(&self) -> u64 {
        self.seed
    }
}
