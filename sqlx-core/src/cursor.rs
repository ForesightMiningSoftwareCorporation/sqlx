//! Contains the `Cursor` trait.

use futures_core::future::BoxFuture;

use crate::database::Database;
use crate::executor::Execute;
use crate::pool::Pool;
use crate::row::HasRow;

/// Represents a result set, which is generated by executing a query against the database.
///
/// A `Cursor` can be created by either [`Executor::fetch`] or [`Query::fetch`].
///
/// ```rust,ignore
/// let mut cursor = sqlx::query("SELECT slug, title, description FROM articles")
///     .fetch(&mut conn);
/// ```
///
/// Initially the `Cursor` is positioned before the first row. The `next` method moves the cursor
/// to the next row, and because it returns `None` when there are no more rows, it can be used
/// in a `while` loop to iterate through all returned rows.
///
/// ```rust,ignore
/// # #[derive(sqlx::FromRow)]
/// # struct Article<'a> {
/// #     slug: &'a str,
/// #     title: &'a str,
/// #     description: &'a str,
/// # }
/// #
/// // For each row in the result set ..
/// while let Some(row) = cursor.next().await? {
///     // .. decode a domain type from the row
///     let obj = Article::from_row(row)?;
/// }
/// ```
///
/// This trait is sealed and cannot be implemented for types outside of SQLx.
///
/// [`Executor::fetch`]: crate::executor::Executor::fetch
/// [`Query::fetch`]: crate::query::Query::fetch
pub trait Cursor<'c, 'q>
where
    Self: Send + Unpin + private::Sealed,
{
    /// The `Database` this `Cursor` is implemented for.
    type Database: Database;

    #[doc(hidden)]
    fn from_pool<E>(pool: &Pool<<Self::Database as Database>::Connection>, query: E) -> Self
    where
        Self: Sized,
        E: Execute<'q, Self::Database>;

    #[doc(hidden)]
    fn from_connection<E>(
        connection: &'c mut <Self::Database as Database>::Connection,
        query: E,
    ) -> Self
    where
        Self: Sized,
        E: Execute<'q, Self::Database>;

    /// Creates a future that attempts to resolve the next row in the cursor.
    fn next<'cur>(
        &'cur mut self,
    ) -> BoxFuture<'cur, crate::Result<Self::Database, Option<<Self::Database as HasRow<'cur>>::Row>>>;
}

// Prevent users from implementing the `Row` trait.
pub(crate) mod private {
    pub trait Sealed {}
}

/// Associate [`Database`] with a [`Cursor`] of a generic lifetime.
///
/// ---
///
/// The upcoming Rust feature, [Generic Associated Types], should obviate
/// the need for this trait.
///
/// [Generic Associated Types]: https://www.google.com/search?q=generic+associated+types+rust&oq=generic+associated+types+rust&aqs=chrome..69i57j0l5.3327j0j7&sourceid=chrome&ie=UTF-8
pub trait HasCursor<'c, 'q> {
    type Database: Database;

    /// The concrete `Cursor` implementation for this database.
    type Cursor: Cursor<'c, 'q, Database = Self::Database>;
}
