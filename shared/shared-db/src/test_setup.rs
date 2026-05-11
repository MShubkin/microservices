use super::*;

/// This function must be inserted into every DB test as follows:
/// ```
/// use asez2_shared_db::test_setup::run_db_test;
/// fn test_this() {
///     run_db_test("my_table", "create table(..)", None, |_db_pool| async {
///         // Test stuff here.    
///     });
/// }
/// ```
/// The function also creates the table as defined and if an SQL string with data is
/// provided initiates it with given data. It should be noted that tests with the same
/// table name should not be run simultaneously as the transaction system on postgres is
/// not watertight.
pub async fn run_db_test<'a, F, FutFn>(
    table_name: &str,
    table_defn: &str,
    initial_data: Option<&str>,
    run: FutFn,
) where
    F: futures::Future<Output = ()> + 'a + Send,
    FutFn: FnOnce(sqlx::Transaction<'a, sqlx::Postgres>) -> F + Send + 'static,
{
    let opt = PgDbOptions::from_env()
        .expect("Не удается получить переменные среды для `PgDbOptions`");

    let pool = opt.get_pool().await.expect("Could not get pool.");

    let drop_stmt = format!("drop table if exists {};", table_name);
    let create = format!("create table {}{};", table_name, table_defn);
    println!("{}", drop_stmt);
    println!("{}", create);
    println!("{:#?}", initial_data);

    let table_name = table_name.to_string();
    let initial_data = initial_data.map(|x| x.to_owned());

    if let Err(e) = tokio::task::spawn(async move {
        let mut t = pool.begin().await.expect("Not possible.");
        sqlx::query(&drop_stmt).execute(&mut t).await.expect("Could not drop");
        sqlx::query(&create).execute(&mut t).await.expect("Could not recreate");

        if let Some(initial_data) = initial_data {
            let insert = format!("insert into {}{};", table_name, initial_data);
            sqlx::query(&insert).execute(&mut t).await.expect("Could not insert");
        }
        let x = run(t).await;
        pool.close().await;
        drop(pool);
        x
    })
    .await
    {
        panic!("Test failed:\n{}", e);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[should_panic]
    async fn test_runner_panic() {
        run_db_test(
            "mongoose",
            "(id serial NOT NULL, name text, alive boolean NOT NULL DEFAULT true)",
            None,
            |_pool| async { panic!("Should panic.") },
        )
        .await;
    }

    #[tokio::test]
    async fn test_runner() {
        run_db_test(
            "mongoose",
            "(id serial NOT NULL, name text, alive boolean NOT NULL DEFAULT true)",
            None,
            |_pool| async { assert!(Some(1).is_some()) },
        )
        .await;
    }
}
