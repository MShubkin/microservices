use std::{
    future::Future,
    path::{Path, PathBuf},
    sync::Arc,
};

use sqlx::PgPool;

use crate::db::prepare_for_test_fixture_files;

/// The enum shows where to get the base migrations.
pub enum BaseMigPath<'a> {
    MigrationsHome,
    New,
    Other(&'a str),
    None,
}
impl<'a> BaseMigPath<'a> {
    /// Runs DB test with database prepared according to standard layout
    ///
    /// ```ignore
    /// crate/
    ///   migrations
    /// ```
    pub async fn run_test_with_migrations<F, FutFn>(
        self,
        migs_dir: impl AsRef<Path> + 'static,
        extra_migs_files: &'static [&'static str],
        drop_tables: &'static [&'static str],
        run: FutFn,
    ) where
        F: Future<Output = ()>,
        FutFn: FnOnce(Arc<PgPool>) -> F + 'static,
    {
        let root_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());

        self.run_test_with_migrations_with_root(
            root_dir,
            migs_dir,
            extra_migs_files,
            drop_tables,
            run,
        )
        .await
    }

    /// Runs DB test with database prepared according to standard layout
    ///
    /// ```ignore
    /// crate/
    ///   migrations/
    ///     up/
    /// ```
    pub async fn run_test_with_migrations_with_root<F, FutFn>(
        self,
        root_dir: impl AsRef<Path> + 'static,
        migs_dir: impl AsRef<Path> + 'static,
        extra_migs_files: &'static [&'static str],
        _drop_tables: &'static [&'static str],
        run: FutFn,
    ) where
        F: Future<Output = ()>,
        FutFn: FnOnce(Arc<PgPool>) -> F + 'static,
    {
        let mig_path = match &self {
            Self::MigrationsHome => Some("migrations"),
            Self::New => Some("migrations/new"),
            Self::Other(a) => Some(*a),
            Self::None => None,
        };
        let migrations = mig_path.map(|p| root_dir.as_ref().join(p));

        let (pool, _id) =
            prepare_for_test_fixture_files(migrations, migs_dir, extra_migs_files)
                .await
                .expect("db is prepared");

        let _ = run(pool.clone()).await;
    }
}
