pub trait BoolOps {
    fn then_some_unit(self) -> Option<()>;
    fn then_none(self) -> Option<()>;
}

impl BoolOps for bool {
    fn then_some_unit(self) -> Option<()> {
        if self { Some(()) } else { None }
    }

    fn then_none(self) -> Option<()> {
        if self { None } else { Some(()) }
    }
}
