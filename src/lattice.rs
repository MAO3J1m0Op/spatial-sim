use std::ops;

/// 3D structure used to model the lattice
#[derive(Debug, Clone)]
pub struct Lattice<T> {
    data: Vec<T>,
    pub size: i16,
}

impl<T> Lattice<T> {
    pub fn filled<F: FnMut(LatticeIdx) -> T>(size: i16, mut filler: F) -> Self {
        let data = Vec::with_capacity(size as usize * size as usize * size as usize);

        let mut this = Self { data, size };

        for idx in LatticeIdx::cube_iter(size) {
            this[idx] = filler(idx)
        }

        this
    }
}

impl<T> ops::Index<LatticeIdx> for Lattice<T> {
    type Output = T;

    fn index(&self, idx: LatticeIdx) -> &Self::Output {
        let first = (idx.0.rem_euclid(self.size as i16)) as usize * self.size as usize * self.size as usize;
        let second = (idx.1.rem_euclid(self.size as i16)) as usize * self.size as usize;
        let third = (idx.2.rem_euclid(self.size as i16)) as usize;
        unsafe {
            self.data.get_unchecked(first + second + third)
        }
    }
}

impl<T> ops::IndexMut<LatticeIdx> for Lattice<T> {
    fn index_mut(&mut self, idx: LatticeIdx) -> &mut Self::Output {
        let first = (idx.0.rem_euclid(self.size as i16)) as usize * self.size as usize * self.size as usize;
        let second = (idx.1.rem_euclid(self.size as i16)) as usize * self.size as usize;
        let third = (idx.2.rem_euclid(self.size as i16)) as usize;
        unsafe {
            self.data.get_unchecked_mut(first + second + third)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LatticeIdx(pub i16, pub i16, pub i16);

impl LatticeIdx {

    pub fn cubed(num: i16) -> LatticeIdx {
        LatticeIdx(num, num, num)
    }

    pub fn cube_iter(size: i16) -> impl Iterator<Item = LatticeIdx> {
        BoxIter {
            exhausted: false,
            pos: LatticeIdx(0, 0, 0),
            high: LatticeIdx::cubed(size)
        }
    }

    // pub fn box_iter(low: LatticeIdx, high: LatticeIdx) 
    //     -> impl Iterator<Item = LatticeIdx>
    // {
    //     BoxIter {
    //         exhausted: false,
    //         pos: low,
    //         high
    //     }
    // }

    pub fn neighbor_iter(center: LatticeIdx)
        -> impl Iterator<Item = LatticeIdx>
    {
        NeighborIter {
            center,
            state: 0
        }
    }
}

struct BoxIter {
    exhausted: bool,
    pos: LatticeIdx,
    high: LatticeIdx,
}

impl Iterator for BoxIter {
    type Item = LatticeIdx;

    fn next(&mut self) -> Option<Self::Item> {

        if self.exhausted { return None; }

        let item = self.pos;
        self.pos.2 += 1;
        if self.pos.2 == self.high.2 {
            self.pos.1 += 1;
            self.pos.2 = 0;
            if self.pos.1 == self.high.1 {
                self.pos.0 += 1;
                self.pos.1 = 0;
                if self.pos.0 == self.high.0 {
                    self.exhausted = true;
                    return None;
                }
            }
        }

        Some(item)
    }
}

struct NeighborIter {
    center: LatticeIdx,
    state: u8,
}

impl Iterator for NeighborIter {
    type Item = LatticeIdx;

    fn next(&mut self) -> Option<Self::Item> {
        if self.state == 6 { return None }
        self.state += 1;
        Some(match self.state {
            1 => self.center + LatticeIdx(-1, 0, 0),
            2 => self.center + LatticeIdx( 1, 0, 0),
            3 => self.center + LatticeIdx(0, -1, 0),
            4 => self.center + LatticeIdx(0,  1, 0),
            5 => self.center + LatticeIdx(0, 0, -1),
            6 => self.center + LatticeIdx(0, 0,  1),
            _ => unreachable!(),
        })
    }
}

impl ops::Add for LatticeIdx {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        LatticeIdx(self.0 + rhs.0, self.1 + rhs.1, self.2 + rhs.2)
    }
}

impl ops::AddAssign for LatticeIdx {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
        self.1 += rhs.1;
        self.2 += rhs.2;
    }
}
