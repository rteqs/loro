use enum_as_inner::EnumAsInner;
use loro_internal::container::ContainerID;
use loro_internal::delta::TreeDiff;
use loro_internal::event::EventTriggerKind;
use loro_internal::handler::{TextDelta, ValueOrHandler};
use loro_internal::FxHashMap;
use loro_internal::{
    event::{Diff as DiffInner, Index},
    ContainerDiff as ContainerDiffInner, DiffEvent as DiffEventInner,
};
use std::sync::Arc;

use crate::ValueOrContainer;

pub type Subscriber = Arc<dyn (for<'a> Fn(DiffEvent<'a>)) + Send + Sync>;

#[derive(Debug)]
pub struct DiffEvent<'a> {
    pub triggered_by: EventTriggerKind,
    pub origin: &'a str,
    pub current_target: Option<ContainerID>,
    pub events: Vec<ContainerDiff<'a>>,
}

#[derive(Debug)]
pub struct ContainerDiff<'a> {
    pub target: &'a ContainerID,
    pub path: &'a [(ContainerID, Index)],
    pub diff: Diff<'a>,
}

#[derive(Debug, EnumAsInner)]
pub enum Diff<'a> {
    List(Vec<ListDiffItem>),
    Text(Vec<TextDelta>),
    Map(MapDelta<'a>),
    Tree(&'a TreeDiff),
}

#[derive(Debug)]
pub enum ListDiffItem {
    Insert {
        insert: Vec<ValueOrContainer>,
        is_move: bool,
    },
    Delete {
        delete: usize,
    },
    Retain {
        retain: usize,
    },
}

#[derive(Debug)]
pub struct MapDelta<'a> {
    pub updated: FxHashMap<&'a str, Option<ValueOrContainer>>,
}

impl<'a> From<DiffEventInner<'a>> for DiffEvent<'a> {
    fn from(value: DiffEventInner<'a>) -> Self {
        DiffEvent {
            triggered_by: value.event_meta.by,
            origin: &value.event_meta.origin,
            current_target: value.current_target,
            events: value.events.iter().map(|&diff| diff.into()).collect(),
        }
    }
}

impl<'a> From<&'a ContainerDiffInner> for ContainerDiff<'a> {
    fn from(value: &'a ContainerDiffInner) -> Self {
        ContainerDiff {
            target: &value.id,
            path: &value.path,
            diff: (&value.diff).into(),
        }
    }
}

impl<'a> From<&'a DiffInner> for Diff<'a> {
    fn from(value: &'a DiffInner) -> Self {
        match value {
            DiffInner::List(l) => {
                let mut ans = Vec::new();
                for item in l.iter() {
                    match item {
                        delta::DeltaItem::Retain { len, .. } => {
                            ans.push(ListDiffItem::Retain { retain: *len });
                        }
                        delta::DeltaItem::Replace {
                            value,
                            delete,
                            attr,
                        } => {
                            if value.len() > 0 {
                                ans.push(ListDiffItem::Insert {
                                    insert: value
                                        .iter()
                                        .map(|v| ValueOrContainer::from(v.clone()))
                                        .collect(),
                                    is_move: attr.from_move,
                                });
                            }
                            if *delete > 0 {
                                ans.push(ListDiffItem::Delete { delete: *delete });
                            }
                        }
                    }
                }

                Diff::List(ans)
            }
            DiffInner::Map(m) => Diff::Map(MapDelta {
                updated: m
                    .updated
                    .iter()
                    .map(|(k, v)| (k.as_ref(), v.value.clone().map(|v| v.into())))
                    .collect(),
            }),
            DiffInner::Text(t) => {
                let text = TextDelta::from_text_diff(t.iter());
                Diff::Text(text)
            }
            DiffInner::Tree(t) => Diff::Tree(t),
            _ => todo!(),
        }
    }
}

impl From<ValueOrHandler> for ValueOrContainer {
    fn from(value: ValueOrHandler) -> Self {
        match value {
            ValueOrHandler::Value(v) => ValueOrContainer::Value(v),
            ValueOrHandler::Handler(h) => ValueOrContainer::Container(h.into()),
        }
    }
}
