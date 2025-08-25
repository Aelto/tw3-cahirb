use crate::decompiler::CodeEmitter;

pub trait WithCodeEmitting {
    fn emit_code(&self, f: &mut CodeEmitter);
}

impl WithCodeEmitting for &String {
    fn emit_code(&self, f: &mut CodeEmitter) {
        f.append(self);
    }
}

impl WithCodeEmitting for &str {
    fn emit_code(&self, f: &mut CodeEmitter) {
        f.append(self);
    }
}

impl<T> WithCodeEmitting for Box<T>
where
    T: WithCodeEmitting,
{
    fn emit_code(&self, f: &mut CodeEmitter) {
        let inner: &T = self.as_ref();
        inner.emit_code(f);
    }
}

/// For quickly emiting 3 items, most useful for doing
/// `("prefix", item, "suffix").emit_code(f)`
impl<A, B, C> WithCodeEmitting for (&A, &B, &C)
where
    A: WithCodeEmitting + ?Sized,
    B: WithCodeEmitting + ?Sized,
    C: WithCodeEmitting + ?Sized,
{
    fn emit_code(&self, f: &mut CodeEmitter) {
        self.0.emit_code(f);
        self.1.emit_code(f);
        self.2.emit_code(f);
    }
}

impl<T> WithCodeEmitting for Option<T>
where
    T: WithCodeEmitting,
{
    fn emit_code(&self, f: &mut CodeEmitter) {
        if let Some(item) = self {
            item.emit_code(f);
        }
    }
}

/// For quickly emiting options, though it may make code less intuitive for
/// the readers.
impl<T, SUFFIX> WithCodeEmitting for (&Option<T>, SUFFIX)
where
    T: WithCodeEmitting,
    SUFFIX: WithCodeEmitting,
{
    fn emit_code(&self, f: &mut CodeEmitter) {
        if let Some(item) = &self.0 {
            item.emit_code(f);
            self.1.emit_code(f);
        }
    }
}

impl<T> WithCodeEmitting for Vec<T>
where
    T: WithCodeEmitting,
{
    fn emit_code(&self, f: &mut CodeEmitter) {
        for item in self {
            item.emit_code(f);
        }
    }
}

/// For looping over code emitters while also adding a &str between each
/// item. The &str is not added after the last item
impl<T> WithCodeEmitting for (&Vec<T>, &str)
where
    T: WithCodeEmitting,
{
    fn emit_code(&self, f: &mut CodeEmitter) {
        if let Some(last) = self.0.last() {
            let before_last = &self.0[..self.0.len() - 1];

            for item in before_last {
                item.emit_code(f);
                self.1.emit_code(f);
            }

            last.emit_code(f);
        }
    }
}
