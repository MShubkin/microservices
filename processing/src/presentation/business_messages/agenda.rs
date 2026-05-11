use shared_essential::{
    domain::{EcAgenda, EcProtocol, EcProtocolItem, PlanOrAmendment},
    presentation::dto::response_request::{BusinessMessage, Message},
};

#[derive(Debug)]
pub enum AgendaCreateMessage<'a> {
    Success(&'a EcAgenda),
    InvalidPlanStatus,
    AlreadyInProtocol(&'a EcProtocol, &'a EcProtocolItem),
    AlreadyInAgenda(&'a EcAgenda),
}

#[derive(Debug)]
pub enum AgendaAddPlansMessage<'a> {
    Success(&'a EcAgenda),
    InvalidPlanStatus,
    InvalidCommissionKind,
    AlreadyInProtocol(&'a EcProtocol, &'a EcProtocolItem),
    AlreadyInAgenda(&'a EcAgenda),
    AlreadyInCurrentAgenda(&'a EcAgenda),
    DifferentDepartment(&'a EcAgenda),
}

#[derive(Debug)]
pub enum AgendaTransferPlansMessage<'a> {
    Success(&'a EcAgenda),
    InvalidPlanStatus,
    #[allow(unused)]
    InvalidCommissionKind,
    AlreadyInProtocol(&'a EcProtocol, &'a EcProtocolItem),
    NotIncludedInAgenda,
}

#[derive(Debug)]
pub enum AgendaGetItemsMessage<'a> {
    Success(&'a EcAgenda),
    InvalidCommissionKind,
    AlreadyInProtocol(&'a EcProtocol, &'a EcProtocolItem),
    AlreadyInAgenda(&'a EcAgenda),
    AlreadyInCurrentAgenda(&'a EcAgenda),
    DifferentDepartment(&'a EcAgenda),
}

#[derive(Debug)]
pub enum AgendaRemoveItemsMessage<'a> {
    AlreadyInProtocol(&'a EcProtocol),
}

#[derive(Debug)]
pub enum AgendaSendMessage {
    Success,
    EmptyAgenda,
    InvalidAgendaStatus,
}

#[derive(Debug)]
#[allow(clippy::enum_variant_names)]
pub enum AgendaUpdateMessage<'a> {
    ExclusionAlreadyInProtocol(&'a EcProtocol, &'a EcProtocolItem),
    ExclusionAlreadyInAgenda(&'a EcAgenda),
    AlreadyInProtocol(&'a EcProtocol, &'a EcProtocolItem),
    AlreadyInAgenda(&'a EcAgenda),
    ExclusionInvalidCommissionKind,
}

impl<'a> BusinessMessage for AgendaCreateMessage<'a> {
    type Entity = PlanOrAmendment;

    fn singular(&self, entity: &Self::Entity) -> Message {
        match self {
            Self::Success(agenda) => Message::success(format!(
                "Вы сформировали Повестку очной СК № {} на {}",
                agenda.id,
                agenda.meeting_date
            ))
            .with_param_item(*agenda),

            Self::InvalidPlanStatus => Message::error(format!(
                "Формирование Повестки запрещено. ППЗ/ДС {} находится не на статусах СК",
                entity.id()
            ))
            .with_param_item(entity),
            Self::AlreadyInProtocol(protocol, protocol_item) => Message::error(format!(
                r#"Формирование Повестки запрещено. ППЗ/ДС {} включена в Протокол {} от {} с решением "{}""#,
                entity.id(), protocol.id, protocol.protocol_date, protocol_item.result_id
            ))
            .with_param_item(*protocol)
            .with_param_item(entity),
            Self::AlreadyInAgenda(agenda) => Message::error(format!(
                "Формирование Повестки запрещено. ППЗ/ДС {} включена в Повестку {} на {}",
                entity.id(), agenda.id, agenda.meeting_date
            ))
            .with_param_item(*agenda)
            .with_param_item(entity),
        }
    }

    fn plural<T>(&self, entities: &[T]) -> Message
    where
        T: AsRef<Self::Entity>,
    {
        match self {
            Self::Success(agenda) => Message::success(format!(
                "Вы сформировали Повестку очной СК № {} на {}",
                agenda.id,
                agenda.meeting_date
            ))
            .with_param_item(*agenda),

            Self::InvalidPlanStatus => Message::error(format!(
                "Формирование Повестки запрещено. {} ППЗ/ДС находятся не на статусах СК",
                entities.len()
            ))
            .with_param_items(entities),
            Self::AlreadyInProtocol(protocol, protocol_item) => Message::error(format!(
                r#"Формирование Повестки запрещено. {} ППЗ/ДС включены в Протокол {} от {} с решением "{}""#,
                entities.len(), protocol.id, protocol.protocol_date, protocol_item.result_id
            ))
            .with_param_item(*protocol)
            .with_param_items(entities),
            Self::AlreadyInAgenda(agenda) => Message::error(format!(
                "Формирование Повестки запрещено. {} ППЗ/ДС включены в Повестку {} на {}",
                entities.len(), agenda.id, agenda.meeting_date
            ))
            .with_param_item(*agenda)
            .with_param_items(entities),
        }
    }
}

impl<'a> AgendaCreateMessage<'a> {
    pub fn different_department() -> Message {
        Message::warn(String::from(
            "По ППЗ/ДС указаны разные Департаменты АЦ. Продолжить?",
        ))
    }

    pub fn different_plan_sections() -> Message {
        Message::warn(String::from("Проверьте возможность включения в Повестку ППЗ/ДС с указанными разделами Плана. Продолжить?"))
    }
}

impl<'a> BusinessMessage for AgendaAddPlansMessage<'a> {
    type Entity = PlanOrAmendment;

    fn singular(&self, entity: &Self::Entity) -> Message {
        let msg = match self {
            Self::Success(agenda) => Message::success(format!(
                "Вы добавили в Повестку {} на {} ППЗ/ДС {}",
                agenda.id,
                agenda.meeting_date,
                entity.id()
            ))
            .with_param_item(*agenda),
            Self::DifferentDepartment(agenda) => Message::warn(format!(
                "По ППЗ/ДС {} отличается «Департамент АЦ» от указанного в Повестке {} на {}. Продолжить?",
                entity.id(),
                agenda.id,
                agenda.meeting_date,
            )),

            Self::InvalidPlanStatus => Message::error(format!(
                "Добавление в Повестку запрещено. ППЗ/ДС {} находится не на статусах СК",
                entity.id()
            )),
            Self::InvalidCommissionKind => Message::error(format!(
                "В ППЗ/ДС {} форма СК не относится к очной СК. Добавление невозможно.",
                entity.id()
            )),
            Self::AlreadyInProtocol(protocol, protocol_item) => Message::error(format!(
                r#"Добавление в Повестку запрещено. ППЗ/ДС {} включена в Протокол {} от {} с решением "{}""#,
                entity.id(), protocol.id, protocol.protocol_date, protocol_item.result_id
            ))
            .with_param_item(*protocol),
            Self::AlreadyInAgenda(agenda) => Message::error(format!(
                "Добавление в Повестку запрещено. ППЗ/ДС {} включена в Повестку {} на {}",
                entity.id(), agenda.id, agenda.meeting_date
            ))
            .with_param_item(*agenda),
            Self::AlreadyInCurrentAgenda(agenda) => Message::error(format!(
                "ППЗ/ДС № {} уже включена в Повестку № {} на {}",
                entity.id(), agenda.id, agenda.meeting_date
            )),
        };

        msg.with_param_item(entity)
    }

    fn plural<T>(&self, entities: &[T]) -> Message
    where
        T: AsRef<Self::Entity>,
    {
        let msg = match self {
            Self::Success(agenda) => Message::success(format!(
                "Вы добавили в Повестку {} на {} {} ППЗ/ДС",
                agenda.id,
                agenda.meeting_date,
                entities.len()
            ))
            .with_param_item(*agenda),

            Self::DifferentDepartment(agenda) => Message::warn(format!(
                "По {} ППЗ/ДС отличается «Департамент АЦ» от указанного в Повестке {} на {}. Продолжить?",
                entities.len(),
                agenda.id,
                agenda.meeting_date,
            ))
            .with_param_item(*agenda),

            Self::InvalidPlanStatus => Message::error(format!(
                "Добавление в Повестку запрещено. {} ППЗ/ДС находятся не на статусах СК",
                entities.len()
            )),
            Self::InvalidCommissionKind => Message::error(format!(
                "В {} ППЗ/ДС форма СК не относится к очной СК. Добавление невозможно.",
                entities.len()
            )),
            Self::AlreadyInProtocol(protocol, protocol_item) => Message::error(format!(
                r#"Добавление в Повестку запрещено. {} ППЗ/ДС включены в Протокол {} от {} с решением "{}""#,
                entities.len(), protocol.id, protocol.protocol_date, protocol_item.result_id
            )).with_param_item(*protocol),
            Self::AlreadyInAgenda(agenda) => Message::error(format!(
                "Добавление в Повестку запрещено. {} ППЗ/ДС включены в Повестку {} на {}",
                entities.len(), agenda.id, agenda.meeting_date
            )).with_param_item(*agenda),
            Self::AlreadyInCurrentAgenda(agenda) => Message::error(format!(
                "{} ППЗ/ДС уже включены в Повестку № {} на {}",
                entities.len(), agenda.id, agenda.meeting_date
            )),
        };

        msg.with_param_items(entities)
    }
}

impl<'a> AgendaAddPlansMessage<'a> {
    pub fn invalid_agenda_status(agenda: &EcAgenda) -> Message {
        Message::error(
            format!(r#"Добавить ППЗ/ДС в Повестку {} на {} невозможно. Повестка находится на статусе "{}""#, agenda.id, agenda.meeting_date, agenda.status_id)
        ).with_param_item(agenda)
    }
}

impl<'a> BusinessMessage for AgendaGetItemsMessage<'a> {
    type Entity = PlanOrAmendment;

    fn singular(&self, entity: &Self::Entity) -> Message {
        match self {
            Self::Success(agenda) => Message::success(format!(
                "Вы успешно добавили ППЗ/ДС {} в Повестку очной СК {} на {}",
                entity.id(),
                agenda.id,
                agenda.meeting_date
            ))
            .with_param_item(entity),

            Self::DifferentDepartment(agenda) => {
                AgendaAddPlansMessage::DifferentDepartment(agenda).singular(entity)
            }

            Self::InvalidCommissionKind => {
                AgendaAddPlansMessage::InvalidCommissionKind.singular(entity)
            }
            Self::AlreadyInProtocol(protocol, protocol_item) => {
                AgendaAddPlansMessage::AlreadyInProtocol(protocol, protocol_item)
                    .singular(entity)
            }
            Self::AlreadyInAgenda(agenda) => {
                AgendaAddPlansMessage::AlreadyInAgenda(agenda).singular(entity)
            }
            Self::AlreadyInCurrentAgenda(agenda) => {
                AgendaAddPlansMessage::AlreadyInCurrentAgenda(agenda)
                    .singular(entity)
            }
        }
    }

    fn plural<T>(&self, entities: &[T]) -> Message
    where
        T: AsRef<Self::Entity>,
    {
        match self {
            Self::Success(agenda) => Message::success(format!(
                "Вы успешно добавили {} ППЗ/ДС в Повестку очной СК {} на {}",
                entities.len(),
                agenda.id,
                agenda.meeting_date,
            ))
            .with_param_items(entities),

            Self::DifferentDepartment(agenda) => {
                AgendaAddPlansMessage::DifferentDepartment(agenda).plural(entities)
            }

            Self::InvalidCommissionKind => {
                AgendaAddPlansMessage::InvalidCommissionKind.plural(entities)
            }
            Self::AlreadyInProtocol(protocol, protocol_item) => {
                AgendaAddPlansMessage::AlreadyInProtocol(protocol, protocol_item)
                    .plural(entities)
            }
            Self::AlreadyInAgenda(agenda) => {
                AgendaAddPlansMessage::AlreadyInAgenda(agenda).plural(entities)
            }
            Self::AlreadyInCurrentAgenda(agenda) => {
                AgendaAddPlansMessage::AlreadyInCurrentAgenda(agenda)
                    .plural(entities)
            }
        }
    }
}

impl BusinessMessage for AgendaSendMessage {
    type Entity = EcAgenda;

    fn singular(&self, entity: &Self::Entity) -> Message {
        let msg = match self {
            AgendaSendMessage::Success => {
                Message::success(format!(
                    "Вы отправили Повестку № {} очной СК Экспертам АЦ и Руководителю АЦ",
                    entity.id
                ))
            },
            AgendaSendMessage::InvalidAgendaStatus => {
                Message::error(format!(
                    r#"Выполнить отправку Повестки {id} на {meeting_date} невозможно. Повестка находится на статусе "{agenda_status}"."#,
                    id = entity.id,
                    meeting_date = entity.meeting_date,
                    agenda_status = entity.status_id,
                ))
            },
            AgendaSendMessage::EmptyAgenda => {
                Message::error(format!(
                    "Выполнить отправку Повестки {} на {} невозможно. В Повестке ППЗ/ДС отсутствуют или сняты с рассмотрения",
                    entity.id, entity.meeting_date
                ))
            },
        };

        msg.with_param_item(entity)
    }

    fn plural<T>(&self, entities: &[T]) -> Message
    where
        T: AsRef<Self::Entity>,
    {
        let msg = match self {
            AgendaSendMessage::Success => {
                Message::success(format!(
                    "Вы отправили {} Повесткок очной СК Экспертам АЦ и Руководителю АЦ",
                    entities.len()
                ))
            },
            AgendaSendMessage::InvalidAgendaStatus => {
                Message::error(String::from(
                    "Выполнить отправку Повесткок невозможно. Повестки находятся не на требуемых статусах"
                ))
            },
            AgendaSendMessage::EmptyAgenda => {
                Message::error(String::from(
                    "Выполнить отправку Повесток невозможно. В Повестках ППЗ/ДС отсутствуют или сняты с рассмотрения"
                ))
            },
        };

        msg.with_param_items(entities)
    }
}

impl<'a> BusinessMessage for AgendaUpdateMessage<'a> {
    type Entity = PlanOrAmendment;

    fn singular(&self, entity: &Self::Entity) -> Message {
        match self {
            Self::ExclusionAlreadyInProtocol(protocol, protocol_item) => {
                Message::error(format!(
                    r#"ППЗ/ДС {id} включена в Протокол {protocol_id} от {protocol_date} с решением "{result}". Исключить признак "Снято с рассмотрения" невозможно"#,
                    id = entity.id(), protocol_id = protocol.id, protocol_date = protocol.protocol_date, result = protocol_item.result_id
                ))
                .with_param_item(*protocol)
                .with_param_item(entity)
            },
            Self::ExclusionAlreadyInAgenda(agenda) => {
                Message::error(format!(
                    r#"ППЗ/ДС {id} включена в Повестку {agenda_id} на {meeting_date}. Исключить признак "Снято с рассмотрения" невозможно"#,
                    id = entity.id(), agenda_id = agenda.id, meeting_date = agenda.meeting_date
                ))
                .with_param_item(*agenda)
                .with_param_item(entity)
            },
            Self::ExclusionInvalidCommissionKind => {
                Message::error(format!(
                    r#"В ППЗ/ДС {} форма СК не относится к очной СК. Исключить признак "Снято с рассмотрения" невозможно"#,
                    entity.id()
                ))
                .with_param_item(entity)
            },
            Self::AlreadyInProtocol(protocol, protocol_item) => {
                AgendaAddPlansMessage::AlreadyInProtocol(protocol, protocol_item)
                    .singular(entity)
            },
            Self::AlreadyInAgenda(agenda) => {
                AgendaAddPlansMessage::AlreadyInAgenda(agenda).singular(entity)
            },
        }
    }

    fn plural<T>(&self, entities: &[T]) -> Message
    where
        T: AsRef<Self::Entity>,
    {
        match self {
            Self::ExclusionAlreadyInProtocol(protocol, protocol_item) => {
                Message::error(format!(
                    r#"{count} ППЗ/ДС включены в Протокол {protocol_id} от {protocol_date} с решением "{result}". Исключить признак "Снято с рассмотрения" невозможно"#,
                    count = entities.len(), protocol_id = protocol.id, protocol_date = protocol.protocol_date, result = protocol_item.result_id
                ))
                .with_param_item(*protocol)
                .with_param_items(entities)
            },
            Self::ExclusionAlreadyInAgenda(agenda) => {
                Message::error(format!(
                    r#"{count} ППЗ/ДС включены в Повестку {agenda_id} на {meeting_date}. Исключить признак "Снято с рассмотрения" невозможно"#,
                    count = entities.len(), agenda_id = agenda.id, meeting_date = agenda.meeting_date
                ))
                .with_param_item(*agenda)
                .with_param_items(entities)
            },
            Self::ExclusionInvalidCommissionKind => {
                Message::error(format!(
                    r#"В {} ППЗ/ДС форма СК не относится к очной СК. Исключить признак "Снято с рассмотрения" невозможно"#,
                    entities.len()
                ))
                .with_param_items(entities)
            },
            Self::AlreadyInProtocol(protocol, protocol_item) => {
                AgendaAddPlansMessage::AlreadyInProtocol(protocol, protocol_item)
                    .plural(entities)
            },
            Self::AlreadyInAgenda(agenda) => {
                AgendaAddPlansMessage::AlreadyInAgenda(agenda).plural(entities)
            },
        }
    }
}

impl<'a> BusinessMessage for AgendaRemoveItemsMessage<'a> {
    type Entity = PlanOrAmendment;

    fn singular(&self, entity: &Self::Entity) -> Message {
        let msg = match self {
            AgendaRemoveItemsMessage::AlreadyInProtocol(protocol) => {
                Message::error(format!(
                    "ППЗ/ДС {} включена в Протокол {} от {}. Удаление выполнить невозможно.", 
                    entity.id(), protocol.id, protocol.protocol_date
                ))
                .with_param_item(*protocol)
            }
        };

        msg.with_param_item(entity)
    }

    fn plural<T>(&self, entities: &[T]) -> Message
    where
        T: AsRef<Self::Entity>,
    {
        let msg = match self {
            AgendaRemoveItemsMessage::AlreadyInProtocol(protocol) => {
                Message::error(format!(
                    "{} ППЗ/ДС включены в Протокол {} от {}. Удаление выполнить невозможно.", 
                    entities.len(), protocol.id, protocol.protocol_date
                ))
                .with_param_item(*protocol)
            }
        };

        msg.with_param_items(entities)
    }
}

impl<'a> BusinessMessage for AgendaTransferPlansMessage<'a> {
    type Entity = PlanOrAmendment;

    fn singular(&self, entity: &Self::Entity) -> Message {
        let msg = match self {
            Self::Success(agenda) => Message::success(
                format!("Вы изменили Повестку очной СК для ППЗ/ДС {}", entity.id())
            ).with_param_item(*agenda),

            Self::InvalidPlanStatus => Message::error(format!(
                "Изменение Повестки запрещено. ППЗ/ДС {} находится не на статусах СК",
                entity.id()
            )),
            Self::InvalidCommissionKind => Message::error(format!(
                "Изменение Повестки запрещено. В ППЗ/ДС {} форма СК не относится к очной СК",
                entity.id()
            )),
            Self::AlreadyInProtocol(protocol, protocol_item) => Message::error(format!(
                r#"Изменение Повестки запрещено. ППЗ/ДС {} включена в Протокол {} от {} с решением "{}""#,
                entity.id(), protocol.id, protocol.protocol_date, protocol_item.result_id
            ))
            .with_param_item(*protocol),
            Self::NotIncludedInAgenda => {
                Message::error(format!(
                    "Изменение Повестки запрещено. ППЗ/ДС {} не включена в Повестку",
                    entity.id()
                ))
            }
        };

        msg.with_param_item(entity)
    }

    fn plural<T>(&self, entities: &[T]) -> Message
    where
        T: AsRef<Self::Entity>,
    {
        let msg = match self {
            Self::Success(agenda) => Message::success(
                format!("Вы изменили Повестку очной СК для {} ППЗ/ДС", entities.len())
            ).with_param_item(*agenda),

            Self::InvalidPlanStatus => Message::error(format!(
                "Изменение Повестки запрещено. {} ППЗ/ДС находятся не на статусах СК",
                entities.len(),
            )),
            Self::InvalidCommissionKind => Message::error(format!(
                "Изменение Повестки запрещено. В {} ППЗ/ДС форма СК не относится к очной СК",
                entities.len(),
            )),
            Self::AlreadyInProtocol(protocol, protocol_item) => Message::error(format!(
                r#"Изменение Повестки запрещено. {} ППЗ/ДС включены в Протокол {} от {} с решением "{}""#,
                entities.len(), protocol.id, protocol.protocol_date, protocol_item.result_id
            ))
            .with_param_item(*protocol),
            Self::NotIncludedInAgenda => {
                Message::error(format!(
                    "Изменение Повестки запрещено. {} ППЗ/ДС не включены в Повестку",
                    entities.len(),
                ))
            }
        };

        msg.with_param_items(entities)
    }
}

impl<'a> AgendaTransferPlansMessage<'a> {
    pub fn different_department() -> Message {
        Message::warn(String::from(
            "По ППЗ/ДС указаны разные Департаменты АЦ. Продолжить?",
        ))
    }

    pub fn different_plan_sections() -> Message {
        Message::warn(String::from("Проверьте возможность включения в Повестку ППЗ/ДС с указанными разделами Плана. Продолжить?"))
    }

    pub fn invalid_agenda_status(agenda: &EcAgenda) -> Message {
        Message::error(format!(
            r#"Повестка {} находится на статусе "Сформирован Протокол". Выполнить изменение невозможно."#,
            agenda.id
        ))
    }
}
