#![feature(proc_macro_hygiene, decl_macro, slice_ptr_len)]

#[macro_use]
extern crate rocket;
use rocket::response::Stream;

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
fn people_json() -> Option<Stream<std::fs::File>> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("static")
        .join("data.json");
    std::fs::File::open(path)
        .map(|file| Stream::from(file))
        .ok()
}

#[get("/people.bin")]
fn people_bin() -> Option<Stream<std::fs::File>> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("static")
        .join("data.bin");
    std::fs::File::open(path)
        .map(|file| Stream::from(file))
        .ok()
}

#[get("/people.bincode")]
fn people_bincode() -> Option<Stream<std::fs::File>> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("static")
        .join("data.bincode");
    std::fs::File::open(path)
        .map(|file| Stream::from(file))
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

#[cfg(test)]
mod tests {
    #[test]
    fn gen_data() {
        use rand::prelude::*;
        let mut rng = thread_rng();
        let data = (0..1_000_000)
            .map(|_i| shared::Person {
                id: rng.gen(),
                age: rng.gen(),
            })
            .collect::<Box<[shared::Person]>>();
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("static");

        let contents = serde_json::to_string(&data).expect("serialization failed");
        std::fs::write(path.join("data.json"), contents).expect("file write failed");

        let contents = bincode::serialize(&data).expect("serialization failed");
        std::fs::write(path.join("data.bincode"), contents).expect("file write failed");

        let ptr = data.as_ptr() as *const u8;
        let contents = unsafe {
            std::slice::from_raw_parts(ptr, data.len() * std::mem::size_of::<shared::Person>())
        };
        std::fs::write(path.join("data.bin"), contents).expect("file write failed");
    }
}
