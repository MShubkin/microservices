use std::sync::Arc;

use ahash::AHashMap;
use asez2_shared_db::{
    db_item::{AsezTimestamp, DbItemExt, DbItemPartialSelect, DbUpsert},
    result::SharedDbError,
};
use futures::{future::BoxFuture, stream::BoxStream};
use sqlx::{
    database::HasStatement, Database, Executor, PgPool, Postgres,
    Result as SqlxResult, Transaction,
};
use uuid::Uuid;

use crate::presentation::dto::{
    general::ObjectIdentifierWithStatusNote, response_request::Messages,
};

use self::historian::{Historian, HistorianMode};

mod historian;
mod impl_traits_for_records;
mod rules_lawyer;
mod status_handler;

pub use rules_lawyer::RulesLawyer;
pub use status_handler::StatusHandler;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    DbError(#[from] SharedDbError),
    #[error("Ошибка проверки правил: {0}")]
    Rules(&'static str, Messages),
    #[error("Ошибка обновления таблицы `{0}`")]
    UpdateFailed(&'static str, Messages),
    #[error(transparent)]
    StatusError(Box<dyn std::error::Error>),
}

impl Error {
    fn status_error<E: std::error::Error + 'static>(error: E) -> Self {
        Error::StatusError(Box::new(error))
    }
}

impl From<sqlx::Error> for Error {
    fn from(error: sqlx::Error) -> Self {
        Error::DbError(error.into())
    }
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, Copy)]
pub struct UpdateCtx {
    pub user_id: i32,
    pub timestamp: AsezTimestamp,
    /// Обновление на основе внешних данных,
    /// не устанавливать контекстные поля.
    pub is_external: bool,
}

/// Контекст для создания актовного инстанса `Record`.
#[derive(Debug)]
pub struct RecordCtx {
    db_pool: Arc<PgPool>,
    ctx: UpdateCtx,
    status_notes: AHashMap<Uuid, String>,
}

impl UpdateCtx {
    fn new(user_id: i32) -> Self {
        // use timestamp with precision aligned with DB
        let timestamp = AsezTimestamp::now_us();
        UpdateCtx {
            user_id,
            timestamp,
            is_external: false,
        }
    }
}

impl RecordCtx {
    pub fn new(user_id: i32, db_pool: Arc<PgPool>) -> Self {
        let ctx = UpdateCtx::new(user_id);
        RecordCtx {
            db_pool,
            ctx,
            status_notes: Default::default(),
        }
    }

    pub fn with_user_id(mut self, user_id: i32) -> Self {
        self.ctx.user_id = user_id;
        self
    }

    pub fn with_status_notes<I>(mut self, notes: I) -> Self
    where
        I: IntoIterator<Item = ObjectIdentifierWithStatusNote>,
    {
        self.status_notes = notes
            .into_iter()
            .map(|x| (x.uuid, x.status_note))
            .collect::<AHashMap<Uuid, String>>();
        self
    }

    pub fn with_timestamp(mut self, timestamp: AsezTimestamp) -> Self {
        self.ctx.timestamp = timestamp;
        self
    }

    pub fn with_external_ctx(mut self, is_external: bool) -> Self {
        self.ctx.is_external = is_external;
        self
    }

    pub async fn begin(self) -> Result<Recorder<'static>> {
        let RecordCtx {
            db_pool,
            ctx,
            status_notes,
        } = self;
        let tx = db_pool.begin().await?;
        Ok(Recorder {
            db_pool,
            tx,
            ctx,
            status_notes,
        })
    }
}

#[derive(Debug)]
pub struct Recorder<'a> {
    db_pool: Arc<PgPool>,
    tx: Transaction<'a, Postgres>,
    ctx: UpdateCtx,
    status_notes: AHashMap<Uuid, String>,
}

impl<'a> Recorder<'a> {
    pub fn user_id(&self) -> i32 {
        self.ctx.user_id
    }

    pub fn timestamp(&self) -> AsezTimestamp {
        self.ctx.timestamp
    }

    pub fn ctx(&self) -> UpdateCtx {
        self.ctx
    }

    pub fn status_notes(&self) -> &AHashMap<Uuid, String> {
        &self.status_notes
    }

    pub fn db_pool(&mut self) -> &PgPool {
        &self.db_pool
    }

    pub fn tx(&mut self) -> &mut Transaction<'a, Postgres> {
        &mut self.tx
    }

    pub fn find_status_note(&self, uuid: &Uuid) -> Option<String> {
        self.status_notes.get(uuid).cloned()
    }

    pub async fn process_update<T: ProcessUpsert>(
        &mut self,
        items: Vec<T>,
        fields_to_update: &[&'static str],
        messages: &mut Messages,
    ) -> Result<Vec<T>> {
        self.process_update_inner(
            items,
            fields_to_update,
            messages,
            HistorianMode::Update,
        )
        .await
    }
    pub async fn process_insert<T: ProcessUpsert>(
        &mut self,
        items: Vec<T>,
        messages: &mut Messages,
    ) -> Result<Vec<T>> {
        self.process_update_inner(items, T::FIELDS, messages, HistorianMode::Insert)
            .await
    }

    /// Вставляет записи, но при конфликте по первичным ключам обновляет.
    pub async fn process_upsert<T: ProcessUpsert>(
        &mut self,
        items: Vec<T>,
        fields_to_update: &[&'static str],
        messages: &mut Messages,
    ) -> Result<Vec<T>> {
        self.process_update_inner(
            items,
            fields_to_update,
            messages,
            HistorianMode::Upsert,
        )
        .await
    }

    pub async fn process_update_checked<
        T: ProcessUpsert + RulesLawyer,
        U: StatusHandler,
    >(
        &mut self,
        items: Vec<T>,
        fields_to_update: &[&'static str],
        status_handler: U,
        messages: &mut Messages,
    ) -> Result<Vec<T>> {
        self.process_update_inner_checked(
            items,
            fields_to_update,
            messages,
            status_handler,
            HistorianMode::Update,
        )
        .await
    }

    async fn process_update_inner<T: ProcessUpsert>(
        &mut self,
        items: Vec<T>,
        fields_to_update: &[&'static str],
        messages: &mut Messages,
        historian_mode: HistorianMode,
    ) -> Result<Vec<T>> {
        // empty updates are silly, but not criminal.
        if items.is_empty() {
            return Ok(vec![]);
        }
        let mut historian = Historian::new(items, fields_to_update, historian_mode);
        historian
            .pre_update(
                messages,
                &mut self.tx,
                &self.db_pool,
                &self.status_notes,
                &self.ctx,
            )
            .await?;
        historian.complete(&mut self.tx).await
    }

    pub async fn commit(self) -> Result<()> {
        self.tx.commit().await?;
        Ok(())
    }

    async fn process_update_inner_checked<
        T: ProcessUpsert + RulesLawyer,
        U: StatusHandler,
    >(
        &mut self,
        items: Vec<T>,
        fields_to_update: &[&'static str],
        messages: &mut Messages,
        status_handler: U,
        historian_mode: HistorianMode,
    ) -> Result<Vec<T>> {
        // empty updates are silly, but not criminal.
        if items.is_empty() {
            return Ok(vec![]);
        }
        let mut historian = Historian::new(items, fields_to_update, historian_mode);
        historian
            .pre_update(
                messages,
                &mut self.tx,
                &self.db_pool,
                &self.status_notes,
                &self.ctx,
            )
            .await?;
        // We return messages either way, but if we have a "serious error" we stop
        // and return them.
        historian.check_rules(messages, status_handler, &self.db_pool).await?;
        historian.complete(&mut self.tx).await
    }
}

impl<'c> Executor<'c> for &'c mut Recorder<'c> {
    type Database = Postgres;

    fn fetch_many<'e, 'q: 'e, E: 'q>(
        self,
        query: E,
    ) -> BoxStream<
        'e,
        SqlxResult<
            sqlx::Either<
                <Self::Database as Database>::QueryResult,
                <Self::Database as Database>::Row,
            >,
        >,
    >
    where
        'c: 'e,
        E: sqlx::Execute<'q, Self::Database>,
    {
        self.tx.fetch_many(query)
    }

    fn fetch_optional<'e, 'q: 'e, E: 'q>(
        self,
        query: E,
    ) -> BoxFuture<'e, SqlxResult<Option<<Self::Database as Database>::Row>>>
    where
        'c: 'e,
        E: sqlx::Execute<'q, Self::Database>,
    {
        self.tx.fetch_optional(query)
    }

    fn prepare_with<'e, 'q: 'e>(
        self,
        sql: &'q str,
        parameters: &'e [<Self::Database as Database>::TypeInfo],
    ) -> BoxFuture<'e, SqlxResult<<Self::Database as HasStatement<'q>>::Statement>>
    where
        'c: 'e,
    {
        self.tx.prepare_with(sql, parameters)
    }

    fn describe<'e, 'q: 'e>(
        self,
        sql: &'q str,
    ) -> BoxFuture<'e, SqlxResult<sqlx::Describe<Self::Database>>>
    where
        'c: 'e,
    {
        self.tx.describe(sql)
    }
}

pub trait ProcessUpsert: DbUpsert + DbItemExt + DbItemPartialSelect {
    const CTX_UPDATE_FIELDS: &'static [&'static str];
    const STATUS_FIELD: Option<&'static str> = None;

    /// Генерация UUID элемента, если он NIL.
    fn generate_uuid_if_needed(&mut self);

    /// Заполнение полей при обновлении записи исходя из переданного контекста.
    /// Контекст применяется только к тем записям, которые будут обновлены
    fn apply_update_ctx(&mut self, ctx: &UpdateCtx);

    /// Заполнение полей при создании записи исходя из переданного контекста.
    /// Контекст применяется только к тем записям, которые будут заинсерчены
    fn apply_insert_ctx(&mut self, ctx: &UpdateCtx);

    /// Проверяет, является ли поле `field_name` полем контекста обновления.
    fn is_ctx_update_field(field_name: &str) -> bool {
        Self::CTX_UPDATE_FIELDS.contains(&field_name)
    }
}
