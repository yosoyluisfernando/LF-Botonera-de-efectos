//! Decide cómo añadir una raíz ya normalizada sin duplicar su árbol.
use std::path::Path;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LibraryCollection {
    Music,
    Effects,
}

impl LibraryCollection {
    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "music" => Ok(Self::Music),
            "effects" => Ok(Self::Effects),
            _ => Err("invalid_library_collection".into()),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Music => "music",
            Self::Effects => "effects",
        }
    }
}

#[derive(Clone, Debug)]
pub struct RootSpec {
    pub id: i64,
    pub path_key: String,
    pub collection: LibraryCollection,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RootAddKind {
    Add,
    AlreadyCovered { root_id: i64 },
    ChangeCollection { root_id: i64 },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RootAddPlan {
    pub kind: RootAddKind,
    pub merge_root_ids: Vec<i64>,
    pub exception_root_ids: Vec<i64>,
}

pub fn owner_for_path<'a>(roots: &'a [RootSpec], path_key: &str) -> Option<&'a RootSpec> {
    roots
        .iter()
        .filter(|root| Path::new(path_key).starts_with(Path::new(&root.path_key)))
        .max_by_key(|root| Path::new(&root.path_key).components().count())
}

pub fn plan_root_add(
    existing: &[RootSpec],
    path_key: &str,
    collection: LibraryCollection,
) -> RootAddPlan {
    for root in existing {
        if root.path_key == path_key {
            let kind = if root.collection == collection {
                RootAddKind::AlreadyCovered { root_id: root.id }
            } else {
                RootAddKind::ChangeCollection { root_id: root.id }
            };
            return RootAddPlan {
                kind,
                merge_root_ids: vec![],
                exception_root_ids: vec![],
            };
        }
    }
    let covering = existing
        .iter()
        .filter(|root| is_parent(&root.path_key, path_key))
        .max_by_key(|root| Path::new(&root.path_key).components().count());
    if let Some(root) = covering {
        if root.collection == collection {
            return RootAddPlan {
                kind: RootAddKind::AlreadyCovered { root_id: root.id },
                merge_root_ids: vec![],
                exception_root_ids: vec![],
            };
        }
    }
    let mut merge_root_ids = Vec::new();
    let mut exception_root_ids = Vec::new();
    for root in existing
        .iter()
        .filter(|root| is_parent(path_key, &root.path_key))
    {
        if root.collection == collection {
            merge_root_ids.push(root.id);
        } else {
            exception_root_ids.push(root.id);
        }
    }
    merge_root_ids.sort_unstable();
    exception_root_ids.sort_unstable();
    RootAddPlan {
        kind: RootAddKind::Add,
        merge_root_ids,
        exception_root_ids,
    }
}

fn is_parent(parent: &str, child: &str) -> bool {
    parent != child && Path::new(child).starts_with(Path::new(parent))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn root(id: i64, path: &str, collection: LibraryCollection) -> RootSpec {
        RootSpec {
            id,
            path_key: path.into(),
            collection,
        }
    }

    #[test]
    fn same_root_is_not_added_twice() {
        let roots = [root(1, "d:/audio", LibraryCollection::Music)];
        let plan = plan_root_add(&roots, "d:/audio", LibraryCollection::Music);
        assert_eq!(plan.kind, RootAddKind::AlreadyCovered { root_id: 1 });
    }

    #[test]
    fn same_collection_subfolder_is_already_covered() {
        let roots = [root(1, "d:/audio", LibraryCollection::Music)];
        let plan = plan_root_add(&roots, "d:/audio/rock", LibraryCollection::Music);
        assert_eq!(plan.kind, RootAddKind::AlreadyCovered { root_id: 1 });
    }

    #[test]
    fn parent_unifies_children_of_the_same_collection() {
        let roots = [
            root(2, "d:/audio/rock", LibraryCollection::Music),
            root(3, "d:/audio/pop", LibraryCollection::Music),
        ];
        let plan = plan_root_add(&roots, "d:/audio", LibraryCollection::Music);
        assert_eq!(plan.merge_root_ids, [2, 3]);
        assert!(plan.exception_root_ids.is_empty());
    }

    #[test]
    fn different_collection_subfolder_is_an_exception() {
        let roots = [root(1, "d:/audio", LibraryCollection::Music)];
        let plan = plan_root_add(&roots, "d:/audio/fx", LibraryCollection::Effects);
        assert_eq!(plan.kind, RootAddKind::Add);
        assert!(plan.merge_root_ids.is_empty());
    }

    #[test]
    fn parent_keeps_more_specific_other_collection() {
        let roots = [
            root(2, "d:/audio/rock", LibraryCollection::Music),
            root(3, "d:/audio/fx", LibraryCollection::Effects),
        ];
        let plan = plan_root_add(&roots, "d:/audio", LibraryCollection::Music);
        assert_eq!(plan.merge_root_ids, [2]);
        assert_eq!(plan.exception_root_ids, [3]);
    }

    #[test]
    fn same_path_in_another_collection_requires_reclassification() {
        let roots = [root(1, "d:/audio", LibraryCollection::Music)];
        let plan = plan_root_add(&roots, "d:/audio", LibraryCollection::Effects);
        assert_eq!(plan.kind, RootAddKind::ChangeCollection { root_id: 1 });
    }

    #[test]
    fn most_specific_root_decides_which_collection_covers_a_path() {
        let roots = [
            root(1, "d:/audio", LibraryCollection::Music),
            root(2, "d:/audio/fx", LibraryCollection::Effects),
        ];
        let plan = plan_root_add(&roots, "d:/audio/fx/jingles", LibraryCollection::Music);
        assert_eq!(plan.kind, RootAddKind::Add);
        assert_eq!(
            owner_for_path(&roots, "d:/audio/fx/sirena.wav").unwrap().id,
            2
        );
    }
}
