pub struct Cache<T> {
    pub value: T,
    pub dirty: bool
}

impl<T> Cache<T> {
    pub fn new(value: T) -> Cache<T> {
        return Cache::<T> {
            value: value,
            dirty: true,
        }
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn clear(&mut self) {
        self.dirty = false;
    }
}

pub struct VecCache<T> {
    pub values: Vec<T>,
    pub dirty: bool,
}

impl<T> VecCache<T> {
    pub fn new(values: Vec<T>) -> VecCache<T> {
        return VecCache::<T> {
            values: values,
            dirty: true,
        }
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn clear(&mut self) {
        self.dirty = false;
    }
}
