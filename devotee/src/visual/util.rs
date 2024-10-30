#[derive(Clone, Copy, Debug)]
pub struct AngleIterator<'a, T> {
    index: usize,
    vertices: &'a [T],
}

impl<'a, T> AngleIterator<'a, T> {
    pub fn new(vertices: &'a [T]) -> Self {
        let index = 0;
        Self { index, vertices }
    }
}

impl<'a, T> Iterator for AngleIterator<'a, T> {
    type Item = (&'a T, &'a T, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        let index = self.index;
        self.index += 1;
        match index {
            i if i < self.vertices.len() - 2 => Some((
                &self.vertices[i],
                &self.vertices[i + 1],
                &self.vertices[i + 2],
            )),
            i if i == self.vertices.len() - 2 => {
                Some((&self.vertices[i], &self.vertices[i + 1], &self.vertices[0]))
            }
            i if i == self.vertices.len() - 1 => {
                Some((&self.vertices[i], &self.vertices[0], &self.vertices[1]))
            }
            _ => None,
        }
    }
}
