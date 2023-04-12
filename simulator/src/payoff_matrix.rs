use crate::State;

/// 3x3 matrix that determines the fitness of each population in the presence of the other.
#[derive(Debug)]
pub struct PayoffMatrix {
    resorption: [f32; 3],
    formation:  [f32; 3],
    quiescence: [f32; 3],
}

impl PayoffMatrix {
    pub fn new(
        resorption: [f32; 3],
        formation:  [f32; 3],
        quiescence: [f32; 3],
    ) -> Self {
        Self { resorption, formation, quiescence }
    }

    pub fn by_params(alpha: [f32; 3], beta: [f32; 3]) -> PayoffMatrix {

        //let theta = 0.485;
        let omega = 0.1;

        Self::new(
            [
                1.0,
                alpha[2] * omega + 1.0,
                beta[1] * omega + 1.0,
            ],
            [
                beta[2] * omega + 1.0,
                1.0,
                alpha[0] * omega + 1.0,
            ],
            [
                alpha[1] * omega + 1.0,
                beta[0] * omega + 1.0,
                1.0
            ]
        )
    }

    pub fn get(&self, cell: State, against: State) -> f32 {

        let idx = match against {
            State::Resorption => 0,
            State::Formation => 1,
            State::Quiescence => 2,
        };

        let array = match cell {
            State::Resorption => &self.resorption,
            State::Formation => &self.formation,
            State::Quiescence => &self.quiescence,
        };

        // SAFETY: idx will always be a valid index; no sense doing a bounds check
        unsafe {
            *array.get_unchecked(idx)
        }
    }
}
