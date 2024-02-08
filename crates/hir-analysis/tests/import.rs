mod test_db;
use std::path::Path;

use dir_test::{dir_test, Fixture};
use fe_compiler_test_utils::snap_test;
use fe_hir_analysis::name_resolution::{ImportAnalysisPass, NameDerivation, ResolvedImports};
use hir::{analysis_pass::ModuleAnalysisPass, hir_def::Use};
use rustc_hash::FxHashMap;
use test_db::{HirAnalysisTestDb, HirPropertyFormatter};

#[dir_test(
    dir: "$CARGO_MANIFEST_DIR/test_files/imports",
    glob: "*.fe"
)]
fn test_standalone(fixture: Fixture<&str>) {
    let mut db = HirAnalysisTestDb::default();
    let path = Path::new(fixture.path());
    let file_name = path.file_name().and_then(|file| file.to_str()).unwrap();
    let (top_mod, mut prop_formatter) = db.new_stand_alone(file_name, fixture.content());

    let mut pass = ImportAnalysisPass::new(&db);
    let resolved_imports = pass.resolve_imports(top_mod.ingot(&db));
    let diags = pass.run_on_module(top_mod);
    if !diags.is_empty() {
        panic!("Failed to resolve imports");
    }

    let res = format_imports(&db, &mut prop_formatter, resolved_imports);
    snap_test!(res, fixture.path());
}

fn format_imports(
    db: &HirAnalysisTestDb,
    prop_formatter: &mut HirPropertyFormatter,
    imports: &ResolvedImports,
) -> String {
    let mut use_res_map: FxHashMap<Use, Vec<String>> = FxHashMap::default();

    for name_resolved in imports.named_resolved.values().flat_map(|r| r.values()) {
        for res in name_resolved.iter() {
            match res.derivation {
                NameDerivation::NamedImported(use_) => use_res_map
                    .entry(use_)
                    .or_default()
                    .push(res.pretty_path(db).unwrap()),
                _ => unreachable!(),
            }
        }
    }

    for (_, glob_set) in imports.glob_resolved.iter() {
        for (&use_, res_set_with_ident) in glob_set.iter() {
            for (ident, res_set) in res_set_with_ident.iter() {
                let ident = ident.data(db);
                for res in res_set {
                    let def_path = res.pretty_path(db).unwrap();
                    let resolved = format!("{} as {}", def_path, ident);
                    use_res_map.entry(use_).or_default().push(resolved)
                }
            }
        }
    }
    for (use_, mut values) in use_res_map.into_iter() {
        let use_span = use_.lazy_span().into();
        values.sort_unstable();
        let imported_names = values.join(" | ");
        prop_formatter.push_prop(use_.top_mod(db), use_span, imported_names)
    }

    prop_formatter.finish(db)
}