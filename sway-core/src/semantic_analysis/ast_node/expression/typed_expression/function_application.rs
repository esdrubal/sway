use crate::{
    declaration_engine::ReplaceDecls,
    error::*,
    language::{ty, *},
    semantic_analysis::{ast_node::*, TypeCheckContext},
};
use std::collections::HashMap;
use sway_error::error::CompileError;
use sway_types::Spanned;

#[allow(clippy::too_many_arguments)]
pub(crate) fn instantiate_function_application(
    mut ctx: TypeCheckContext,
    mut function_decl: ty::TyFunctionDeclaration,
    call_path: CallPath,
    arguments: Vec<Expression>,
) -> CompileResult<ty::TyExpression> {
    let mut warnings = vec![];
    let mut errors = vec![];

    // 'purity' is that of the callee, 'opts.purity' of the caller.
    if !ctx.purity().can_call(function_decl.purity) {
        errors.push(CompileError::StorageAccessMismatch {
            attrs: promote_purity(ctx.purity(), function_decl.purity).to_attribute_syntax(),
            span: call_path.span(),
        });
    }

    // check that the number of parameters and the number of the arguments is the same
    check!(
        check_function_arguments_arity(arguments.len(), &function_decl, &call_path),
        return err(warnings, errors),
        warnings,
        errors
    );

    // Type check the arguments from the function application and unify them with
    // the arguments from the function application.
    let typed_arguments: Vec<(Ident, ty::TyExpression)> = arguments
        .into_iter()
        .zip(function_decl.parameters.iter())
        .map(|(arg, param)| {
            let ctx = ctx
                .by_ref()
                .with_help_text(
                    "The argument that has been provided to this function's type does \
                    not match the declared type of the parameter in the function \
                    declaration.",
                )
                .with_type_annotation(insert_type(TypeInfo::Unknown));
            let exp = check!(
                ty::TyExpression::type_check(ctx, arg.clone()),
                ty::TyExpression::error(arg.span()),
                warnings,
                errors
            );
            append!(
                unify_right(
                    exp.return_type,
                    param.type_id,
                    &exp.span,
                    "The argument that has been provided to this function's type does \
                    not match the declared type of the parameter in the function \
                    declaration."
                ),
                warnings,
                errors
            );

            // check for matching mutability
            let param_mutability =
                ty::VariableMutability::new_from_ref_mut(param.is_reference, param.is_mutable);
            if exp.gather_mutability().is_immutable() && param_mutability.is_mutable() {
                errors.push(CompileError::ImmutableArgumentToMutableParameter { span: arg.span() });
            }

            (param.name.clone(), exp)
        })
        .collect();

    // Handle the trait constraints. This includes checking to see if the trait
    // constraints are satisfied and replacing old decl ids based on the
    // constraint with new decl ids based on the new type.
    let decl_mapping = check!(
        TypeParameter::gather_decl_mapping_from_trait_constraints(
            ctx.by_ref(),
            &function_decl.type_parameters,
            &call_path.span()
        ),
        return err(warnings, errors),
        warnings,
        errors
    );
    function_decl.replace_decls(&decl_mapping);
    let return_type = function_decl.return_type;
    let span = function_decl.span.clone();
    let new_decl_id = de_insert_function(function_decl);

    let exp = ty::TyExpression {
        expression: ty::TyExpressionVariant::FunctionApplication {
            call_path,
            contract_call_params: HashMap::new(),
            arguments: typed_arguments,
            function_decl_id: new_decl_id,
            self_state_idx: None,
            selector: None,
        },
        return_type,
        span,
    };

    ok(exp, warnings, errors)
}

pub(crate) fn check_function_arguments_arity(
    arguments_len: usize,
    function_decl: &ty::TyFunctionDeclaration,
    call_path: &CallPath,
) -> CompileResult<()> {
    let warnings = vec![];
    let mut errors = vec![];
    match arguments_len.cmp(&function_decl.parameters.len()) {
        std::cmp::Ordering::Equal => ok((), warnings, errors),
        std::cmp::Ordering::Less => {
            errors.push(CompileError::TooFewArgumentsForFunction {
                span: call_path.span(),
                method_name: function_decl.name.clone(),
                expected: function_decl.parameters.len(),
                received: arguments_len,
            });
            err(warnings, errors)
        }
        std::cmp::Ordering::Greater => {
            errors.push(CompileError::TooManyArgumentsForFunction {
                span: call_path.span(),
                method_name: function_decl.name.clone(),
                expected: function_decl.parameters.len(),
                received: arguments_len,
            });
            err(warnings, errors)
        }
    }
}
