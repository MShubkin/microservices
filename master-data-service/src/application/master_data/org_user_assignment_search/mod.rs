use std::time::Duration;

use asez2_shared_db::{
    ahash::AHashMap,
    db_item::{joined::JoinTo, AsezTimestamp, Select},
    DbItem,
};
use itertools::Itertools;
use monolith_service::{dto::user::MonolithUser, http::MonolithHttpService};
use shared_essential::{
    domain::{
        DepartmentLevel, OrganizationalStructure, UserAssignmentAndOrgSelector,
    },
    presentation::dto::master_data::{
        error::MasterDataResult, response::OrgUserAssignmentResItem,
    },
};
use sqlx::PgPool;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use crate::{
    infrastructure::GlobalConfig, presentation::dto::OrgUserAssignmentResData,
};

/// Период обновления данных по пользователям, 1 час.
const ORGANIZATIONAL_USER_ASSIGNMENT_FETCH_INTERVAL: Duration =
    Duration::from_secs(60 * 60);

#[derive(Debug, Default)]
pub(crate) struct SearchData {
    /// Пользователи, в привязке к иерархии профильных департаментов.
    pub(crate) users: AHashMap<i32, OrgUserAssignment>,
    /// Иерархия профильных департаментов.
    pub(crate) deps: AHashMap<i32, OrgStructData>,
    /// временная метка последнего обновления данных.
    pub(crate) timestamp: AsezTimestamp,
}

impl SearchData {
    /// Поиск пользователей, относящихся к департаменту/отделу `dep_id`,
    /// находящемуся на уровне `level` иерархии, по подстроке `text`.
    fn search(
        &self,
        text: &str,
        dep_id: i32,
        level: DepartmentLevel,
        from: usize,
        quantity: usize,
    ) -> Vec<OrgUserAssignmentResItem> {
        let text = text.to_lowercase();
        self.users
            .values()
            .filter(|x| x.matches_dep_id(dep_id, level))
            .filter(|x| x.ui_text.to_lowercase().contains(&text))
            .sorted_by_key(|x| {
                (&x.user.last_name, &x.user.first_name, &x.user.patronymic_name)
            })
            .skip(from)
            .take(quantity)
            .map(Into::into)
            .collect()
    }

    fn search_by_id(&self, id: i32) -> Option<&OrgUserAssignment> {
        self.users.get(&id)
    }

    /// Возвращает идентификатор уровня организационной единицы с идентификатором `dep_id`.
    fn structure_level(&self, dep_id: i32) -> Option<DepartmentLevel> {
        self.deps.get(&dep_id).map(|x| x.level)
    }

    /// Возвращает идентификатор департамента, к которой приписан пользователь `user_id`.
    fn department_id(&self, user_id: i32) -> Option<i32> {
        self.users
            .get(&user_id)
            .and_then(|org_user_assign| org_user_assign.dep_ids.first())
            .and_then(|dep_ids| dep_ids.get(&DepartmentLevel::Department))
    }

    fn is_up_to_date(&self) -> bool {
        *AsezTimestamp::now() - *self.timestamp
            < ORGANIZATIONAL_USER_ASSIGNMENT_FETCH_INTERVAL
    }
}

#[derive(Debug, Default)]
pub(crate) struct OrgUserAssignment {
    pub(crate) user: MonolithUser,
    pub(crate) ui_text: String,
    pub(crate) dep_ids: Vec<OrgHierarchy>,
}

impl OrgUserAssignment {
    fn matches_dep_id(&self, dep_id: i32, level: DepartmentLevel) -> bool {
        self.dep_ids.iter().any(|x| x.matches(dep_id, level))
    }
}

impl From<&OrgUserAssignment> for OrgUserAssignmentResItem {
    fn from(value: &OrgUserAssignment) -> Self {
        let OrgUserAssignment {
            user:
                MonolithUser {
                    id,
                    email,
                    first_name,
                    last_name,
                    patronymic_name,
                    text,
                    phone,
                    ..
                },
            ui_text,
            dep_ids,
        } = value;
        OrgUserAssignmentResItem {
            id: *id,
            first_name: first_name.clone(),
            patronymic_name: patronymic_name.clone(),
            last_name: last_name.clone(),
            text: text.clone(),
            ui_text: ui_text.clone(),
            email: email.clone(),
            phone: phone.clone(),
            organization_structure_ids: dep_ids
                .iter()
                .map(|x| x.to_ids())
                .collect(),
        }
    }
}

#[derive(Debug, Default)]
pub(crate) struct OrgHierarchy {
    pub(crate) org_ids: Vec<Option<(i32, DepartmentLevel)>>,
}

impl FromIterator<(i32, DepartmentLevel)> for OrgHierarchy {
    fn from_iter<T: IntoIterator<Item = (i32, DepartmentLevel)>>(iter: T) -> Self {
        let mut h = OrgHierarchy::new();
        iter.into_iter().for_each(|(id, level)| h.push(id, level));
        h
    }
}

impl OrgHierarchy {
    fn new() -> Self {
        OrgHierarchy { org_ids: vec![] }
    }

    fn index(level: &DepartmentLevel) -> usize {
        (*level as i16 - 1).try_into().unwrap_or_default()
    }

    fn push(&mut self, id: i32, level: DepartmentLevel) {
        let idx = Self::index(&level);
        if self.org_ids.len() <= idx {
            self.org_ids.resize(idx + 1, None);
        }
        self.org_ids[idx] = Some((id, level));
    }

    fn matches(&self, id: i32, level: DepartmentLevel) -> bool {
        let idx = Self::index(&level);
        self.org_ids.get(idx).and_then(|x| *x).map_or(false, |x| x.0 == id)
    }

    fn to_ids(&self) -> Vec<i32> {
        self.org_ids.iter().map(|x| x.map_or(0, |x| x.0)).collect()
    }

    fn get(&self, level: &DepartmentLevel) -> Option<i32> {
        let idx = Self::index(level);
        self.org_ids.get(idx).and_then(Option::as_ref).map(|x| x.0)
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct OrgStructData {
    level: DepartmentLevel,
    parent: Option<i32>,
}

pub(crate) async fn fetch_user_search_data(
    pool: &PgPool,
    monolith: MonolithHttpService,
    token: String,
) -> MasterDataResult<SearchData> {
    let org_users = fetch_org_users(pool).await?;
    let org_structure = fetch_org_structure(pool).await?;
    let users = monolith.search_users_by_id(org_users.keys(), token, 0).await?;

    let org_struct_ids = |id| {
        org_users
            .get(&id)
            .map_or(vec![], |org_ids| org_struct_ids(org_ids, &org_structure))
    };

    let users = users
        .into_iter()
        .map(|user| {
            let ui_text = user.ui_text();
            let org_struct_ids = org_struct_ids(user.id);
            (
                user.id,
                OrgUserAssignment {
                    user,
                    ui_text,
                    dep_ids: org_struct_ids,
                },
            )
        })
        .collect();
    Ok(SearchData {
        users,
        deps: org_structure,
        timestamp: AsezTimestamp::now(),
    })
}

fn org_struct_ids(
    org_ids: &[i32],
    org_structure: &AHashMap<i32, OrgStructData>,
) -> Vec<OrgHierarchy> {
    org_ids
        .iter()
        .copied()
        .map(|org_id| {
            // функция получения данных по идентификатору организации
            let osd = |org_id: &_| {
                org_structure
                    .get(org_id)
                    .copied()
                    .map(|OrgStructData { level, parent }| (*org_id, level, parent))
            };
            // строим иерархию организаций от org_id вверх
            std::iter::successors(osd(&org_id), |(_, level, parent_id)| {
                parent_id.as_ref().and_then(osd).and_then(|next|
                // sanity check: level should be strictly decreasing.
                // this also protects the algorithm against cycles.
                if &next.1 < level {
                    Some(next)
                } else {
                    warn!(kind = "org_users_cache", id = %next.0, level = %next.1, "invalid level in org structure");
                    None
                })
            })
            .map(|(org_id, level, _)| (org_id, level))
            .collect()
        })
        .collect()
}

/// По данным из таблицы `organizational_structure` строит отображение отделов в
/// их уровень иерархии и вышестоящие орг единицы.
async fn fetch_org_structure(
    pool: &PgPool,
) -> MasterDataResult<AHashMap<i32, OrgStructData>> {
    const FIELDS: &[&str] = &[
        OrganizationalStructure::id,
        OrganizationalStructure::parent_id,
        OrganizationalStructure::level,
    ];
    Ok(OrganizationalStructure::select(
        &Select::with_fields(FIELDS).eq(OrganizationalStructure::is_removed, false),
        pool,
    )
    .await?
    .into_iter()
    .map(|x| {
        if x.parent_id.is_none() && x.level != DepartmentLevel::GP {
            warn!(kind = "org_user_search", id = x.id, level = %x.level,
                  "структурная единица не верхнего уровня не имеет вышестоящей организации");
        }
        (x.id, OrgStructData { level: x.level, parent: x.parent_id})
    })
    .collect())
}

/// По данным из таблицы `organization_user_assignment` строит отображение из
/// идентификаторов пользователей в отделы, к которым они приписаны.
async fn fetch_org_users(
    pool: &PgPool,
) -> MasterDataResult<AHashMap<i32, Vec<i32>>> {
    let mut res = AHashMap::new();
    for (user_id, org_ids) in &UserAssignmentAndOrgSelector::new(Select::default())
        .set_org_structure(OrganizationalStructure::join_default().selecting(
            Select::default().eq(OrganizationalStructure::is_removed, false),
        ))
        .get(pool)
        .await?
        .into_iter()
        .map(|x| (x.user_assignment.user_id, x.org_structure.id))
        .group_by(|(user_id, _)| *user_id)
    {
        res.insert(user_id, org_ids.map(|(_, org_id)| org_id).collect());
    }
    Ok(res)
}

lazy_static::lazy_static!(
    pub(crate) static ref ORGANIZATIONAL_USER_ASSIGNMENT: RwLock<Option<SearchData>> = RwLock::new(Default::default());
);

/// Запускает таску, обновляющую кеш организационных назначений пользователей.
pub(crate) async fn run_refresh_organizational_user_assignment(
    token: String,
    config: &GlobalConfig,
) {
    let pool = config.db_pool.clone();
    let monolith = config.monolith.clone();
    tokio::spawn(async move {
        debug!(kind = "org_users_cache", "Запрос на обновление кеша");
        match refresh_maybe_organizational_user_assignment(token, &pool, monolith)
            .await
        {
            Ok(false) => {}
            Ok(true) => {
                info!(
                    kind = "org_users_cache",
                    "Кеш организационных назначений обновлен"
                );
            }
            Err(e) => {
                error!(
                    kind = "org_users_cache",
                    "Не удалось обновить кеш организационных назначений: {e}"
                );
            }
        }
    });
}

/// Обновляет кеш организационных назначений пользователя, если предыдущее
/// обновление было выполнено ранее, чем
/// [`ORGANIZATIONAL_USER_ASSIGNMENT_FETCH_INTERVAL`](ORGANIZATIONAL_USER_ASSIGNMENT_FETCH_INTERVAL)
/// секунд назад.
pub(crate) async fn refresh_maybe_organizational_user_assignment(
    token: String,
    pool: &PgPool,
    monolith: MonolithHttpService,
) -> MasterDataResult<bool> {
    let mut organizational_user_assignment =
        ORGANIZATIONAL_USER_ASSIGNMENT.write().await;

    if organizational_user_assignment
        .as_ref()
        .map_or(false, SearchData::is_up_to_date)
    {
        debug!(kind = "org_users_cache", "Обновление кеша не требуется");
        return Ok(false);
    }

    debug!(kind = "org_users_cache", "Обновление кеша");
    let new_val = fetch_user_search_data(pool, monolith, token).await?;
    *organizational_user_assignment = Some(new_val);

    Ok(true)
}

pub(crate) async fn organizational_user_assignment(
    user_id: i32,
    search: &str,
    organization_structure_id: Option<i32>,
    from: usize,
    quantity: usize,
) -> MasterDataResult<OrgUserAssignmentResData> {
    let search_data = ORGANIZATIONAL_USER_ASSIGNMENT.read().await;
    let Some(search_data) = (*search_data).as_ref() else {
        warn!(kind = "org_users_cache", "Кеш поиска не инициализирован");
        return Ok(OrgUserAssignmentResData {
            value: Default::default(),
        });
    };

    let org_id_level = if let Some(org_id) = organization_structure_id {
        search_data.structure_level(org_id).map(|level| (org_id, level))
    } else {
        search_data
            .department_id(user_id)
            .map(|department_id| (department_id, DepartmentLevel::Department))
    };

    let value = org_id_level
        .map(|(org_id, level)| {
            search_data.search(search, org_id, level, from, quantity)
        })
        .unwrap_or_default();

    Ok(OrgUserAssignmentResData { value })
}

pub async fn organizational_user_assignment_by_id<'a, I>(
    ids: I,
) -> MasterDataResult<OrgUserAssignmentResData>
where
    I: IntoIterator<Item = &'a i32>,
{
    let search_data = ORGANIZATIONAL_USER_ASSIGNMENT.read().await;
    let Some(search_data) = (*search_data).as_ref() else {
        warn!(kind = "org_users_cache", "Кеш поиска не инициализирован");
        return Ok(OrgUserAssignmentResData {
            value: Default::default(),
        });
    };

    let value = ids
        .into_iter()
        .filter_map(|id| search_data.search_by_id(*id).map(Into::into))
        .collect();

    Ok(OrgUserAssignmentResData { value })
}

#[cfg(test)]
mod tests {
    use super::{DepartmentLevel::*, *};

    #[test]
    fn org_hierarchy() {
        let mut orgs = OrgHierarchy::new();

        orgs.push(10, Department);
        assert_eq!(orgs.org_ids[1], Some((10, Department)));

        orgs.push(20, SubDivision);
        assert_eq!(orgs.org_ids[3], Some((20, SubDivision)));

        orgs.push(15, Division);
        assert_eq!(orgs.org_ids[2], Some((15, Division)));

        let orgs = orgs.to_ids();
        assert_eq!(&orgs, &[0, 10, 15, 20]);
    }

    macro_rules! os_item {
        ($id:expr, $level:expr, $parent:expr) => {
            (
                $id,
                OrgStructData {
                    level: $level,
                    parent: Some($parent),
                },
            )
        };
        ($id:expr, $level:expr) => {
            (
                $id,
                OrgStructData {
                    level: $level,
                    parent: None,
                },
            )
        };
    }

    #[test]
    fn org_structure_ids() {
        let org_structure = [
            os_item!(1, GP),
            os_item!(2, Department, 1),
            os_item!(21, Division, 2),
            os_item!(211, SubDivision, 21),
            os_item!(3, Department, 1),
        ]
        .into_iter()
        .collect();
        let ids = org_struct_ids(&[211], &org_structure);
        assert_eq!(ids.len(), 1);
        assert_eq!(
            &ids[0].org_ids,
            &[
                Some((1, GP)),
                Some((2, Department)),
                Some((21, Division)),
                Some((211, SubDivision)),
            ]
        );
    }

    #[test]
    fn org_structure_ids_cycle() {
        let org_structure = [os_item!(111, Division, 111)].into_iter().collect();
        let _ = org_struct_ids(&[111], &org_structure);

        let org_structure =
            [os_item!(321, SubDivision, 21), os_item!(21, Division, 321)]
                .into_iter()
                .collect();
        let _ = org_struct_ids(&[321], &org_structure);
    }
}
