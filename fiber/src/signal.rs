use floem::reactive::RwSignal;

pub enum MaybeSignal<T>
where
    T: 'static,
{
    Static(T),
    Signal(RwSignal<T>),
}

impl<T> PartialEq for MaybeSignal<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Static(s), Self::Static(o)) => s == o,
            (Self::Static(s), Self::Signal(o)) => o.with_untracked(|v| v == s),
            (Self::Signal(s), Self::Static(o)) => s.with_untracked(|v| v == o),
            (Self::Signal(s), Self::Signal(o)) => s.with_untracked(|sv| o.with_untracked(|ov| sv == ov)),
        }
    }
}

impl<T> Default for MaybeSignal<T>
where
    T: Default,
{
    fn default() -> Self {
        Self::Static(T::default())
    }
}

#[allow(clippy::expl_impl_clone_on_copy)]
impl<T> Clone for MaybeSignal<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        match self {
            Self::Static(item) => Self::Static(item.clone()),
            Self::Signal(signal) => Self::Signal(*signal),
        }
    }
}

impl<T> Copy for MaybeSignal<T> where T: Copy {}

impl<T> MaybeSignal<T>
where
    T: Clone,
{
    pub fn get(&self) -> T {
        match self {
            MaybeSignal::Static(val) => val.clone(),
            MaybeSignal::Signal(sig) => sig.get(),
        }
    }

    pub fn get_untracked(&self) -> T {
        match self {
            MaybeSignal::Static(val) => val.clone(),
            MaybeSignal::Signal(sig) => sig.get_untracked(),
        }
    }
}

impl<T> MaybeSignal<T> {
    pub fn with(&self, f: impl Fn(&T)) {
        match self {
            MaybeSignal::Static(val) => f(val),
            MaybeSignal::Signal(sig) => sig.with(f),
        }
    }

    pub fn with_untracked(&self, f: impl Fn(&T)) {
        match self {
            MaybeSignal::Static(val) => f(val),
            MaybeSignal::Signal(sig) => sig.with_untracked(f),
        }
    }

    pub fn set(&mut self, value: T) {
        match self {
            MaybeSignal::Static(val) => *val = value,
            MaybeSignal::Signal(sig) => sig.set(value),
        }
    }

    pub fn update(&mut self, f: impl Fn(&mut T)) {
        match self {
            MaybeSignal::Static(val) => f(val),
            MaybeSignal::Signal(sig) => sig.update(f),
        }
    }
}

impl<T> From<RwSignal<T>> for MaybeSignal<T> {
    fn from(value: RwSignal<T>) -> Self {
        MaybeSignal::Signal(value)
    }
}

impl<T> From<T> for MaybeSignal<T> {
    fn from(value: T) -> Self {
        MaybeSignal::Static(value)
    }
}

impl<T> From<&T> for MaybeSignal<String>
where
    T: ToString + ?Sized,
{
    fn from(value: &T) -> Self {
        MaybeSignal::Static(value.to_string())
    }
}

impl<'a, T> From<&'a T> for MaybeSignal<Vec<u8>>
where
    T: Into<Vec<u8>>,
    Vec<u8>: std::convert::From<&'a T>,
{
    fn from(value: &'a T) -> Self {
        MaybeSignal::Static(value.into())
    }
}

impl From<MaybeSignal<String>> for RwSignal<String> {
    fn from(value: MaybeSignal<String>) -> Self {
        match value {
            MaybeSignal::Static(val) => RwSignal::new(val),
            MaybeSignal::Signal(sig) => sig,
        }
    }
}
