pub trait Callback<T>
where
    T: Callback<T>,
{
    fn new() -> T;

    fn created(&self);

    fn resized(&self, width: u32, height: u32);

    fn destroyed(&self);
}
