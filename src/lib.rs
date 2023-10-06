#![allow(unused_variables, unused_imports, dead_code, unused_mut)]
#![feature(portable_simd)]
mod quadtree;
use std::{
    f32::consts::PI,
    ops::{Add, AddAssign, Mul, Neg, Sub, SubAssign},
};

use quadtree::*;

// todo, ok but what about a simulation game just like simtower/project highrise but with trains
// you can design trains and attach them onto each other, each carriage is a module, maybe one can act
// as a transformer that turns DC to AC, where the DC power come from another carriage, probably nuclear
// or something... sleeper ones, seat ones and other kinds, dining area, water spa train, shop train

// todo frame animation

// todo, render to texture

// todo bezier curve using the above linear transformation to achieve graceful animation

#[repr(C)]
#[derive(Clone, Copy, PartialEq)]
struct Vector2d {
    x: f32,
    y: f32,
}
impl Vector2d {
    fn set_scale(&mut self, x_scale: f32, y_scale: f32) {
        self.x *= x_scale;
        self.y *= y_scale;
    }

    fn set_rotation(&mut self, rad: f32) {
        // positive rad for clockwise
        // so rotate 180 == scale(-1, -1)
        // x2 = cosβx1 − sinβy1
        // y2 = sinβx1 + cosβy1
        // this formula is counterclockwise rotation
        let angle = 2.0 * PI - rad;
        self.x = angle.cos() * self.x - angle.sin() * self.y;
        self.y = angle.sin() * self.x - angle.cos() * self.y;
    }

    fn to_origin(self) -> Origin {
        Origin { vector2d: self }
    }
}

#[derive(Clone, Copy)]
struct Origin {
    vector2d: Vector2d,
}
impl Origin {
    fn new(x: f32, y: f32) -> Self {
        Self {
            vector2d: Vector2d { x, y },
        }
    }

    fn new_empty() -> Self {
        Self {
            vector2d: Vector2d { x: 0.0, y: 0.0 },
        }
    }

    fn set(&mut self, x: f32, y: f32) {
        self.vector2d.x = x;
        self.vector2d.y = y;
    }

    fn x(&self) -> f32 {
        self.vector2d.x
    }

    fn y(&self) -> f32 {
        self.vector2d.y
    }
}

impl Add for Vector2d {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl AddAssign for Vector2d {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs
    }
}

impl SubAssign for Vector2d {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs
    }
}

impl Sub for Vector2d {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl Neg for Vector2d {
    type Output = Self;

    fn neg(mut self) -> Self::Output {
        self.x *= -1.0;
        self.y *= -1.0;
        self
    }
}

// there's only one copy of Sprite, once it's gone all the sprites are gone
struct Sprite {
    // but now we use quadtree to store both rect and texture how do we remotely handle this??
    data: *mut Scene,
    key_top_left: Option<Key>,
    key_bottom_right: Option<Key>,
    origin: Origin,
}
impl Sprite {
    fn read_rect<'a, 'b>(&'a self) -> &'b Rect {
        unsafe {
            match &(*self.data).part_map[self.key_top_left.as_ref().unwrap()] {
                SpriteDataPoint::TopLeft(data) => &data.rect,
                SpriteDataPoint::BottomRight => panic!("shouldn't be br point"),
            }
        }
    }

    fn read_tex<'a, 'b>(&'a self) -> &'b Texture {
        unsafe {
            match &(*self.data).part_map[self.key_top_left.as_ref().unwrap()] {
                SpriteDataPoint::TopLeft(data) => &data.texture,
                SpriteDataPoint::BottomRight => panic!("shouldn't be br point"),
            }
        }
    }

    fn read_rect_mut<'a, 'b>(&'a mut self) -> &'b mut Rect {
        unsafe {
            match &mut (*self.data).part_map[self.key_top_left.as_ref().unwrap()] {
                SpriteDataPoint::TopLeft(data) => &mut data.rect,
                SpriteDataPoint::BottomRight => panic!("shouldn't be br point"),
            }
        }
    }

    fn read_tex_mut<'a, 'b>(&'a mut self) -> &'b mut Texture {
        unsafe {
            match &mut (*self.data).part_map[self.key_top_left.as_ref().unwrap()] {
                SpriteDataPoint::TopLeft(data) => &mut data.texture,
                SpriteDataPoint::BottomRight => panic!("shouldn't be br point"),
            }
        }
    }

    fn update_keys(&mut self) {
        let bound_rect = self.read_rect().get_bounding_rect();
        self.key_top_left = Some(unsafe {
            (*self.data)
                .part_map
                .move_point(
                    self.key_top_left.as_ref().unwrap().copy_internal(),
                    bound_rect.top_left(),
                )
                .unwrap()
        });
        self.key_bottom_right = Some(unsafe {
            (*self.data)
                .part_map
                .move_point(
                    self.key_bottom_right.as_ref().unwrap().copy_internal(),
                    bound_rect.bottom_right(),
                )
                .unwrap()
        });
    }

    // relative to 0,0 which is the top left of the sprite rect
    fn set_origin(&mut self, offset_x: f32, offset_y: f32) {
        self.origin.set(offset_x, offset_y);
    }

    // of origin
    fn get_pos_origin_global(&self) -> Vector2d {
        self.read_rect().top_left() + self.origin.vector2d
    }

    fn get_pos_top_left(&self) -> Vector2d {
        self.read_rect().top_left()
    }

    // ok but how does ui layer /
    fn set_layer(&mut self, layer: u8, is_ui: bool) {
        let rect = self.read_rect_mut();
        if is_ui {
            rect.set_depth(layer as f32)
        } else {
            rect.set_depth(layer as f32 + 128.0)
        }
        self.update_keys();
    }

    // sets position according to the origin
    fn set_pos(&mut self, x: f32, y: f32) {
        self.offset_pos(Vector2d { x, y } - self.get_pos_origin_global());
    }

    fn offset_pos(&mut self, vector: Vector2d) {
        self.read_rect_mut().offset_pos(vector);
        self.update_keys();
    }

    fn set_size(&mut self, width: f32, height: f32) {
        let x_scale = width / self.read_rect().width();
        let y_scale = height / self.read_rect().height();
        self.set_scale(x_scale, y_scale);
    }

    fn set_scale(&mut self, x_scale: f32, y_scale: f32) {
        self.origin = self
            .read_rect_mut()
            .set_scale_with_origin(x_scale, y_scale, self.origin);
        self.update_keys();
    }

    // totally gonna be careful here, need to get a bounding box whose lines are parallel to axis
    fn set_rotation(&mut self, rad: f32) {
        self.origin = self
            .read_rect_mut()
            .set_rotation_with_origin(rad, self.origin);
        self.update_keys();
    }

    fn set_tex(&mut self, tex: &str) {
        self.disable_float_stencil();
        todo!()
    }

    // only would have an effect if it's an animated texture, that is, a slice of frames with len longer than one
    fn start_playing(&mut self) {
        todo!()
    }

    fn enable_float_stencil(&mut self, bound: (), is_wrapping: bool) {
        // after setting this, the set
        todo!()
    }

    fn disable_float_stencil(&mut self) {
        todo!()
    }

    fn render_to_texture(&mut self) {
        todo!()
    }

    // some kinda special clipping sprite that many sprites can be under???
    // either this or entire opaque layer
    fn set_clipping_rect() {}

    fn remove(self) {
        drop(self)
    }
}
impl Drop for Sprite {
    fn drop(&mut self) {
        unsafe {
            (*self.data)
                .remove_sprite_raw(
                    self.key_top_left.as_ref().unwrap().copy_internal(),
                    self.key_bottom_right.as_ref().unwrap().copy_internal(),
                )
                .unwrap();
        }
    }
}

/// this is for saving only
#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct Tex {
    top_left: (u32, u32),
    bottom_right: (u32, u32),
}
impl Tex {
    fn to_tex(self, width: u32, height: u32) -> Texture {
        Texture {
            top_left: (
                self.top_left.0 as f32 / width as f32,
                self.top_left.1 as f32 / height as f32,
            ),
            bottom_left: (
                self.top_left.0 as f32 / width as f32,
                self.bottom_right.1 as f32 / height as f32,
            ),
            bottom_right: (
                self.bottom_right.0 as f32 / width as f32,
                self.bottom_right.1 as f32 / height as f32,
            ),
            top_right: (
                self.bottom_right.0 as f32 / width as f32,
                self.top_left.1 as f32 / height as f32,
            ),
        }
    }
}

/// this is a base level concept, don't deal with it directly
/// the coords system for this is right -> x++ and down -> y++
#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct Rect {
    top_left: (f32, f32, f32),
    bottom_left: (f32, f32, f32),
    bottom_right: (f32, f32, f32),
    top_right: (f32, f32, f32),
}
impl Rect {
    fn new_raw(x: f32, y: f32, z: f32, width: f32, height: f32) -> Self {
        Self {
            top_left: (x, y, z),
            bottom_left: (x, y + height, z),
            bottom_right: (x + width, y + height, z),
            top_right: (x + width, y, z),
        }
    }

    fn from_two_points(top_left: Vector2d, bottom_right: Vector2d, depth: f32) -> Self {
        // todo what if height/width is negative???
        Self {
            top_left: (top_left.x, top_left.y, depth),
            bottom_left: (top_left.x, bottom_right.y, depth),
            bottom_right: (bottom_right.x, bottom_right.y, depth),
            top_right: (bottom_right.x, top_left.y, depth),
        }
    }

    fn set_raw(&mut self, x: f32, y: f32, z: f32, width: f32, height: f32) {
        self.top_left = (x, y, z);
        self.bottom_left = (x, y - height, z);
        self.bottom_right = (x + width, y - height, z);
        self.top_right = (x + width, y, z);
    }

    // set top left corner
    fn set_pos_top_left(&mut self, x: f32, y: f32) {
        self.set_raw(x, y, self.top_left.2, self.width(), self.height());
    }

    fn offset_pos(&mut self, mut vector: Vector2d) {
        vector += self.top_left();
        self.set_pos_top_left(vector.x, vector.y);
    }

    fn set_pos_with_origin(&mut self, origin: Origin, origin_x: f32, origin_y: f32) {
        let origin_global = origin.vector2d + self.top_left();
        let offset = Vector2d {
            x: origin_x,
            y: origin_y,
        } - origin_global;
        self.offset_pos(offset);
    }

    fn top_left(&self) -> Vector2d {
        Vector2d {
            x: self.top_left.0,
            y: self.top_left.1,
        }
    }

    fn bottom_right(&self) -> Vector2d {
        Vector2d {
            x: self.bottom_right.0,
            y: self.bottom_right.1,
        }
    }

    fn top_right(&self) -> Vector2d {
        Vector2d {
            x: self.top_right.0,
            y: self.top_right.1,
        }
    }

    fn bottom_left(&self) -> Vector2d {
        Vector2d {
            x: self.bottom_left.0,
            y: self.bottom_left.1,
        }
    }

    fn width(&self) -> f32 {
        self.top_right.0 - self.top_left.0
    }

    fn height(&self) -> f32 {
        self.bottom_left.1 - self.top_left.1
    }

    fn set_depth(&mut self, z: f32) {
        self.bottom_left.2 = z;
        self.top_left.2 = z;
        self.bottom_right.2 = z;
        self.top_right.2 = z;
    }

    fn set_top_left(&mut self, pos: Vector2d) {
        self.top_left.0 = pos.x;
        self.top_left.1 = pos.y;
    }

    fn set_top_right(&mut self, pos: Vector2d) {
        self.top_right.0 = pos.x;
        self.top_right.1 = pos.y;
    }

    fn set_bottom_left(&mut self, pos: Vector2d) {
        self.bottom_left.0 = pos.x;
        self.bottom_left.1 = pos.y;
    }

    fn set_bottom_right(&mut self, pos: Vector2d) {
        self.bottom_right.0 = pos.x;
        self.bottom_right.1 = pos.y;
    }

    fn get_bounding_rect(&self) -> Self {
        let top = self.top_left().y.max(
            self.top_right()
                .y
                .max(self.bottom_left().y.max(self.bottom_right().y)),
        );

        let bottom = self.top_left().y.min(
            self.top_right()
                .y
                .min(self.bottom_left().y.min(self.bottom_right().y)),
        );

        let right = self.top_left().x.max(
            self.top_right()
                .x
                .max(self.bottom_left().x.max(self.bottom_right().x)),
        );

        let left = self.top_left().x.min(
            self.top_right()
                .x
                .min(self.bottom_left().x.min(self.bottom_right().x)),
        );
        Self {
            top_left: (top, left, self.top_left.2),
            bottom_left: (bottom, left, self.bottom_left.2),
            bottom_right: (bottom, right, self.bottom_right.2),
            top_right: (top, right, self.top_right.2),
        }
    }

    /// returns the new relative origin
    fn set_rotation_with_origin(&mut self, rad: f32, origin: Origin) -> Origin {
        let origin_global = origin.vector2d + self.top_left();

        let mut tl = self.top_left() - origin_global;
        tl.set_rotation(rad);
        tl += origin_global;
        self.set_top_left(tl);

        let mut tr = self.top_right() - origin_global;
        tr.set_rotation(rad);
        tr += origin_global;
        self.set_top_right(tr);

        let mut bl = self.bottom_left() - origin_global;
        bl.set_rotation(rad);
        bl += origin_global;
        self.set_bottom_left(bl);

        let mut br = self.bottom_right() - origin_global;
        br.set_rotation(rad);
        br += origin_global;
        self.set_top_right(br);

        (origin_global - tl).to_origin()
    }

    /// returns the new relative origin
    fn set_scale_with_origin(&mut self, x_scale: f32, y_scale: f32, origin: Origin) -> Origin {
        let origin_global = origin.vector2d + self.top_left();

        let mut tl = self.top_left() - origin_global;
        tl.set_scale(x_scale, y_scale);
        tl += origin_global;
        self.set_top_left(tl);

        let mut tr = self.top_right() - origin_global;
        tr.set_scale(x_scale, y_scale);
        tr += origin_global;
        self.set_top_right(tr);

        let mut bl = self.bottom_left() - origin_global;
        bl.set_scale(x_scale, y_scale);
        bl += origin_global;
        self.set_bottom_left(bl);

        let mut br = self.bottom_right() - origin_global;
        br.set_scale(x_scale, y_scale);
        br += origin_global;
        self.set_top_right(br);

        (origin_global - tl).to_origin()
    }

    fn center_origin(&self) -> Origin {
        Vector2d {
            x: self.width() / 2.0,
            y: self.height() / 2.0,
        }
        .to_origin()
    }

    fn center_global(&self) -> Vector2d {
        self.top_left()
            + Vector2d {
                x: self.width() / 2.0,
                y: self.height() / 2.0,
            }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
struct Texture {
    top_left: (f32, f32),
    bottom_left: (f32, f32),
    bottom_right: (f32, f32),
    top_right: (f32, f32),
}
impl Texture {
    fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        todo!()
    }

    // the width/height is normalized version
    fn set_tex_raw(&mut self, x: f32, y: f32, width: f32, height: f32) {
        todo!()
    }
}

#[cfg_attr(target_family = "wasm", wasm_bindgen)]
struct WasmVertAttribPtr {
    sprite_pos: *const u8,
    tex_pos: *const u8,
    index: *const u8,

    sprite_pos_len: u32,
    tex_pos_len: u32,
    index_len: u32,
}

impl WasmVertAttribPtr {
    fn new(render_data: &RenderData) -> Self {
        WasmVertAttribPtr {
            sprite_pos: render_data.sprite_pos.as_ptr() as _,
            sprite_pos_len: render_data.sprite_pos.len() as u32 * 12,
            tex_pos: render_data.tex_pos.as_ptr() as _,
            tex_pos_len: render_data.tex_pos.len() as u32 * 8,
            index: render_data.index.as_ptr() as _,
            index_len: render_data.sprite_pos.len() as u32 * 6,
        }
    }
}

struct TextureMap {
    map: HashMap<String, Texture>,
    size: (u32, u32),
}
impl TextureMap {
    fn new(width: u32, height: u32) -> Self {
        todo!()
    }

    fn load_texture_map(
        &mut self,
        map: HashMap<String, Tex>,
        width: u32,
        height: u32,
    ) -> Result<(), &'static str> {
        todo!()
    }

    // return a default sized rect
    fn get(&self, texture: &str) -> Result<(Texture, Rect), &'static str> {
        todo!()
    }
}

// this is for occlusion only, you need to do a matrix transformation in shader to get the correct
// rendering
struct Camera {
    rect: Rect,
}
impl Camera {
    fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        // z is irrelavent for cam
        Camera {
            rect: Rect::new_raw(x, y, 0.0, width, height),
        }
    }

    // the center of the cam
    fn set_center_pos(&mut self, x: f32, y: f32) {
        self.rect
            .set_pos_with_origin(self.rect.center_origin(), x, y);
    }

    fn set_zoom_level(&mut self, scale: f32) {
        self.rect
            .set_scale_with_origin(scale, scale, self.rect.center_origin());
    }
}

struct SpriteData {
    rect: Rect,
    texture: Texture,
}

enum SpriteDataPoint {
    TopLeft(SpriteData),
    BottomRight,
}

impl PartitionMap<SpriteDataPoint> {
    pub(crate) fn query_n_load(
        &mut self,
        top_left: Vector2d,
        bottom_right: Vector2d,
        attrib: &mut RenderData,
    ) -> Result<(), &'static str> {
        let regions = self.points_to_regions(top_left, bottom_right)?;
        for reg in regions {
            let vec = self.raw_map.get(&reg).unwrap();
            for each in vec {
                if each.is_some() {
                    match each.as_ref().unwrap() {
                        SpriteDataPoint::TopLeft(data) => attrib.load(data),
                        SpriteDataPoint::BottomRight => {}
                    }
                }
            }
        }
        Ok(())
    }
}

struct RenderData {
    sprite_pos: Vec<Rect>,
    tex_pos: Vec<Texture>,
    index: Vec<u16>,
}

impl RenderData {
    fn new() -> Self {
        Self {
            sprite_pos: Vec::with_capacity(64),
            tex_pos: Vec::with_capacity(64),
            index: vec![0, 1, 3, 1, 2, 3],
        }
    }

    fn load(&mut self, sprite: &SpriteData) {
        self.sprite_pos.push(sprite.rect);
        self.tex_pos.push(sprite.texture);
    }

    fn ensure_index_len(&mut self, size: usize) {
        if self.index.len() < size {
            self.index.reserve(self.index.len());
            unsafe {
                let ptr = self.index.as_mut_ptr_range();
                self.index.set_len(self.index.len() * 2);
                std::ptr::copy(ptr.start, ptr.end, self.index.len() / 2);
            }
            // only if there a simd impl of the same thing
            for index in self.index.len() / 2..self.index.len() {
                self.index[index] += (self.index.len() / 3) as u16;
            }
            // log_str(&format!("{:?}, len is {:?}", self.index, self.index.len()));
            self.ensure_index_len(size);
        }
    }

    fn clear(&mut self) {
        self.sprite_pos.clear();
        self.tex_pos.clear();
        // we leave index array as is
    }
}

/// a fixed sized partition tree that represent a scene, this one is for occlusion culling
/// should have another quadtree for broad phase collision detection
struct Scene {
    vert_attrib: RenderData,
    part_map: PartitionMap<SpriteDataPoint>,
    tex_map: TextureMap,
}

impl Scene {
    fn new_empty(size: u32, depth: u32, texture_map: TextureMap) -> Self {
        Self {
            // this really feels like UB but it's not?????
            tex_map: texture_map,
            vert_attrib: RenderData::new(),
            part_map: PartitionMap::new(size, depth),
        }
    }

    // ok but how does the coord system works
    fn add_sprite(&mut self, pos: Vector2d, texture: &str) -> Result<Sprite, &'static str> {
        let (tex, mut rect) = self.tex_map.get(texture)?;
        rect.set_pos_top_left(pos.x, pos.y);

        let tl_data = SpriteDataPoint::TopLeft(SpriteData { rect, texture: tex });
        let br_data = SpriteDataPoint::BottomRight;

        // basically after inserting this point the pos data is lost since it's griddified already
        let tl_key = self.part_map.insert_point(pos, tl_data)?;
        let br_key = self.part_map.insert_point(pos, br_data)?;

        // you don't need to get bounding box since there's no rotation going on here
        let sprite = Sprite {
            data: self,
            key_top_left: Some(tl_key),
            key_bottom_right: Some(br_key),
            origin: Origin::new_empty(),
        };
        Ok(sprite)
    }

    fn remove_sprite_raw(
        &mut self,
        id_topleft: Key,
        id_bottomright: Key,
    ) -> Result<(), &'static str> {
        self.part_map.remove_point(id_topleft)?;
        self.part_map.remove_point(id_bottomright)?;
        Ok(())
    }

    fn update(&mut self, cam: &Camera) -> WasmVertAttribPtr {
        self.vert_attrib.clear();

        self.part_map
            .query_n_load(
                cam.rect.top_left(),
                cam.rect.bottom_right(),
                &mut self.vert_attrib,
            )
            .unwrap();

        self.vert_attrib
            .ensure_index_len(self.vert_attrib.sprite_pos.len());

        WasmVertAttribPtr::new(&self.vert_attrib)
    }

    // ui and world can each only have one layer with ysort enabled
    fn enable_ysort(&mut self, layer: u8, is_ui: bool) -> Result<(), &'static str> {
        todo!()
    }

    fn disable_ysort(&mut self, is_ui: bool) {
        todo!()
    }
}
