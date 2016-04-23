use query_builder::AsQuery;

pub trait BelongingToDsl<T: ?Sized> {
    type SqlType;
    type Output: AsQuery<SqlType=Self::SqlType>;

    fn belonging_to(other: &T) -> Self::Output;
}
