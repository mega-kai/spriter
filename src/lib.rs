#![allow(unused_variables, unused_imports, dead_code, unused_mut)]
#![feature(portable_simd)]
mod quadtree;
use std::ops::{Add, AddAssign, Mul, Sub, SubAssign};

use quadtree::*;

// todo, ok but what about a simulation game just like simtower/project highrise but with trains
// you can design trains and attach them onto each other, each carriage is a module, maybe one can act
// as a transformer that turns DC to AC, where the DC power come from another carriage, probably nuclear
// or something... sleeper ones, seat ones and other kinds, dining area, water spa train, shop train

// todo frame animation

// todo, render to texture

// todo, redo the api so there's an object that returns a read only slice containing all the render data;
// or in wasm's case, as raw pointers

// todo bezier curve using the above linear transformation to achieve graceful animation

#[repr(C)]
#[derive(Clone, Copy)]
struct Vector2d {
    x: f32,
    y: f32,
}
impl Vector2d {
    fn dot(self, another: Self) -> f32 {
        todo!()
    }

    fn cross(self, another: Self) -> Self {
        todo!()
    }

    fn set_scale(&mut self, x_scale: f32, y_scale: f32) {
        todo!()
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

// there's only one copy of Sprite, once it's gone all the sprites are gone
struct Sprite {
    // but now we use quadtree to store both rect and texture how do we remotely handle this??
    data: *mut Scene,
    id_topleft: Option<Key>,
    id_bottomright: Option<Key>,
    origin: Origin,
}
impl Sprite {
    fn read_rect<'a, 'b>(&'a self) -> &'b Rect {
        unsafe {
            match &(*self.data).part_map[self.id_topleft.as_ref().unwrap()] {
                SpriteDataPoint::TopLeft(data) => &data.rect,
                SpriteDataPoint::BottomRight => panic!("shouldn't be br point"),
            }
        }
    }

    fn read_tex<'a, 'b>(&'a self) -> &'b Texture {
        unsafe {
            match &(*self.data).part_map[self.id_topleft.as_ref().unwrap()] {
                SpriteDataPoint::TopLeft(data) => &data.texture,
                SpriteDataPoint::BottomRight => panic!("shouldn't be br point"),
            }
        }
    }

    fn read_rect_mut<'a, 'b>(&'a mut self) -> &'b mut Rect {
        unsafe {
            match &mut (*self.data).part_map[self.id_topleft.as_ref().unwrap()] {
                SpriteDataPoint::TopLeft(data) => &mut data.rect,
                SpriteDataPoint::BottomRight => panic!("shouldn't be br point"),
            }
        }
    }

    fn read_tex_mut<'a, 'b>(&'a mut self) -> &'b mut Texture {
        unsafe {
            match &mut (*self.data).part_map[self.id_topleft.as_ref().unwrap()] {
                SpriteDataPoint::TopLeft(data) => &mut data.texture,
                SpriteDataPoint::BottomRight => panic!("shouldn't be br point"),
            }
        }
    }

    fn update_keys(&mut self) {
        let bound_rect = self.read_rect().get_bound_rect();
        self.id_topleft = Some(unsafe { (*self.data).part_map.move_point(key, point) });
        todo!()
    }

    // relative to 0,0 which is the top left of the sprite rect
    fn set_origin(&mut self, offset_x: f32, offset_y: f32) {
        self.origin.set(offset_x, offset_y);
    }

    // of origin
    fn get_pos_origin(&self) -> Vector2d {
        self.read_rect().top_left() + self.origin.vector2d
    }

    fn get_pos_top_left(&self) -> Vector2d {
        self.read_rect().top_left()
    }

    // ok but how does ui layer /
    fn set_layer(&mut self, layer: u8, is_ui: bool) {
        todo!()
    }

    // sets position for the origin
    fn set_pos(&mut self, x: f32, y: f32) {
        // todo this logic
        let top_left = self.read_rect().top_left();
        let origin_pos_abs = self.get_pos_origin();
        let delta_x = x - origin_pos_abs.x;
        let delta_y = y - origin_pos_abs.y;
        self.offset_pos(delta_x, delta_y);
    }

    fn offset_pos(&mut self, delta_x: f32, delta_y: f32) {
        self.read_rect_mut().offset_pos(delta_x, delta_y);
        self.update_keys();
    }

    fn set_size(&mut self, width: f32, height: f32) {
        let x_scale = width / self.read_rect().width();
        let y_scale = height / self.read_rect().height();
        self.set_scale(x_scale, y_scale);
    }

    fn set_scale(&mut self, x_scale: f32, y_scale: f32) {
        // this will scale things up linearly with the 0,0 being the origin
        let origin = self.get_pos_origin();
        let rect = self.read_rect_mut();

        // relative to origin
        let mut tl = rect.top_left() - origin;
        tl.set_scale(x_scale, y_scale);
        tl += origin;
        rect.set_top_left(tl);

        let mut tr = rect.top_right() - origin;
        tr.set_scale(x_scale, y_scale);
        tr += origin;
        rect.set_top_right(tr);

        let mut bl = rect.bottom_left() - origin;
        bl.set_scale(x_scale, y_scale);
        bl += origin;
        rect.set_bottom_left(bl);

        let mut br = rect.bottom_right() - origin;
        br.set_scale(x_scale, y_scale);
        br += origin;
        rect.set_top_right(br);

        self.update_keys();
    }

    // totally gonna be careful here, need to get a bounding box whose lines are parallel to axis
    fn set_rotation(&mut self, degree: ()) {
        todo!()
    }

    // this is would disable the previous enable float stencil setting
    fn set_tex(&mut self, tex: &str) {
        // which is really also a texture
        todo!()
    }

    // only would have an effect if it's an animated texture, that is, a slice of frames with len longer than one
    fn start_playing(&mut self) {
        todo!()
    }

    // todo, set bound and wrapping/clamping
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
            let mut tl: Option<Key> = None;
            let mut br: Option<Key> = None;
            std::mem::swap(&mut tl, &mut self.id_topleft);
            std::mem::swap(&mut br, &mut self.id_bottomright);
            (*self.data)
                .remove_sprite_raw(tl.unwrap(), br.unwrap())
                .unwrap();
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct Tex {
    top_left: (u32, u32),
    bottom_right: (u32, u32),
}
impl Tex {
    fn to_tex(self, width: u32, height: u32) -> Texture {
        todo!()
    }
}

/// this is a base level concept, don't deal with it directly
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
            bottom_left: (x, y - height, z),
            bottom_right: (x + width, y - height, z),
            top_right: (x + width, y, z),
        }
    }

    fn set(&mut self, x: f32, y: f32, z: f32, width: f32, height: f32) {
        self.top_left = (x, y, z);
        self.bottom_left = (x, y - height, z);
        self.bottom_right = (x + width, y - height, z);
        self.top_right = (x + width, y, z);
    }

    // top left corner
    fn set_pos(&mut self, x: f32, y: f32) {
        self.set(x, y, self.top_left.2, self.width(), self.height());
    }

    fn offset_pos(&mut self, delta_x: f32, delta_y: f32) {
        todo!()
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
        todo!()
    }

    fn bottom_left(&self) -> Vector2d {
        todo!()
    }

    fn width(&self) -> f32 {
        self.top_right.0 - self.top_left.0
    }

    fn height(&self) -> f32 {
        self.bottom_left.1 - self.top_left.1
    }

    fn set_layer(&mut self, layer: u8) {
        todo!()
    }

    // todo, check if changing single point would interfere with visible occlusion
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

    fn get_bound_rect(&self) -> Self {
        todo!()
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
    /// todo transform this to the old start from top left thing
    fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            top_left: (x, y + height),
            bottom_left: (x, y),
            bottom_right: (x + width, y),
            top_right: (x + width, y + height),
        }
    }

    // the width/height is normalized version
    fn set_tex(&mut self, x: f32, y: f32, width: f32, height: f32) {
        self.top_left = (x, y + height);
        self.bottom_left = (x, y);
        self.bottom_right = (x + width, y);
        self.top_right = (x + width, y + height);
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
    fn new() -> Self {
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
    fn new() -> Self {
        todo!()
    }

    // the center of the cam
    fn set_center_pos(&mut self, x: f32, y: f32) {
        // self.rect.set_pos(x, y);
        todo!()
    }

    fn set_zoom_level(&mut self, scale: f32) {
        todo!()
    }

    fn to_rect(&self) -> Rect {
        todo!()
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
    fn new_empty(size: u32, depth: u32) -> Self {
        Self {
            // this really feels like UB but it's not?????
            tex_map: TextureMap::new(),
            vert_attrib: RenderData::new(),
            part_map: PartitionMap::new(size, depth),
        }
    }

    // ok but how does the coord system works
    fn add_sprite(&mut self, pos: Vector2d, texture: &str) -> Result<Sprite, &'static str> {
        let (tex, mut rect) = self.tex_map.get(texture)?;
        rect.set_pos(pos.x, pos.y);

        let tl_data = SpriteDataPoint::TopLeft(SpriteData { rect, texture: tex });
        let br_data = SpriteDataPoint::BottomRight;

        // basically after inserting this point the pos data is lost since it's griddified already
        let tl_key = self.part_map.insert_point(pos, tl_data)?;
        let br_key = self.part_map.insert_point(pos, br_data)?;

        // you don't need to get bounding box since there's no rotation going on here
        let sprite = Sprite {
            data: self,
            id_topleft: Some(tl_key),
            id_bottomright: Some(br_key),
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
