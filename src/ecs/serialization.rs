use std::io;
use std::fmt;

use ron;

use specs::prelude::*;
use specs::saveload::{U64MarkerAllocator, U64Marker, SerializeComponents, DeserializeComponents};
use specs::error::NoError;

use shred::DynamicSystemData;
use ecs::components_prelude::*;

#[derive(Debug)]
enum Combined {
    Ron(ron::ser::Error),
}

impl fmt::Display for Combined {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Combined::Ron(ref e) => write!(f, "{}", e),
        }
    }
}

impl From<ron::ser::Error> for Combined {
    fn from(x: ron::ser::Error) -> Self {
        Combined::Ron(x)
    }
}

impl From<NoError> for Combined {
    fn from(e: NoError) -> Self {
        match e {}
    }
}


pub fn copy_entity(entity: Entity, world: &mut World) -> Entity {
    let copy_ent = world.entities().create();

    {
        let lazy_updater = world.read_resource::<LazyUpdate>();

        macro_rules! add_copy_comp {
            ($comp:ty) => {
                match world.read_storage::<$comp>().get(entity) {
                    Some(c) => lazy_updater.insert(copy_ent.clone(), c.clone()),
                    None => {}
                };
            };
        }

        add_copy_comp!(RectComponent);
        add_copy_comp!(SpriteComponent);
        add_copy_comp!(PhysicsComponent);
        add_copy_comp!(InputComponent);
    }

    world.maintain();

    copy_ent
}

pub struct SerializeSystem<W: io::Write> {
    pub writer: W
}

impl<'a, W: io::Write> System<'a> for SerializeSystem<W> {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, RectComponent>,
        ReadStorage<'a, SpriteComponent>,
        ReadStorage<'a, InputComponent>,
        ReadStorage<'a, U64Marker>,
    );

    fn run(&mut self, (ents, rects, sprites, inputs, markers): Self::SystemData) {
        let mut ser = ron::ser::Serializer::new(Some(Default::default()), true);
        SerializeComponents::<NoError, U64Marker>::serialize(
            &(&rects, &sprites, &inputs),
            &ents,
            &markers,
            &mut ser,
        ).unwrap_or_else(|e| eprintln!("Error: {}", e));
        // TODO: Specs should return an error which combines serialization
        // and component errors.

        self.writer.write_all(ser.into_output_string().as_bytes()).expect("Impossible d'enregistrer le niveau !");
    }
}

pub struct DeserializeSystem<R: io::Read> {
    pub reader: R
}

impl<'a, R: io::Read> System<'a> for DeserializeSystem<R> {
    type SystemData = (
        Entities<'a>,
        Write<'a, U64MarkerAllocator>,
        WriteStorage<'a, RectComponent>,
        WriteStorage<'a, SpriteComponent>,
        WriteStorage<'a, InputComponent>,
        WriteStorage<'a, U64Marker>,
    );

    fn run(&mut self, (ent, mut alloc, rects, sprites, inputs, mut markers): Self::SystemData) {
        use ron::de::Deserializer;

        let mut content: Vec<u8> = vec![];

        self.reader.read_to_end(&mut content).unwrap();

        if let Ok(mut de) = Deserializer::from_bytes(&content) {
            DeserializeComponents::<Combined, _>::deserialize(
                &mut (rects, sprites, inputs),
                &ent,
                &mut markers,
                &mut alloc,
                &mut de,
            ).unwrap_or_else(|e| eprintln!("Error: {}", e));
        }
    }
}