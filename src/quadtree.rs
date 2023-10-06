use crate::*;
pub(crate) use std::collections::HashMap;
use std::ops::{Index, IndexMut};
pub(crate) use wasm_bindgen::prelude::wasm_bindgen;

// literally the coords
#[derive(Hash, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Region(u32, u32);

pub struct Key(Region, usize);

impl Key {
    pub(crate) fn copy_internal(&self) -> Self {
        Self(self.0, self.1)
    }
}

fn insert<T>(vec: &mut Vec<Option<T>>, value: T) -> usize {
    let mut already_has_none = false;
    let mut index: usize = !0;

    for (i, each) in vec.iter_mut().enumerate() {
        if each.is_none() {
            index = i;
            already_has_none = true;
            break;
        }
    }

    if !already_has_none {
        index = vec.len();
        vec.push(Some(value));
    } else {
        vec[index] = Some(value);
    }

    if index == !0 {
        panic!("something went wrong")
    }
    index
}

pub(crate) struct PartitionMap<T> {
    // each region contains an array of points that could either be top left or bottom right points of a rect
    pub(crate) raw_map: HashMap<Region, Vec<Option<T>>>,
    pub(crate) size: u32,
    pub(crate) div_size: f32,
}

impl<T> PartitionMap<T> {
    // should probably handle more than just square???
    pub(crate) fn new(size: u32, depth: u32) -> Self {
        Self {
            raw_map: HashMap::new(),
            size,
            div_size: size as f32 / (2u32.pow(depth)) as f32,
        }
    }

    pub(crate) fn insert_point(&mut self, point: Vector2d, value: T) -> Result<Key, &'static str> {
        let region = self.point_to_region(point)?;
        let vec = self.raw_map.entry(region).or_insert(vec![]);
        let mut index = insert(vec, value);
        Ok(Key(region, index))
    }

    pub(crate) fn point_to_region(&self, point: Vector2d) -> Result<Region, &'static str> {
        if point.x >= 0.0 && point.x < self.size as _ {
            let div_x = (point.x / self.div_size).trunc() as u32;
            if point.y >= 0.0 && point.y < self.size as _ {
                let div_y = (point.y / self.div_size).trunc() as u32;
                return Ok(Region(div_x, div_y));
            } else {
                return Err("y out of bound");
            }
        } else {
            return Err("x out of bound");
        }
    }

    pub(crate) fn remove_point(&mut self, key: Key) -> Result<T, &'static str> {
        let thing = self
            .raw_map
            .get_mut(&key.0)
            .ok_or("region not yet init in raw map")?;

        if thing[key.1].is_none() {
            return Err("invalid key");
        }
        let mut another = None;
        std::mem::swap(&mut thing[key.1], &mut another);
        Ok(another.unwrap())
    }

    pub(crate) fn move_point(&mut self, key: Key, point: Vector2d) -> Result<Key, &'static str> {
        let val = self.remove_point(key)?;
        Ok(self.insert_point(point, val)?)
    }

    pub(crate) fn points_to_regions(
        &self,
        top_left: Vector2d,
        bottom_right: Vector2d,
    ) -> Result<Vec<Region>, &'static str> {
        let tl = self.point_to_region(top_left)?;
        let br = self.point_to_region(bottom_right)?;

        let mut result_vec: Vec<Region> = vec![];
        for each_y in br.0..tl.0 {
            for each_x in br.1..tl.1 {
                result_vec.push(Region(each_x, each_y));
            }
        }
        Ok(result_vec)
    }
}

impl<T> Index<&Region> for PartitionMap<T> {
    type Output = Vec<Option<T>>;

    fn index(&self, index: &Region) -> &Self::Output {
        self.raw_map.get(index).unwrap()
    }
}

impl<T> IndexMut<&Region> for PartitionMap<T> {
    fn index_mut(&mut self, index: &Region) -> &mut Self::Output {
        self.raw_map.get_mut(index).unwrap()
    }
}

impl<T> Index<&Key> for PartitionMap<T> {
    type Output = T;

    fn index(&self, index: &Key) -> &Self::Output {
        let reg = &self[&index.0];
        let res = &reg[index.1];
        res.as_ref().unwrap()
    }
}

impl<T> IndexMut<&Key> for PartitionMap<T> {
    fn index_mut(&mut self, index: &Key) -> &mut Self::Output {
        let reg = &mut self[&index.0];
        let res = &mut reg[index.1];
        res.as_mut().unwrap()
    }
}
