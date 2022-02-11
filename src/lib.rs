use std::fmt::Debug;

#[derive(Clone, Copy)]
struct AllocId<const ID: usize>(usize);
impl<const ID: usize> AllocId<ID> {
    fn as_usize(&self) -> usize {
        self.0
    }
}

impl<const ID: usize> From<usize> for AllocId<ID> {
    fn from(u: usize) -> Self {
        Self(u)
    }
}

pub struct Ref<T, const ID: usize>(AllocId<ID>, &'static mut Heap<T, ID>)
where
    T: Mark<ID> + 'static;

impl<T, const ID: usize> std::fmt::Debug for Ref<T, ID>
where
    T: Debug + Mark<ID>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "&{}:{}: {:?}", ID, self.as_usize(), self.deref())
    }
}

impl<T: Mark<ID>, const ID: usize> Ref<T, ID> {
    fn as_usize(&self) -> usize {
        self.0 .0
    }
    pub fn deref(&self) -> Option<&T> {
        self.1.get(self.0)
    }
    pub fn deref_mut(&mut self) -> Option<&mut T> {
        self.1.get_mut(self.0)
    }
}

pub enum Container<T> {
    Free,
    Value(T),
}

impl<T: Debug> Debug for Container<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Free => write!(f, "[]"),
            Self::Value(value) => write!(f, "[{:?}]", value),
        }
    }
}

impl<T> Container<T> {
    fn free(&mut self) {
        *self = Self::Free;
    }

    pub fn as_value(&self) -> Option<&T> {
        if let Self::Value(v) = self {
            Some(v)
        } else {
            None
        }
    }
    pub fn as_value_mut(&mut self) -> Option<&mut T> {
        if let Self::Value(v) = self {
            Some(v)
        } else {
            None
        }
    }
}

enum Marker {
    Static,
    Mark,
    Unmark,
    Free,
}
impl Marker {
    fn mark(&mut self) {
        if let Self::Unmark = self {
            *self = Self::Mark
        }
    }
    fn unmark(&mut self) {
        if let Self::Mark = self {
            *self = Self::Unmark
        }
    }
}

pub struct Heap<T, const ID: usize> {
    values: Vec<Container<T>>,
    marks: Vec<Marker>,
    free: Vec<usize>,
}

impl<T: Debug, const ID: usize> std::fmt::Debug for Heap<T, ID> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Heap")
            .field("inner", &self.values.iter().enumerate().collect::<Vec<_>>())
            .finish()
    }
}

pub trait Mark<const ID: usize> {
    fn mark(&self);
}

impl<T: Mark<ID>, const ID: usize> Heap<T, ID> {
    pub fn new() -> Self {
        Self {
            values: Default::default(),
            marks: Default::default(),
            free: Default::default(),
        }
    }

    pub fn alloc(&'static mut self, value: T) -> Ref<T, ID> {
        if let Some(slot) = self.free.pop() {
            self.values[slot] = Container::Value(value);
            Ref(slot.into(), self)
        } else {
            self.values.push(Container::Value(value));
            self.marks.push(Marker::Unmark);
            Ref((self.values.len() - 1).into(), self)
        }
    }

    pub fn static_alloc(&'static mut self, value: T) -> Ref<T, ID> {
        if let Some(slot) = self.free.pop() {
            self.values[slot] = Container::Value(value);
            self.marks[slot] = Marker::Static;
            Ref(slot.into(), self)
        } else {
            self.values.push(Container::Value(value));
            self.marks.push(Marker::Static);
            Ref((self.values.len() - 1).into(), self)
        }
    }

    pub fn drop(&mut self, vref: Ref<T, ID>) {
        self.values[vref.as_usize()].free();
        self.marks[vref.as_usize()] = Marker::Free;
        self.free.push(vref.as_usize())
    }

    pub fn mark(&mut self, vref: &Ref<T, ID>) {
        self.marks[vref.as_usize()].mark();
        self.values[vref.as_usize()].as_value().map(T::mark);
    }

    pub fn free(&mut self) {
        for (i, marker) in self.marks.iter_mut().enumerate() {
            self.values[i].free();
            marker.unmark();
            self.free.push(i);
        }
    }

    fn get(&self, rf: AllocId<ID>) -> Option<&T> {
        let cont = self.values.get(rf.as_usize())?;
        cont.as_value()
    }

    fn get_mut(&mut self, vref: AllocId<ID>) -> Option<&mut T> {
        let cont = self.values.get_mut(vref.as_usize())?;
        cont.as_value_mut()
    }
}

impl<T: Mark<ID>, const ID: usize> Default for Heap<T, ID> {
    fn default() -> Self {
        Self::new()
    }
}
