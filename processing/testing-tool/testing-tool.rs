use std::error::Error;
use std::path::PathBuf;
use std::result::Result;

use ahash::AHashMap;
use itertools::Itertools;
use sqlx::postgres::PgPoolOptions;
use sqlx::{Executor, PgPool, Postgres, Transaction};

const COMMIT: &str = "--commit";
const HELP: &str = "--help";
const PATH: &str = "--path";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let a = std::env::args().collect::<Vec<_>>();
    let args = a
        .iter()
        .map(|x| x.split_once('=').unwrap_or((x, "")))
        .collect::<AHashMap<_, _>>();

    if args.contains_key(HELP) {
        println!("{BLURB}");
        return Ok(());
    }
    // Файлы берутся из локальный директории
    // В принципе можно и усложнить.
    let dir = match args.get(PATH) {
        Some(path) => PathBuf::from(path),
        None => {
            let dir = env!("CARGO_MANIFEST_DIR").to_string();
            PathBuf::from(dir).join("testing-tool").join("sql")
        }
    };

    let pg = env_setup::PostgresCfg::from_env()?;
    let db = pg.get_connection_string();

    let conn = get_conn(&db).await?;
    let mut transaction = conn.begin().await?;

    match process_dir(dir, &mut transaction, args.contains_key(COMMIT)).await {
        Ok(()) => println!("TEST DATA IS OK."),
        Err(e) => {
            println!("TEST DATA IS BAD: {e}.");
            return Err(e);
        }
    }

    if args.contains_key(COMMIT) {
        println!("COMMITTING CHANGES TO TEST DATA ON: \"{db}\".");
        transaction.commit().await?;
    } else {
        println!("ROLLING BACK CHANGES TO TEST DATA ON: \"{db}\".");
        transaction.rollback().await?;
    }

    Ok(())
}

async fn get_conn(db: &str) -> sqlx::Result<PgPool> {
    println!("CONNECTING TO DB: {db}");
    PgPoolOptions::new().connect(db).await
}

/// Провести проверку SQL на директории (обычно локальной)
async fn process_dir(
    dir: PathBuf,
    transaction: &mut Transaction<'_, Postgres>,
    is_commit: bool,
) -> Result<(), Box<dyn Error>> {
    let entries = std::fs::read_dir(dir)?;
    let sorted_entries =
        entries.flat_map(|x| x.ok()).sorted_by_key(|dir| dir.path());
    for entry in sorted_entries {
        let path = entry.path();
        // We skip non-files (directories and symlinks).
        if !path.is_file() {
            continue;
        }
        let sql_script = std::fs::read_to_string(&path)?;
        let name = path.file_name().unwrap();
        println!("Testing file: {name:?}...");

        for sql in sql_script.split_inclusive(';') {
            if is_commit {
                println!("Executing query: \"{}\"", sql);
                sqlx::query(sql).execute(&mut *transaction).await?;
            } else {
                // можно не "выполнять", достаточно "prepare" (+ debug print, чтобы видеть где конкретно упали)
                println!("Preparing query: \"{}\"", sql);
                transaction.prepare(sql).await?;
            }
        }
        println!("...FILE IS OK: {name:?}");
    }
    Ok(())
}

const BLURB: &str = "
sql-testing-tool
------------

Маленькая программа для того чтобы проверить правильно-ли заливаются данные для тестировщиков.

Программа берёт SQL файлы из местной директории sql, заливает их как часть транзакции, но коммит не делает. Если вылетает ошибка, значит процесс не прошел.

.env файл должен быть правильный!

Аргументы:

--help      Вывести на экран эти инструкции.

--commit    Влить, при успешной проверки, данные в БД.

--path      Использовать иную директорию как источник sql. Формат: --path=/some/other/dir/

";
