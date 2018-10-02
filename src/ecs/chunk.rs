use specs::prelude::*;

use na::{self, Vector2, Point2};

use ndarray::prelude::*;

use ecs::rect::RectComponent;

use utils::math::Rect;
use utils::constants::CHUNK_SIZE;
use utils::camera::Camera;

pub struct ActiveChunksRect {
    rect: Rect,
    scale: f32,
    dirty: bool
}

impl ActiveChunksRect {
    pub fn new(rect: Rect, scale: f32) -> Self {
        ActiveChunksRect { rect, scale, dirty: true }
    }

    pub fn get_rect(&self) -> &Rect { &self.rect }

    pub fn move_by(&mut self, by: &Vector2<f64>) {
        self.rect.move_by(by);
        self.dirty = true;
    }

    pub fn move_to(&mut self, to: &Point2<f64>) {
        if self.rect.pos != *to {
            self.rect.move_to(to);
            self.dirty = true;
        }
    }

    pub fn update_camera(&mut self, camera: &Camera) {
        self.move_to(&camera.location_zero());
        let view_size = &camera.view_size();
        self.rect.resize_to(&Vector2::new(view_size.x as u32, view_size.y as u32));
    }

    pub fn update_dirty(&mut self) -> bool {
        if self.dirty {
            self.dirty = false;
            true
        } else {
            false
        }
    }
}

pub type Chunk = (usize, usize);

#[derive(Component, Default, Debug)]
#[storage(NullStorage)]
pub struct ActiveChunkMarker;

#[derive(Component, Default)]
pub struct ChunkComponent {
    pub chunks: Vec<Chunk>
}

pub struct ChunkSystem {
    chunks: Array2<Vec<Entity>>,
    chunks_rect: Rect,
    num_cells: (usize, usize),
    dirty_insert: BitSet,
    dirty_modify: BitSet,
    dirty_remove: BitSet,
    insert_rect_reader: Option<ReaderId<InsertedFlag>>,
    modify_rect_reader: Option<ReaderId<ModifiedFlag>>,
    remove_rect_reader: Option<ReaderId<RemovedFlag>>
}

impl ChunkSystem {
    pub fn new((width, height): (usize, usize), _active_rect: Rect) -> Self {
        let chunks: Array2<Vec<Entity>> = Array::default((width, height));
        let chunks_rect = Rect::new(0., 0., CHUNK_SIZE * width as u32, CHUNK_SIZE * height as u32);
        ChunkSystem {
            chunks,
            chunks_rect,
            num_cells: (width, height),
            dirty_insert: BitSet::new(),
            dirty_modify: BitSet::new(),
            dirty_remove: BitSet::new(),
            insert_rect_reader: None,
            modify_rect_reader: None,
            remove_rect_reader: None
        }
    }

    /// TODO Remove
    pub fn get_chunks_rects(&self) -> Vec<Rect> {
        let mut rects = vec![];

        for x in 0..self.num_cells.0 {
            for y in 0..self.num_cells.1 {
                rects.push(Self::get_chunk_rect((x, y)));
            }
        }

        rects
    }

    pub fn get_bounds_chunks(&self) -> &Rect { &self.chunks_rect }

    fn get_chunk_rect(chunk: Chunk) -> Rect {
        Rect::new(chunk.0 as f64 * CHUNK_SIZE as f64, chunk.1 as f64 * CHUNK_SIZE as f64, CHUNK_SIZE as u32, CHUNK_SIZE as u32)
    }

    fn get_chunks_overlap_rect(&self, rect: &Rect) -> Vec<Chunk> {
        let mut overlaps_chunks = vec![];

        let chunk_contains = |(x, y): Chunk| -> bool {
            x < self.num_cells.0 && y < self.num_cells.1 && Self::get_chunk_rect((x, y)).overlaps(rect)
        };

        if self.chunks_rect.overlaps(rect) {
            let mut x = na::max(0, rect.pos.x as usize / CHUNK_SIZE as usize);
            let mut y = na::max(0, rect.pos.y as usize / CHUNK_SIZE as usize);

            if  chunk_contains((x, y)) {
                overlaps_chunks.push((x, y));

                let first_x_chunk = x;

                loop {
                    if chunk_contains((x + 1, y)) {
                        x += 1;
                        overlaps_chunks.push((x, y));
                        continue
                    } else {
                        x = first_x_chunk;
                    }

                    if chunk_contains((x, y + 1)) {
                        y += 1;
                        overlaps_chunks.push((x, y))
                    } else {
                        break
                    }
                }
            }
        }

        overlaps_chunks
    }

    fn update_active_entities<'a>(&mut self, entities: &Entities<'a>, active_storage: &mut WriteStorage<'a, ActiveChunkMarker>, active_rect: &ActiveChunksRect) {
        active_storage.clear();

        for chunk in self.get_chunks_overlap_rect(active_rect.get_rect()).iter() {
            for ent in self.chunks.get(*chunk).unwrap().clone().iter() {
                if entities.is_alive(*ent) {
                    active_storage.insert(*ent, ActiveChunkMarker).unwrap();
                } else {
                    self.chunks.get_mut(*chunk).unwrap().retain(|e| e != ent);
                }
            }
        }
    }

    fn insert_entity_chunks<'a>(&mut self, ent: Entity, chunk_comp: &mut ChunkComponent, rect: &Rect, active_storage: &mut WriteStorage<'a, ActiveChunkMarker>, active_rect: &ActiveChunksRect) {
        let chunks = self.get_chunks_overlap_rect(rect);
        for chunk in chunks.iter() {
            self.chunks.get_mut(*chunk).unwrap().push(ent);
        }

        chunk_comp.chunks.extend(chunks);

        if active_rect.get_rect().overlaps(rect) {
            active_storage.insert(ent, ActiveChunkMarker).unwrap();
        }
    }

    fn remove_entity_chunks<'a>(&mut self, ent: Entity, chunk_comp: &mut ChunkComponent, active_storage: &mut WriteStorage<'a, ActiveChunkMarker>) {
        for chunk in chunk_comp.chunks.drain(..) {
            self.chunks.get_mut(chunk).unwrap().retain(|e| *e != ent);
        }

        active_storage.remove(ent).unwrap();
    }

    fn update_entity_chunks<'a>(&mut self, ent: Entity, chunk_comp: &mut ChunkComponent, rect: &Rect, active_storage: &mut WriteStorage<'a, ActiveChunkMarker>, active_rect: &ActiveChunksRect) {
        // TODO pas terrible
        self.remove_entity_chunks(ent, chunk_comp, active_storage);
        self.insert_entity_chunks(ent, chunk_comp, rect, active_storage, active_rect);
    }

}

impl<'a> System<'a> for ChunkSystem {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, RectComponent>,
        WriteStorage<'a, ChunkComponent>,
        WriteStorage<'a, ActiveChunkMarker>,
        WriteExpect<'a, ActiveChunksRect>
    );

    fn run(&mut self, (entities, rect, mut chunk, mut active_chunk, mut active_rect): Self::SystemData) {
        self.dirty_insert.clear();
        self.dirty_modify.clear();
        self.dirty_remove.clear();

        rect.populate_inserted(&mut self.insert_rect_reader.as_mut().unwrap(), &mut self.dirty_insert);
        rect.populate_modified(&mut self.modify_rect_reader.as_mut().unwrap(), &mut self.dirty_modify);
        rect.populate_removed(&mut self.remove_rect_reader.as_mut().unwrap(), &mut self.dirty_remove);

        for (ent, rect, _) in (&*entities, &rect, self.dirty_insert.clone()).join() {
            let mut comp = ChunkComponent::default();
            self.insert_entity_chunks(ent, &mut comp, rect.get_rect(), &mut active_chunk, &active_rect);
            chunk.insert(ent, comp).unwrap();
        }

        for (ent, chunk, rect, _) in (&*entities, &mut chunk, &rect, self.dirty_modify.clone()).join() {
            self.update_entity_chunks(ent, chunk, rect.get_rect(), &mut active_chunk, &active_rect);
        }

        for (ent, chunk, _) in (&*entities, &mut chunk, self.dirty_remove.clone()).join() {
            self.remove_entity_chunks(ent, chunk, &mut active_chunk);
        }

        if active_rect.update_dirty() {
           self.update_active_entities(&entities, &mut active_chunk, &active_rect);
        }
    }

    fn setup(&mut self, res: &mut Resources) {
        Self::SystemData::setup(res);
        let mut storage: WriteStorage<RectComponent> = SystemData::fetch(&res);

        self.insert_rect_reader = Some(storage.track_inserted());
        self.modify_rect_reader = Some(storage.track_modified());
        self.remove_rect_reader = Some(storage.track_removed());
    }
}
