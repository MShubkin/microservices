use shared_essential::{
    domain::{CommissionKind, EcAgenda, EcProtocol, PlanOrAmendment, ProtocolType},
    presentation::dto::response_request::{BusinessMessage, Message},
};

#[derive(Debug)]
pub enum ProtocolCreateMessage<'a> {
    InvalidPlanStatus,
    AlreadyInProtocol(&'a EcProtocol),
}

#[derive(Debug)]
pub enum ProtocolAddPlansMessage<'a> {
    Success(&'a EcProtocol),
    InvalidPlanStatus,
    AlreadyInProtocol(&'a EcProtocol),
}

#[derive(Debug)]
pub enum ProtocolGetItemsMessage<'a> {
    Success(&'a EcProtocol),
    InvalidPlanStatus,
    InvalidInPersonCommissionKind,
    InvalidCorrespondenceCommissionKind,
    AlreadyInProtocol(&'a EcProtocol),
}

#[derive(Debug)]
pub enum ProtocolApproveMessage {
    InvalidProtocolStatus,
    InPersonSuccess,
    CorrespondenceSuccess,
}

#[derive(Debug)]
pub enum ProtocolRemoveMessage {
    Success(ProtocolType),
    InvalidProtocolStatus,
    ProtocolStatusWarn,
}

#[derive(Debug)]
pub enum ProtocolUpdateMessage<'a> {
    ExclusionAlreadyInProtocol(&'a EcProtocol),
    ExclusionInvalidCommissionKind,
}

pub enum ConfirmDecisionMessage {
    Success,
}

pub enum ProtocolSignMessage {
    Success,
    InvalidProtocolStatus,
}

impl<'a> BusinessMessage for ProtocolCreateMessage<'a> {
    type Entity = PlanOrAmendment;

    fn singular(&self, entity: &Self::Entity) -> Message {
        let msg = match self {
            Self::InvalidPlanStatus => {
                Message::error(format!(
                    "ППЗ/ДС {} находится не на статусах СК. Выполнить формирование Протокола невозможно.",
                    entity.id()
                ))
            },
            Self::AlreadyInProtocol(protocol) => {
                Message::error(format!(
                    "ППЗ/ДС {} включена в Протокол {} от {}. Cоздание Протокола запрещено.",
                    entity.id(), protocol.id, protocol.protocol_date
                ))
                .with_param_item(*protocol)
            },
        };

        msg.with_param_item(entity)
    }

    fn plural<T>(&self, entities: &[T]) -> Message
    where
        T: AsRef<Self::Entity>,
    {
        let msg = match self {
            ProtocolCreateMessage::InvalidPlanStatus => {
                Message::error(format!(
                    "{} ППЗ/ДС находятся не на статусах СК. Выполнить формирование Протокола невозможно.",
                    entities.len()
                ))
            },
            ProtocolCreateMessage::AlreadyInProtocol(protocol) => {
                Message::error(format!(
                    "{} ППЗ/ДС включены в Протокол {} от {}. Cоздание Протокола запрещено.",
                    entities.len(), protocol.id, protocol.protocol_date
                ))
                .with_param_item(*protocol)
            }
        };

        msg.with_param_items(entities)
    }
}

impl<'a> ProtocolCreateMessage<'a> {
    pub fn success(protocol: &EcProtocol) -> Message {
        let msg = if protocol.protocol_type_id == ProtocolType::InPersonMeeting {
            Message::success(format!(
                "Вы сформировали Протокол очного заседания СК № {} от {}.",
                protocol.id, protocol.protocol_date
            ))
        } else {
            Message::success(format!(
                "Вы сформировали Протокол заочного заседания СК № {} от {}.",
                protocol.id, protocol.protocol_date
            ))
        };

        msg.with_param_item(protocol)
    }

    pub fn invalid_agenda_status(agenda: &EcAgenda) -> Message {
        Message::error(format!(
            r#"Выполнить формирование Протокола/добавление Повестки в Протокол невозможно. Повестка находится на статусе "{}""#, agenda.status_id
        ))
        .with_param_item(agenda)
    }

    pub fn empty_agenda(agenda: &EcAgenda) -> Message {
        Message::error(format!(
            "Выполнить формирование Протокола невозможно. В Повестке {} на {} отсутствуют или сняты с рассмотрения ППЗ/ДС", agenda.id, agenda.meeting_date
        ))
        .with_param_item(agenda)
    }
}

impl<'a> BusinessMessage for ProtocolAddPlansMessage<'a> {
    type Entity = PlanOrAmendment;

    fn singular(&self, entity: &Self::Entity) -> Message {
        let msg = match self {
            Self::Success(protocol) => {
                let msg = if protocol.protocol_type_id == ProtocolType::InPersonMeeting {
                    Message::success(format!(
                        "Вы добавили элемент Повестки СК в Протокол очной СК № {} от {}",
                        protocol.id, protocol.protocol_date
                    ))
                } else {
                    Message::success(format!(
                        "Вы добавили ППЗ/ДС {} в Протокол заочной СК № {} от {}",
                        entity.id(), protocol.id, protocol.protocol_date
                    ))
                };

                msg.with_param_item(*protocol)
            }
            Self::InvalidPlanStatus => {
                Message::error(format!(
                    "ППЗ/ДС {} находится не на статусах СК. Добавление в Протокол невозможно.",
                    entity.id()
                ))
            },
            Self::AlreadyInProtocol(protocol) => {
                Message::error(format!(
                    "ППЗ/ДС {} включена в Протокол {} от {}. Добавление в Протокол запрещено.",
                    entity.id(), protocol.id, protocol.protocol_date
                ))
                .with_param_item(*protocol)
            },
        };

        msg.with_param_item(entity)
    }

    fn plural<T>(&self, entities: &[T]) -> Message
    where
        T: AsRef<Self::Entity>,
    {
        let msg = match self {
            Self::Success(protocol) => {
                let msg = if protocol.protocol_type_id == ProtocolType::InPersonMeeting {
                    let case = match entities.len() {
                        ..=1 => "элемент",
                        2..=4 => "элемента",
                        _ => "элементов"
                    };

                    Message::success(format!(
                        "Вы добавили {} {} Повестки СК в Протокол очной СК № {} от {}",
                        entities.len(), case, protocol.id, protocol.protocol_date
                    ))
                } else {
                    Message::success(format!(
                        "Вы добавили {} ППЗ/ДС в Протокол заочной СК № {} от {}",
                        entities.len(), protocol.id, protocol.protocol_date
                    ))
                };

                msg.with_param_item(*protocol)
            }
            Self::InvalidPlanStatus => {
                Message::error(format!(
                    "{} ППЗ/ДС находятся не на статусах СК. Добавление в Протокол невозможно.",
                    entities.len()
                ))
            },
            Self::AlreadyInProtocol(protocol) => {
                Message::error(format!(
                    "{} ППЗ/ДС включены в Протокол {} от {}. Добавление в Протокол запрещено.",
                    entities.len(), protocol.id, protocol.protocol_date
                ))
                .with_param_item(*protocol)
            }
        };

        msg.with_param_items(entities)
    }
}

impl<'a> ProtocolAddPlansMessage<'a> {
    pub fn invalid_agenda_status(agenda: &EcAgenda) -> Message {
        Message::error(format!(
            r#"Выполнить добавление Повестки в Протокол невозможно. Повестка находится на статусе "{}""#, agenda.status_id
        ))
        .with_param_item(agenda)
    }

    pub fn empty_agenda(agenda: &EcAgenda) -> Message {
        Message::error(format!(
            "Выполнить добавление Повестки {} на {} в Протокол невозможно. В Повестке ППЗ/ДС отсутствуют или сняты с рассмотрения", agenda.id, agenda.meeting_date
        ))
        .with_param_item(agenda)
    }
}

impl<'a> BusinessMessage for ProtocolGetItemsMessage<'a> {
    type Entity = PlanOrAmendment;

    fn singular(&self, entity: &Self::Entity) -> Message {
        let msg = match self {
            Self::Success(protocol) => {
                let ty = match protocol.protocol_type_id {
                    ProtocolType::InPersonMeeting => "очной",
                    _ => "заочной"
                };
                Message::success(format!(
                    "Вы успешно добавили ППЗ/ДС {} в Протокол {} СК № {} от {}",
                    entity.id(), ty, protocol.id, protocol.protocol_date
                ))
            }

            Self::InvalidPlanStatus => {
                Message::error(format!(
                    "ППЗ/ДС {} находится не на статусах СК. Добавление в Протокол запрещено.",
                    entity.id()
                ))
            },
            Self::InvalidInPersonCommissionKind => Message::error(format!(
                "В ППЗ/ДС {} форма СК не относится к очной СК. Добавление невозможно.",
                entity.id()
            )),
            Self::InvalidCorrespondenceCommissionKind => Message::error(format!(
                "В ППЗ/ДС {} форма СК не относится к заочной СК. Добавление невозможно.",
                entity.id()
            )),
            Self::AlreadyInProtocol(protocol) => {
                Message::error(format!(
                    "ППЗ/ДС {} включена в Протокол {} от {}. Добавление в Протокол запрещено.",
                    entity.id(), protocol.id, protocol.protocol_date
                ))
                .with_param_item(*protocol)
            },
        };

        msg.with_param_item(entity)
    }

    fn plural<T>(&self, entities: &[T]) -> Message
    where
        T: AsRef<Self::Entity>,
    {
        let msg = match self {
            Self::Success(protocol) => {
                let ty = match protocol.protocol_type_id {
                    ProtocolType::InPersonMeeting => "очной",
                    _ => "заочной"
                };
                Message::success(format!(
                    "Вы успешно добавили {} ППЗ/ДС в Протокол {} СК № {} от {}",
                    entities.len(), ty, protocol.id, protocol.protocol_date
                ))
            }
            Self::InvalidPlanStatus => {
                Message::error(format!(
                    "{} ППЗ/ДС находятся не на статусах СК. Добавление в Протокол запрещено.",
                    entities.len()
                ))
            },
            Self::InvalidInPersonCommissionKind => Message::error(format!(
                "В {} ППЗ/ДС форма СК не относится к очной СК. Добавление невозможно.",
                entities.len()
            )),
            Self::InvalidCorrespondenceCommissionKind => Message::error(format!(
                "В {} ППЗ/ДС форма СК не относится к заочной СК. Добавление невозможно.",
                entities.len()
            )),
            Self::AlreadyInProtocol(protocol) => {
                Message::error(format!(
                    "{} ППЗ/ДС включены в Протокол {} от {}. Добавление в Протокол запрещено.",
                    entities.len(), protocol.id, protocol.protocol_date
                ))
                .with_param_item(*protocol)
            }
        };

        msg.with_param_items(entities)
    }
}

impl BusinessMessage for ProtocolApproveMessage {
    type Entity = EcProtocol;

    fn singular(&self, entity: &Self::Entity) -> Message {
        let msg = match self {
            ProtocolApproveMessage::InvalidProtocolStatus => {
                Message::error(format!(
                    r#"Перевести Протокол {} на статус "Утвержден" невозможно. Текущий статус Протокола "{}"."#,
                    entity.id, entity.status_id
                ))
            }

            ProtocolApproveMessage::InPersonSuccess => Message::success(format!(
                "Вы утвердили Протокол {} очной СК",
                entity.id
            )),
            ProtocolApproveMessage::CorrespondenceSuccess => Message::success(
                format!("Вы утверили Протокол {} заочной СК", entity.id),
            ),
        };

        msg.with_param_item(entity)
    }

    fn plural<T>(&self, entities: &[T]) -> Message
    where
        T: AsRef<Self::Entity>,
    {
        let msg = match self {
            ProtocolApproveMessage::InvalidProtocolStatus => {
                Message::error(format!(
                    r#"Перевести {} Протоколов на статус "Утвержден" невозможно."#,
                    entities.len()
                ))
            }

            ProtocolApproveMessage::InPersonSuccess => Message::success(format!(
                "Вы утвердили {} Протоколов очной СК",
                entities.len()
            )),
            ProtocolApproveMessage::CorrespondenceSuccess => Message::success(
                format!("Вы утверили {} Протоколов заочной СК", entities.len()),
            ),
        };

        msg.with_param_items(entities)
    }
}

impl BusinessMessage for ProtocolRemoveMessage {
    type Entity = EcProtocol;

    fn singular(&self, entity: &Self::Entity) -> Message {
        let msg = match self {
            Self::Success(ty) => match ty {
                ProtocolType::InPersonMeeting => Message::success(format!(
                    "Вы удалили Протокол {} очной СК",
                    entity.id
                )),
                _ => Message::success(format!(
                    "Вы удалили Протокол {} заочной СК",
                    entity.id
                )),
            },

            Self::InvalidProtocolStatus => Message::error(format!(
                r#"Перевести Протокол {} на статус "Удален" невозможно. Текущий статус Протокола "{}"."#,
                entity.id, entity.status_id
            )),
            Self::ProtocolStatusWarn => Message::warn(format!(
                r#"Текущий статус Протокола {} - "{}". Вы действительно хотите удалить Протокол?"#,
                entity.id, entity.status_id
            )),
        };

        msg.with_param_item(entity)
    }

    fn plural<T>(&self, entities: &[T]) -> Message
    where
        T: AsRef<Self::Entity>,
    {
        let msg = match self {
            Self::Success(ty) => {
                let case = match entities.len() {
                    ..=4 => "Протокола",
                    _ => "Протоколов",
                };
                match ty {
                    ProtocolType::InPersonMeeting => Message::success(format!(
                        "Вы удалили {} {} очной СК",
                        entities.len(),
                        case
                    )),
                    _ => Message::success(format!(
                        "Вы удалили {} {} заочной СК",
                        entities.len(),
                        case
                    )),
                }
            }

            Self::InvalidProtocolStatus => Message::error(format!(
                r#"Перевести {} Протоколов на статус "Удален" невозможно."#,
                entities.len()
            )),
            Self::ProtocolStatusWarn => Message::warn(format!(
                r#"{} Протоколов имеет статус "На согласовании" и "На подписании". Вы действительно хотите удалить Протокол?"#,
                entities.len()
            )),
        };

        msg.with_param_items(entities)
    }
}

impl<'a> BusinessMessage for ProtocolUpdateMessage<'a> {
    type Entity = PlanOrAmendment;

    fn singular(&self, entity: &Self::Entity) -> Message {
        let msg = match self {
            Self::ExclusionAlreadyInProtocol(protocol) => {
                Message::error(format!(
                    "ППЗ/ДС {id} включена в Протокол {protocol_id} от {protocol_date}. Снять с рассмотрения в Протоколе {protocol_id} от {protocol_date} невозможно.",
                    id = entity.id(), protocol_id = protocol.id, protocol_date = protocol.protocol_date
                ))
            },
            Self::ExclusionInvalidCommissionKind => {
                let commission = match entity.commission_kind_id() {
                    CommissionKind::Undefined => "не установлено",
                    CommissionKind::InPerson => "очную СК",
                    CommissionKind::Correspondence => "заочную СК",
                    CommissionKind::NotRequired => "не требуется СК",
                };
                Message::error(format!(
                    "В ППЗ/ДС {} изменена форма СК на {}. Снять с рассмотрения невозможно",
                    entity.id(), commission
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
            Self::ExclusionAlreadyInProtocol(protocol) => {
                Message::error(format!(
                    "{count} ППЗ/ДС включены в Протокол {protocol_id} от {protocol_date}. Снять с рассмотрения в Протоколе {protocol_id} от {protocol_date} невозможно.",
                    count = entities.len(), protocol_id = protocol.id, protocol_date = protocol.protocol_date
                ))
            },
            Self::ExclusionInvalidCommissionKind => {
                Message::error(format!(
                    "В {} ППЗ/ДС изменена форма СК. Снять с рассмотрения невозможно",
                    entities.len()
                ))
            },
        };

        msg.with_param_items(entities)
    }
}

impl<'a> ProtocolUpdateMessage<'a> {
    pub fn success(protocol: &EcProtocol) -> Message {
        Message::success(format!(
            "Протокол {} на {} сохранен",
            protocol.id, protocol.protocol_date,
        ))
        .with_param_item(protocol)
    }
}

impl BusinessMessage for ConfirmDecisionMessage {
    type Entity = PlanOrAmendment;

    fn singular(&self, entity: &Self::Entity) -> Message {
        let msg = match self {
            ConfirmDecisionMessage::Success => Message::success(format!(
                "Вы подтвердили решение ППЗ/ДС {}",
                entity.id()
            )),
        };

        msg.with_param_item(entity)
    }

    fn plural<T>(&self, entities: &[T]) -> Message
    where
        T: AsRef<Self::Entity>,
    {
        let msg = match self {
            ConfirmDecisionMessage::Success => Message::success(format!(
                "Вы подтвердили решение {} ППЗ/ДС",
                entities.len()
            )),
        };

        msg.with_param_items(entities)
    }
}

impl BusinessMessage for ProtocolSignMessage {
    type Entity = EcProtocol;

    fn singular(&self, entity: &Self::Entity) -> Message {
        let msg = match self {
            Self::Success => Message::success(format!(
                "Вы отправили на подписание Протокол {} очной СК",
                entity.id
            )),
            Self::InvalidProtocolStatus => Message::error(format!(
                r#"Перевести Протокол {} на статус "На подписании" невозможно. Текущий статус Протокола "{}"."#,
                entity.id, entity.status_id
            )),
        };

        msg.with_param_item(entity)
    }

    fn plural<T>(&self, entities: &[T]) -> Message
    where
        T: AsRef<Self::Entity>,
    {
        let msg = match self {
            Self::Success => Message::success(format!(
                "Вы отправили на подписание {} Протоколов очной СК",
                entities.len()
            )),
            Self::InvalidProtocolStatus => Message::error(String::from(
                r#"Перевести Протоколы на статус "На подписании" невозможно."#,
            )),
        };

        msg.with_param_items(entities)
    }
}
