pub struct Pool<T> {
    elements: Vec<Option<T>>,
    free_list: Vec<usize>,
}

impl<T> Pool<T> {
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        let value = self.elements.get_mut(index);
        match value {
            None => None,
            Some(val) => val.as_mut(),
        }
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        let value = self.elements.get(index);
        match value {
            None => None,
            Some(val) => val.as_ref(),
        }
    }

    pub fn allocate(&mut self, value: T) -> usize {
        if self.free_list.len() == 0 {
            self.elements.push(Some(value));
            self.elements.len() - 1
        } else {
            self.free_list.pop().unwrap()
        }
    }

    pub fn delete(&mut self, index: usize) {
        self.free_list.push(index);
        self.elements[index] = None;
    }
}
