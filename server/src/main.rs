#![feature(proc_macro_hygiene, decl_macro, slice_ptr_len)]

#[macro_use]
extern crate rocket;
use rocket::response::Stream;
use std::io::Cursor;

lazy_static::lazy_static! {
    static ref PEOPLE: Box<[shared::Person]> = {
        use rand::prelude::*;
        let mut rng = thread_rng();
        (0..1_000_000)
            .map(|_i| shared::Person {
                id: rng.gen(),
                age: rng.gen(),
            })
            .collect::<Box<[shared::Person]>>()
    };
}

#[get("/people.json")]
fn people_json() -> Option<Stream<Cursor<Vec<u8>>>> {
    serde_json::to_vec(&*PEOPLE)
        .map(|data| Stream::from(Cursor::new(data)))
        .ok()
}

#[get("/people.bin")]
fn people_bin() -> Option<Stream<Cursor<&'static [u8]>>> {
    let ptr = PEOPLE.as_ptr() as *const u8;
    let contents = unsafe {
        std::slice::from_raw_parts(ptr, PEOPLE.len() * std::mem::size_of::<shared::Person>())
    };
    Some(Stream::from(Cursor::new(contents)))
}

#[get("/people.bincode")]
fn people_bincode() -> Option<Stream<Cursor<Vec<u8>>>> {
    bincode::serialize(&*PEOPLE)
        .map(|data| Stream::from(Cursor::new(data)))
        .ok()
}

fn rocket() -> rocket::Rocket {
    let static_files =
        rocket_contrib::serve::StaticFiles::from(concat!(env!("CARGO_MANIFEST_DIR"), "/static"));
    rocket::ignite()
        .mount("/", static_files)
        .mount("/data", routes![people_json, people_bin, people_bincode])
}

fn main() {
    rocket().launch();
}
