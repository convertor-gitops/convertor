pub trait Boxed<T, E> {
    fn boxed(self) -> Result<Box<T>, E>;
    fn boxed_err(self) -> Result<T, Box<E>>;
    fn boxed_map<U, F>(self, op: F) -> Result<U, E>
    where
        F: FnOnce(Box<T>) -> U;
    fn boxed_map_err<F, O>(self, op: O) -> Result<T, F>
    where
        O: FnOnce(Box<E>) -> F;
}

impl<T, E> Boxed<T, E> for Result<T, E> {
    fn boxed(self) -> Result<Box<T>, E> {
        self.map(Box::new)
    }

    fn boxed_err(self) -> Result<T, Box<E>> {
        self.map_err(Box::new)
    }

    fn boxed_map<U, F>(self, op: F) -> Result<U, E>
    where
        F: FnOnce(Box<T>) -> U,
    {
        self.map(|v| op(Box::new(v)))
    }

    fn boxed_map_err<F, O>(self, op: O) -> Result<T, F>
    where
        O: FnOnce(Box<E>) -> F,
    {
        self.map_err(|e| op(Box::new(e)))
    }
}
