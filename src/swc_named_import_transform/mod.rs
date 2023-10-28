use std::collections::HashSet;

use swc_core::ecma::ast::*;
use swc_core::ecma::transforms::testing::test;
use swc_core::ecma::visit::{as_folder, VisitMut};
use swc_core::common::DUMMY_SP;

pub struct NamedImportTransformVisitor;

impl VisitMut for NamedImportTransformVisitor {
    // Implement necessary visit_mut_* methods for actual custom transform.
    // A comprehensive list of possible visitor methods can be found here:
    // https://rustdoc.swc.rs/swc_ecma_visit/trait.VisitMut.html
    fn visit_mut_import_decl(&mut self, import: &mut ImportDecl) {
        let mut specifier_names = HashSet::new();
        let mut skip_transform = false;
        for specifier in &import.specifiers {
            match specifier {
                ImportSpecifier::Named(specifier) => {
                    // Add the import name as string to the set
                    if let Some(imported) = &specifier.imported {
                        match imported {
                            ModuleExportName::Ident(ident) => {
                                specifier_names.insert(ident.sym.to_string());
                            }
                            ModuleExportName::Str(str_) => {
                                specifier_names.insert(str_.value.to_string());
                            }
                        }
                    } else {
                        specifier_names.insert(specifier.local.sym.to_string());
                    }
                }
                ImportSpecifier::Default(_) => {
                    skip_transform = true;
                    break;
                }
                ImportSpecifier::Namespace(_) => {
                    skip_transform = true;
                    break;
                }
            }
        }
        if !skip_transform {
            let mut names = specifier_names.into_iter().collect::<Vec<_>>();
            // Sort the names to make sure the order is consistent
            names.sort();

            let new_src = format!(
                "__barrel_optimize__?names={}!=!{}",
                names.join(","),
                import.src.value
            );

            // Create a new import declaration, keep everything the same except the source
            import.src = Box::new(Str {
                span: DUMMY_SP,
                value: new_src.into(),
                raw: None,
            });
        }
    }
}

test!(
  Default::default(),
  |_| as_folder(NamedImportTransformVisitor {}),
  basic,
  // Input codes
  r#"import { Button, ALink } from "foo";"#,
  // Output codes after transformed with plugin
  r#"import { Button, ALink } from "__barrel_optimize__?names=ALink,Button!=!foo";"#
);