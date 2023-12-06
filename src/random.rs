use bevy::prelude::*;
use rand::prelude::*;
use rand_chacha::ChaCha20Rng;


#[derive(Component)]
pub struct RandomDeterministic {
    pub random: ChaCha20Rng,
    seed: u64,
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
    pub fn reset(&mut self) {
        *self = Self::new_from_seed(self.seed);
    }
    pub fn get_seed(&self) -> u64 {
        self.seed
    }
}
