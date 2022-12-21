use std::{collections::BTreeMap, fmt};

use sway_types::IdentUnique;

use super::DeclarationId;

type SourceDecl = DeclarationId;
type DestinationDecl = DeclarationId;

/// The [DeclMapping] is used to create a mapping between a [SourceDecl] (LHS)
/// and a [DestinationDecl] (RHS).
pub(crate) struct DeclMapping {
    pub(crate) mapping: Vec<(SourceDecl, DestinationDecl)>,
}

impl fmt::Display for DeclMapping {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "DeclMapping {{ {} }}",
            self.mapping
                .iter()
                .map(|(source_type, dest_type)| { format!("{} -> {}", **source_type, **dest_type) })
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

impl fmt::Debug for DeclMapping {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "DeclMapping {{ {} }}",
            self.mapping
                .iter()
                .map(|(source_type, dest_type)| { format!("{:?} -> {:?}", source_type, dest_type) })
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

impl DeclMapping {
    pub(crate) fn is_empty(&self) -> bool {
        self.mapping.is_empty()
    }

    pub(crate) fn from_original_and_new_decl_ids(
        original_decl_ids: BTreeMap<IdentUnique, DeclarationId>,
        new_decl_ids: BTreeMap<IdentUnique, DeclarationId>,
    ) -> DeclMapping {
        let mut mapping = vec![];
        for (original_decl_name, original_decl_id) in original_decl_ids.into_iter() {
            for (new_decl_name, new_decl_id) in new_decl_ids.iter() {
                if new_decl_name.as_str() != original_decl_name.as_str() {
                    continue;
                }
                mapping.push((original_decl_id.clone(), new_decl_id.clone()));
            }
        }
        DeclMapping { mapping }
    }

    pub(crate) fn find_match(&self, decl_id: &SourceDecl) -> Option<DestinationDecl> {
        for (source_decl_id, dest_decl_id) in self.mapping.iter() {
            if **source_decl_id == **decl_id {
                return Some(dest_decl_id.clone());
            }
        }
        None
    }
}
