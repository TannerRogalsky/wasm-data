use futures::task::{Context, Poll};
use futures::StreamExt;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub async fn json_parse(data: Box<[u8]>) -> u8 {
    let data: Box<[shared::Person]> = serde_json::from_slice(&data).unwrap();
    average(futures::stream::iter(data.iter())).await
}

#[wasm_bindgen]
pub async fn json_wasm_parse(data: JsValue) -> u8 {
    let data: Box<[shared::Person]> = serde_wasm_bindgen::from_value(data).unwrap();
    average(futures::stream::iter(data.iter())).await
}

#[wasm_bindgen]
pub async fn raw(data: Box<[u8]>) -> u8 {
    let data = unsafe {
        let ptr = data.as_ptr() as *const shared::Person;
        std::slice::from_raw_parts(ptr, data.len() / std::mem::size_of::<shared::Person>())
    };
    average(futures::stream::iter(data)).await
}

#[wasm_bindgen]
pub async fn stream(stream: web_sys::ReadableStream) -> u8 {
    let stream = wasm_streams::ReadableStream::from_raw(
        wasm_bindgen::JsCast::dyn_into(stream).unwrap_throw(),
    );
    average(ReinterpretStream::new(stream)).await
}

#[wasm_bindgen]
pub async fn bincode_parse(data: Box<[u8]>) -> u8 {
    let data: Box<[shared::Person]> = bincode::deserialize(&data).unwrap();
    average(futures::stream::iter(data.iter())).await
}

async fn average<'a, S>(mut data: S) -> u8
where
    S: futures::Stream<Item = &'a shared::Person> + std::marker::Unpin + 'a,
{
    let mut count = 0usize;
    let mut acc = 0usize;
    while let Some(person) = data.next().await {
        count += 1;
        acc += person.age as usize;
    }
    (acc / count) as u8
}

struct ReinterpretStream<'a> {
    inner: wasm_streams::readable::IntoStream<'static>,
    buffer: Vec<u8>,
    offset: usize,
    lifetime_marker: std::marker::PhantomData<&'a shared::Person>,
}

impl<'a> ReinterpretStream<'a> {
    fn new(stream: wasm_streams::ReadableStream) -> Self {
        Self {
            inner: stream.into_stream(),
            buffer: Default::default(),
            offset: 0,
            lifetime_marker: Default::default(),
        }
    }
}

impl<'a> futures::Stream for ReinterpretStream<'a> {
    type Item = &'a shared::Person;

    fn poll_next(mut self: std::pin::Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        if !futures::stream::FusedStream::is_terminated(&self.inner) {
            if let Poll::Ready(Some(Ok(chunk))) = self.inner.poll_next_unpin(cx) {
                let chunk: js_sys::Uint8Array = wasm_bindgen::JsCast::dyn_into(chunk).unwrap();
                let size = chunk.byte_length() as usize;

                let new_data_size = self.offset + size;
                self.buffer.resize(new_data_size, 0);
                let buffer_range = self.offset..new_data_size;
                chunk.copy_to(&mut self.buffer[buffer_range]);
            }
        }

        const STRIDE: usize = std::mem::size_of::<shared::Person>();
        if self.buffer.len() > self.offset + STRIDE {
            let bytes = &self.buffer[self.offset..(self.offset + STRIDE)];
            let person = unsafe { (bytes.as_ptr() as *const shared::Person).as_ref().unwrap() };
            self.offset += STRIDE;
            Poll::Ready(Some(&person))
        } else if futures::stream::FusedStream::is_terminated(&self.inner) {
            Poll::Ready(None)
        } else {
            Poll::Pending
        }
    }
}

#[wasm_bindgen(start)]
pub fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
}

#[cfg(test)]
mod tests {
    #[test]
    fn average_test() {
        let data = vec![
            shared::Person { id: 0, age: 10 },
            shared::Person { id: 1, age: 20 },
        ];
        let r = super::average(futures::stream::iter(data.iter()));
        let r = futures::executor::block_on(r);
        assert_eq!(r, 15);
    }
}
