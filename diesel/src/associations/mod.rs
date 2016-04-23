use std::hash::Hash;

use backend::Backend;
use prelude::*;
use query_builder::{AsQuery, QueryFragment, QueryId};
use types::HasSqlType;

pub trait Identifiable {
    type Id: Hash + Eq + Copy;

    fn id(&self) -> Self::Id;
}

pub trait BelongsTo<Parent: Identifiable> {
    fn foreign_key(&self) -> Parent::Id;
}

pub trait GroupedBy<Parent>: IntoIterator + Sized {
    fn grouped_by(self, parents: &[Parent]) -> Vec<Vec<Self::Item>>;
}

impl<Parent, Child> GroupedBy<Parent> for Vec<Child> where
    Child: BelongsTo<Parent>,
    Parent: Identifiable,
{
    fn grouped_by(self, parents: &[Parent]) -> Vec<Vec<Child>> {
        use std::collections::HashMap;

        let id_indices: HashMap<_, _> = parents.iter().enumerate().map(|(i, u)| (u.id(), i)).collect();
        let mut result = parents.iter().map(|_| Vec::new()).collect::<Vec<_>>();
        for child in self {
            let index = id_indices[&child.foreign_key()];
            result[index].push(child);
        }
        result
    }
}

pub trait LoadAssociated<Conn> {
    type Parent: Identifiable;
    type Backend: Backend;

    fn load_associated<Child>(&self, connection: &Conn) -> QueryResult<Vec<Vec<Child>>> where
        Child: BelongsTo<Self::Parent> + BelongingToDsl<Self>,
        Child: Queryable<
            <<Child as BelongingToDsl<Self>>::Output as AsQuery>::SqlType,
            Self::Backend,
        >,
        <Child::Output as AsQuery>::Query: QueryFragment<Self::Backend> + QueryId,
        Self::Backend: HasSqlType<<<Child as BelongingToDsl<Self>>::Output as AsQuery>::SqlType>;
}

impl<Parent, Conn> LoadAssociated<Conn> for Vec<Parent> where
    Parent: Identifiable,
    Conn: Connection,
{
    type Parent = Parent;
    type Backend = Conn::Backend;

    fn load_associated<Child>(&self, connection: &Conn)
        -> QueryResult<Vec<Vec<Child>>> where
            Child: BelongsTo<Self::Parent> + BelongingToDsl<Self>,
            Child: Queryable<
                <Child as BelongingToDsl<Self>>::SqlType,
                Self::Backend,
            >,
            <Child::Output as AsQuery>::Query: QueryFragment<Self::Backend> + QueryId,
            Self::Backend: HasSqlType<Child::SqlType>,
    {
        <_ as LoadDsl<_>>::load(Child::belonging_to(&self), connection).map(|children| {
            children.grouped_by(&self)
        })
    }
}
