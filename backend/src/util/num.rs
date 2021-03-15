use std::{rc::Rc, vec::Vec};

use rand::{
    distributions::uniform::{SampleRange, SampleUniform},
    thread_rng, Rng,
};

// const NUMERIC: [u8; 10] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 0];

pub fn rand_numbers<T>(min: T, max: T, amount: usize) -> Vec<T>
where
    T: SampleUniform + PartialOrd + Copy,
{
    let mut d = Vec::<T>::with_capacity(amount);
    let mut rng = thread_rng();

    for _n in 0..amount {
        //let r = min..max;
        d.push(rng.gen_range(min..max))
    }
    d
}
