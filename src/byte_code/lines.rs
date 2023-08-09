use crate::frame::pc::PositionCounter;
#[repr(transparent)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct LineNumber(u8);
#[repr(transparent)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Count(u8);
#[derive(Debug, Clone, Copy)]
pub(crate) struct LineCell(LineNumber, Count);

impl LineCell {
    pub(crate) fn new(line: u8) -> Self {
        Self(LineNumber(line), Count(1))
    }
}

#[repr(transparent)]
pub(crate) struct LinesBuilder(Vec<LineCell>);

impl LinesBuilder {
    pub(crate) fn new() -> Self {
        Self(Vec::new())
    }
    pub(crate) fn push(&mut self, line: u8) {
        for LineCell(LineNumber(ln), Count(c)) in &mut self.0 {
            if *ln == line {
                *c += 1;
                return;
            }
        }
        self.0.push(LineCell::new(line));
    }

    pub(crate) fn finalize(self) -> Lines {
        self.into()
    }
}

#[repr(transparent)]
#[derive(Debug)]
pub(crate) struct Lines(Vec<LineCell>);

impl Lines {
    pub(crate) fn get(&self, pos: PositionCounter) -> Option<u8> {
        let mut pos = *pos;
        for LineCell(LineNumber(ln), Count(c)) in &self.0 {
            if *c > pos as u8 {
                return Some(*ln);
            } else {
                pos -= *c as usize;
            }
        }
        None
    }
}

impl From<LinesBuilder> for Lines {
    fn from(value: LinesBuilder) -> Self {
        Self(value.0)
    }
}
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn push_test() {
        let mut builder = LinesBuilder::new();
        builder.push(1);
        builder.push(1);
        builder.push(2);
        let lines = builder.finalize();

        let mut pos = PositionCounter::default();
        assert_eq!(Some(1), lines.get(pos));
        pos = pos + 1;
        assert_eq!(Some(1), lines.get(pos));
        pos = pos + 1;
        assert_eq!(Some(2), lines.get(pos));
    }
}
