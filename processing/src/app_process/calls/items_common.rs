//! Общий код для обновления позиций Протокола и Повестки

use ahash::AHashMap;
use asez2_shared_db::db_item::AsezTimestamp;
use shared_essential::domain::AgendaProtocolItem;
use uuid::Uuid;

#[derive(Debug, thiserror::Error)]
pub enum ItemsError {
    #[error("Некорректные входные данные. Позиция {0} не включена в запрос на обновление")]
    MissingExisting(Uuid),
    #[error("Ошибка входных данных. Позиция {0} с признаком is_registered_by_d647 = {1}
             включена в запрос на обновление с признаком is_registered_by_d647 = {2}")]
    WrongList(i64, bool, bool),
    #[error(r#"выполнить сохранение невозможно. ППЗ/ДС {0} была одновременно добавлена на вкладках Список ППЗ/ДС и Реестр ППЗ/ДС"#)]
    DupItem(Uuid),
    #[error(r#"выполнить сохранение невозможно. ППЗ/ДС {0} была одновременно добавлена на вкладках Список ППЗ/ДС и Реестр ППЗ/ДС"#)]
    DupItemId(i64),
    #[error("Ошибка входных данных. Позиция {0} после обновления ссылается на ППЗ/ДС {1}, должна ссылаться на {2}")]
    WrongSource(i64, Uuid, Uuid),
}

/// Набор данных, используемых для преобразования входных позиций ППЗ в адаптеры БД.
///
/// Элементы позиций должны обрабатываться в строгом порядке (как присылаются с FE):
/// - Элементы списка (`is_registered_by_d647 == false`)
/// - Удаленные элементы списка
/// - Элементы реестра (`is_registered_by_d647 == true`)
/// - Удаленные элементы реестра
///
/// Неудаленные элементы нумеруются с 1, причем нумерация отдельная для списка и реестра.
/// Удаленным элементам присваивается номер `0`.
///
/// Для вновь добавленных позиций uuid не указывается. По source_uuid надо найти удаленную
/// позицию, и использовать ее uuid, иначе создать новый.
///
/// См. `[PrepareItemContext::number]`.
pub(crate) struct PrepareItemContext<T> {
    /// Повестка/протокол, владеющий позицией.
    container_uuid: Uuid,
    /// Признак реестра для предыдущей позиции, для сброса нумерации.
    prev_is_d647: bool,
    /// Следующий номер для позиции.
    number: i64,
    /// Позиции повестки/протокола до вызова.
    existing_items: Vec<T>,
    /// Отображение uuid предыдущих позиций в индекс в existing_items.
    by_uuid: AHashMap<Uuid, usize>,
    /// Отображение uuid ППЗ/ДС и признака реестра предыдущих позиций в индекс в existing_items.
    by_source: AHashMap<(Uuid, bool), usize>,
    /// Идентификатор пользователя, выполняющего обновление.
    user_id: i32,
    /// Временная метка обновления.
    timestamp: AsezTimestamp,
}

impl<T: AgendaProtocolItem> PrepareItemContext<T> {
    pub(crate) fn new(
        container_uuid: Uuid,
        existing_items: Vec<T>,
        user_id: i32,
    ) -> PrepareItemContext<T> {
        let (by_uuid, by_source) = existing_items
            .iter()
            .enumerate()
            .map(|(i, item)| {
                (
                    (item.uuid(), i),
                    ((item.source_uuid(), item.is_registered_by_d647()), i),
                )
            })
            .unzip();

        PrepareItemContext {
            container_uuid,
            prev_is_d647: false,
            number: 0,
            existing_items,
            by_source,
            by_uuid,
            user_id,
            timestamp: AsezTimestamp::now(),
        }
    }

    pub(crate) fn container_uuid(&self) -> Uuid {
        self.container_uuid
    }

    pub(crate) fn user_id(&self) -> i32 {
        self.user_id
    }

    pub(crate) fn timestamp(&self) -> AsezTimestamp {
        self.timestamp
    }

    fn find_by_uuid(&self, uuid: Uuid) -> Option<&T> {
        self.by_uuid.get(&uuid).map(|i| &self.existing_items[*i])
    }

    fn find_by_source(&self, uuid: Uuid, d647: bool) -> Option<&T> {
        self.by_source.get(&(uuid, d647)).map(|i| &self.existing_items[*i])
    }

    pub(crate) fn find_existing_item(
        &self,
        uuid: Option<Uuid>,
        source_uuid: Uuid,
        d647: bool,
    ) -> Option<&T> {
        if let Some(uuid) = uuid {
            self.find_by_uuid(uuid)
        } else {
            self.find_by_source(source_uuid, d647)
        }
    }

    /// Поиск новых элементов и элементов, которые восстанавливаются с is_removed=true на is_removed=false (но не исключаются)
    pub(crate) fn new_items<'a>(&self, to_upsert_items: &'a [T]) -> Vec<&'a T> {
        to_upsert_items
            .iter()
            .filter(|to_upsert_item| {
                if let Some(existing_item) =
                    self.find_by_uuid(to_upsert_item.uuid())
                {
                    existing_item.is_removed()
                        && !to_upsert_item.is_removed()
                        && !to_upsert_item.is_excluded()
                } else {
                    true
                }
            })
            .collect()
    }

    /// Поиск позиций Протокола/Повестки, которые были добавлены обратно
    pub(crate) fn included_items<'a>(
        &self,
        to_upsert_items: &'a [T],
    ) -> Vec<&'a T> {
        to_upsert_items
            .iter()
            .filter(|to_upsert_item| {
                if let Some(existing_item) =
                    self.find_by_uuid(to_upsert_item.uuid())
                {
                    existing_item.is_excluded() && !to_upsert_item.is_excluded()
                } else {
                    false
                }
            })
            .collect()
    }

    /// Поиск удаляемых, которые переводятся с is_removed=false на is_removed=true
    pub(crate) fn removed_items<'a>(&self, to_upsert_items: &'a [T]) -> Vec<&'a T> {
        to_upsert_items
            .iter()
            .filter(|to_upsert_item| {
                if let Some(existing_item) =
                    self.find_by_uuid(to_upsert_item.uuid())
                {
                    !existing_item.is_removed() && to_upsert_item.is_removed()
                } else {
                    false
                }
            })
            .collect()
    }

    /// Принимает общий массив элементов, которые пользователь хочет обновить,
    /// и возвращает только те элементы, которые действительно можно обновить
    /// или добавить
    ///
    /// has_changes_fn принимает (новый элемент, старый элемент)
    pub(crate) fn upsertable_items<I, F>(
        &self,
        to_upsert_items: I,
        has_changes_fn: F,
    ) -> Vec<T>
    where
        I: IntoIterator<Item = T>,
        F: Fn(&T, &T) -> bool,
    {
        to_upsert_items
            .into_iter()
            .filter(|item| {
                if let Some(old_item) = self.find_by_uuid(item.uuid()) {
                    has_changes_fn(item, old_item)
                } else {
                    true
                }
            })
            .collect()
    }

    /// Проверка, что все позиции в БД, не удаленные из протокола/повестки,
    /// присутствуют в элементах запроса и данные по старым элементам были переданы
    /// правильно
    pub(crate) fn validate_compatability(
        &self,
        to_upsert_items: &[T],
        plan_ids: AHashMap<&Uuid, i64>,
    ) -> Result<(), ItemsError> {
        if to_upsert_items.is_empty() {
            return Ok(());
        }

        let mut already_included = AHashMap::with_capacity(to_upsert_items.len());

        for to_upsert_item in to_upsert_items {
            // Если обновляется старый элемент, то надо проверить что элемент ссылается на тот же ППЗ/ДС
            // и не был перемещен в д647 или обратно
            if let Some(existing_item) = self.find_by_uuid(to_upsert_item.uuid()) {
                if existing_item.source_uuid() != to_upsert_item.source_uuid() {
                    return Err(ItemsError::WrongSource(
                        to_upsert_item.number(),
                        to_upsert_item.uuid(),
                        existing_item.source_uuid(),
                    ));
                }

                if existing_item.is_registered_by_d647()
                    != to_upsert_item.is_registered_by_d647()
                {
                    return Err(ItemsError::WrongList(
                        to_upsert_item.number(),
                        existing_item.is_registered_by_d647(),
                        to_upsert_item.is_registered_by_d647(),
                    ));
                }
            }

            let maybe_already_registered = already_included.insert(
                (
                    to_upsert_item.source_uuid(),
                    to_upsert_item.is_registered_by_d647(),
                ),
                to_upsert_item,
            );

            // В одном списке может быть только одна запись
            if let Some(already_registered) = maybe_already_registered {
                return Err(ItemsError::DupItem(already_registered.source_uuid()));
            }

            if let Some(duplicate) = already_included.get(&(
                to_upsert_item.source_uuid(),
                !to_upsert_item.is_registered_by_d647(),
            )) {
                // Если записи в списке и в реестре валидны (is_removed = false && is_excluded = false), то считаем за дупликат
                if !to_upsert_item.is_removed()
                    && !to_upsert_item.is_excluded()
                    && !duplicate.is_removed()
                    && !duplicate.is_excluded()
                {
                    if let Some(plan_id) = plan_ids.get(&duplicate.source_uuid()) {
                        return Err(ItemsError::DupItemId(*plan_id));
                    } else {
                        // Номер ППЗ/ДС всегда должен присутствовать, но как fallback остается
                        // вариант с uuid
                        return Err(ItemsError::DupItem(duplicate.source_uuid()));
                    }
                }
            }
        }

        // Если пользователь не передал все элементы на обновление
        if let Some(missing_item) = self
            .existing_items
            .iter()
            .filter(|existing_item| !existing_item.is_removed())
            .find(|existing_item| {
                if let Some(included_item) = already_included.get(&(
                    existing_item.source_uuid(),
                    existing_item.is_registered_by_d647(),
                )) {
                    // Элемент в одном списке должен совпадать по uuid
                    existing_item.uuid() != included_item.uuid()
                } else {
                    // Элемент в противоположном списке (список -> реестр и наоборот)
                    let another = already_included.get(&(
                        existing_item.source_uuid(),
                        !existing_item.is_registered_by_d647(),
                    ));

                    another.is_none()
                }
            })
        {
            return Err(ItemsError::MissingExisting(missing_item.uuid()));
        }

        Ok(())
    }

    /// Номер очередной позиции, или 0, если позиция удалена.
    ///
    /// Позиции должны сделовать строго сначала без признака реестра Д647, затем с признаком.
    pub(crate) fn next_number(&mut self, is_removed: bool, d647: bool) -> i64 {
        if self.prev_is_d647 != d647 {
            // когда переключаемся между списком и реестром, нумерация сбрасывается.
            self.prev_is_d647 = d647;
            self.number = 0;
        }
        if is_removed {
            0
        } else {
            self.number += 1;
            self.number
        }
    }
}
