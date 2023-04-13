use rand::Rng;
use rand_distr::{Exp1, Uniform, Standard, Distribution};

use crate::payoff_matrix::PayoffMatrix;
use crate::lattice::{Lattice, LatticeIdx};

/// Stores the state of the lattice, the fitness associated with each lattice
/// point, and the time.
#[derive(Debug)]
pub struct BoneLattice {
    data: Lattice<(State, f32)>,
    pub time: f32,
    payoff_matrix: PayoffMatrix,
}

impl BoneLattice {
    /// Constructs a new lattice with the specified side length.
    pub fn new<F: FnMut(LatticeIdx) -> State>(
        size: i16, matrix:
        PayoffMatrix,
        mut filler: F
    ) -> Self {
        let mut this = Self {
            data: Lattice::filled(size, |idx| (filler(idx), 0.0)),
            time: 0.0,
            payoff_matrix: matrix,
        };

        // Generate initial fitness for every value
        for idx in LatticeIdx::cube_iter(this.size()) {
            this.gen_fitness(idx);
        }

        this
    }

    pub fn size(&self) -> i16 {
        self.data.size
    }

    pub fn state(&self, idx: LatticeIdx) -> &State {
        &self.data[idx].0
    }

    pub fn state_mut(&mut self, idx: LatticeIdx) -> &mut State {
        &mut self.data[idx].0
    }

    /// Computes the fitness of a cell at a particular index by looking at its
    /// neighbors
    pub fn gen_fitness(&mut self, idx: LatticeIdx) {

        let current_state = *self.state(idx);

        let fitness = LatticeIdx::neighbor_iter(idx)
            .map(|neighbor| {
                self.payoff_matrix.get(current_state, *self.state(neighbor))
            })
            .sum();

        *self.fitness_mut(idx) = fitness;
    }

    /// Returns the stored fitness of a cell at a particular point.
    pub fn stored_fitness(&self, idx: LatticeIdx) -> &f32 {
        &self.data[idx].1
    }

    /// Returns the fitness of a cell at a particular point.
    pub fn fitness_mut(&mut self, idx: LatticeIdx) -> &mut f32 {
        &mut self.data[idx].1
    }

    /// Performs one time step in the simulation.
    #[must_use]
    pub fn step(&mut self) -> (LatticeIdx, State) {
        let mut rng = rand::thread_rng();
        let mut min_time = f32::INFINITY;
        let mut min_time_idx = LatticeIdx(0, 0, 0);

        // Compute expected times of invasion based on each value's fitness, and
        // find the lowest
        for idx in LatticeIdx::cube_iter(self.size()) {
            let lambda = *self.stored_fitness(idx);
            let time = rng.sample::<f32, _>(Exp1) / lambda;
            //dbg!((lambda, time));
            if time < min_time {
                min_time = time;
                min_time_idx = idx;
            }
        }

        // Choose a neighbor uniformly at random to invade
        let invaded = match rng.sample::<u8, _>(Uniform::new(0, 6)) {
            0 => LatticeIdx(-1, 0, 0),
            1 => LatticeIdx(1, 0, 0),
            2 => LatticeIdx(0, -1, 0),
            3 => LatticeIdx(0, 1, 0),
            4 => LatticeIdx(0, 0, -1),
            5 => LatticeIdx(0, 0, 1),
            _ => unreachable!(),
        };

        // Invade the neighbor
        let invasion_state = *self.state(min_time_idx);
        *self.state_mut(min_time_idx + invaded) = invasion_state;

        // Regenerate fitness for the neighbors surrounded
        for idx in LatticeIdx::neighbor_iter(min_time_idx) {
            self.gen_fitness(idx);
        }

        self.time += min_time;

        // Return info about what was changed
        (min_time_idx, invasion_state)
    }

    /// Gets the number of cells in each state, with 0 being resorption, 1 being
    /// formation, and 2 being quiescence.
    pub fn count(&self) -> (usize, usize, usize) {
        let mut count: (usize, usize, usize) = (0, 0, 0);
        for idx in LatticeIdx::cube_iter(self.size()) {
            match self.state(idx) {
                State::Resorption => count.0 += 1,
                State::Formation => count.1 += 1,
                State::Quiescence => count.2 += 1,
            }
        }
        count
    }
}

/// The three populations that are competing
#[derive(Debug, Clone, Copy)]
pub enum State {
    Resorption,
    Formation,
    Quiescence
}

impl Distribution<State> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> State {
        let number = rng.sample::<f32, _>(Uniform::new(0.0, 1.0));
        if number < 0.2 {
            State::Resorption
        } else if number < 0.5 {
            State::Formation
        } else {
            State::Quiescence
        }
    }
}
