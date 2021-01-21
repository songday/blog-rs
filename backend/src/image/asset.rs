use crate::util::num;

pub struct NumberImage {
    pub data: &'static [u8],
}

include!(concat!("number_image.rs"));

pub fn rand_group_number_image(num_pos: usize) -> &'static NumberImage {
    let group = num::rand_numbers(0, 4, 1);
    &NUMBER_IMAGE_GROUPS[group[0]][num_pos]
}
