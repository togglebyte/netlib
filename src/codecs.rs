pub trait Codec<T> {
    fn decode(&mut self) -> T;

    // fn encode(&mut self) -> ?
}
