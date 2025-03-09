#![feature(rustc_private)]
#![feature(let_chains)]
#![warn(unused_extern_crates)]
use std::sync::OnceLock;

use clippy_utils::diagnostics::{span_lint_and_help, span_lint_and_then};
use clippy_utils::ty::implements_trait;
use clippy_utils::get_trait_def_id;
use rustc_hir::Body;
use rustc_hir::def_id::{DefId, LocalDefId};
use rustc_lint::{LateContext, LateLintPass};
use rustc_middle::mir::{self, TerminatorKind};
use rustc_middle::ty::TyCtxt;

static DROP_WARNING_DEF_ID: OnceLock<Option<DefId>> = OnceLock::new();

const DROP_WARNING_PATH: [&str; 2] = ["nearly_linear", "DropWarning"];

extern crate rustc_hir;
extern crate rustc_middle;


dylint_linting::declare_late_lint! {
    /// ### What it does
    ///
    /// ### Why is this bad?
    ///
    /// ### Known problems
    ///
    /// Remove if none.
    ///
    /// ### Example
    ///
    /// ```rust
    /// // example code where a warning is issued
    /// ```
    ///
    /// Use instead:
    ///
    /// ```rust
    /// // example code that does not raise a warning
    /// ```
    pub DROP_LINEAR_TYPE,
    Warn,
    "description goes here"
}

/// Returns the `mir::Body` containing the node associated with `LocalDefId`.
pub fn enclosing_mir(tcx: TyCtxt<'_>, body_id_owner_def_id: LocalDefId) -> Option<&mir::Body<'_>> {
    if tcx.hir_body_owner_kind(body_id_owner_def_id).is_fn_or_closure() {
        Some(tcx.optimized_mir(body_id_owner_def_id.to_def_id()))
    } else {
        None
    }
}


/// Checks if a type implements DropWarning or contains any field that implements DropWarning
fn contains_drop_warning<'tcx>(cx: &LateContext<'tcx>, ty: rustc_middle::ty::Ty<'tcx>, drop_warning: DefId) -> bool {
    // Check if the type itself implements DropWarning
    if implements_trait(cx, ty, drop_warning, &[]) {
        return true;
    }

    // Recursively check fields for ADTs (structs, enums, unions)
    if let rustc_middle::ty::TyKind::Adt(adt_def, substs) = ty.kind() {
        for variant in adt_def.variants() {
            for field in &variant.fields {
                let field_ty = field.ty(cx.tcx, substs);
                if contains_drop_warning(cx, field_ty, drop_warning) {
                    return true;
                }
            }
        }
    }

    false
}


impl LateLintPass<'_> for DropLinearType {
    fn check_body(&mut self, cx: &LateContext<'_>, body: &Body<'_>) {
        // Find the MIR for the owner of this body. The owner is the function/closure/const
        // whose body this belongs to. https://rustc-dev-guide.rust-lang.org/hir.html#hir-bodies
        let body_owner_def_id = cx.tcx.hir_body_owner_def_id(body.id());
        let Some(mir) = enclosing_mir(cx.tcx, body_owner_def_id) else {
            return;
        };

        // Iterate through all basic blocks in the MIR
        for bb_data in mir.basic_blocks.iter() {
            // If it's a cleanup block (i.e. if it only runs after a panic), skip it.
            if bb_data.is_cleanup {
                continue;
            }
            if let Some(terminator) = &bb_data.terminator {
                // If the terminator is a drop, check if the type implements DropWarning.
                if let TerminatorKind::Drop { place, .. } = terminator.kind {
                    let ty = place.ty(mir, cx.tcx).ty;
                    if let Some(drop_warning) =
                        DROP_WARNING_DEF_ID.get_or_init(|| get_trait_def_id(cx.tcx, &DROP_WARNING_PATH))
                    {
                        if contains_drop_warning(cx, ty, *drop_warning) {
                            if let Some(decl) = place.as_local().and_then(|local| mir.local_decls.get(local)) {
                                let decl_span = decl.source_info.span;
                                // cx.span_lint(DROP_LINEAR_TYPE, decl_span, |diag| {
                                //     diag.primary_message("dropping an item that should be used");
                                //     diag.help("this item should always be consumed before going out of scope. You may
                                // have forgotten to call a function that consumes this value.");
                                //     diag.span_note(terminator.source_info.span, "Item is dropped here without being
                                // used");     diag.note("If you're sure it's safe to
                                // drop this value without using it, call `DropWarning::done()` on the value before
                                // exiting the scope."); });
                                span_lint_and_then(
                                    cx,
                                    DROP_LINEAR_TYPE,
                                    decl_span,
                                    "dropping an item that should be used",
                                    |diag| {
                                        diag.help("this item should always be consumed before going out of scope. You may have forgotten to call a function that consumes this value");
                                        diag.span_note(
                                            terminator.source_info.span,
                                            "item is dropped here without being used",
                                        );
                                        diag.note("this type will provide destructors which perform any necessary cleanup. Check its documentatino for more details");
                                    },
                                );
                            } else {
                                span_lint_and_help(
                                    cx,
                                    DROP_LINEAR_TYPE,
                                    terminator.source_info.span,
                                    "dropping a type that should be used",
                                    None,
                                    "one of the types in this scope was supposed to be used but was dropped. This lint *should* show you which variable was dropped, but was unable to do so because of a bug in the linter. Please report this error",
                                );
                            }
                        }
                    }
                }
            }
        }
    }
}


#[test]
fn ui() {
    dylint_testing::ui_test_examples(env!("CARGO_PKG_NAME"),);
}
