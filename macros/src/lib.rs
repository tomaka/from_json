#![feature(plugin_registrar)]
#![feature(quote)]

extern crate rustc;
extern crate syntax;

use syntax::ast;
use syntax::ext::base;
use syntax::ext::build::AstBuilder;
use syntax::ext::deriving::generic;
use syntax::codemap;
use syntax::parse::token;
use syntax::ptr::P;

#[doc(hidden)]
#[plugin_registrar]
pub fn registrar(registry: &mut rustc::plugin::Registry) {
    use syntax::parse::token;
    registry.register_syntax_extension(token::intern("from_json_struct"),
        syntax::ext::base::Decorator(box expand_struct));
}

pub fn expand_struct(ecx: &mut base::ExtCtxt, span: codemap::Span,
                     meta_item: &ast::MetaItem, item: &ast::Item,
                     push: |P<ast::Item>|)
{
    generic::TraitDef {
        span: span,
        attributes: Vec::new(),
        path: generic::ty::Path {
            path: vec!["from_json", "FromJson"],
            lifetime: None,
            params: Vec::new(),
            global: true,
        },
        additional_bounds: Vec::new(),
        generics: generic::ty::LifetimeBounds::empty(),
        methods: vec![
            generic::MethodDef {
                name: "from_json",
                generics: generic::ty::LifetimeBounds::empty(),
                explicit_self: None,
                args: vec![
                    generic::ty::Ptr(
                        box generic::ty::Literal(generic::ty::Path {
                            path: vec!["serialize", "json", "Json"],
                            lifetime: None,
                            params: vec![],
                            global: false,
                        }),
                        generic::ty::Borrowed(None, syntax::ast::MutImmutable)
                    )
                ],

                ret_ty: generic::ty::Literal(generic::ty::Path::new_(
                    vec!["std", "result", "Result"],
                    None,
                    vec![
                        box generic::ty::Self,
                        box generic::ty::Literal(
                            generic::ty::Path::new(vec!["from_json", "FromJsonError"])
                        )
                    ],
                    true
                )),
                attributes: vec![],
                combine_substructure: generic::combine_substructure(expand_struct_body),
            },
        ],
    }.expand(ecx, meta_item, item, push);
}

fn expand_struct_body(ecx: &mut base::ExtCtxt, span: codemap::Span,
                      substr: &generic::Substructure) -> P<ast::Expr>
{
    let ecx: &base::ExtCtxt = ecx;

    let input_param = substr.nonself_args[0].clone();

    let struct_name = format!("object {}", substr.type_ident.as_str());
    let struct_name = struct_name.as_slice();

    match substr.fields {
        &generic::StaticStruct(_, generic::Named(ref fields)) => {
            let content = fields.iter()
                .map(|&(ident, _)| {
                    let ident_str = token::get_ident(ident);
                    let ident_str = ident_str.get();

                    let member_assign = quote_expr!(ecx, {
                        match $input_param.find(&$ident_str.to_string()) {
                            Some(elem) => match ::from_json::FromJson::from_json(elem) {
                                Ok(value) => value,
                                Err(e) => return Err(e)
                            },
                            None => match ::from_json::FromJson::from_json(&::serialize::json::Null) {
                                Ok(value) => value,
                                Err(::from_json::ExpectError(_, _)) => return Err(
                                    ::from_json::FieldNotFound($ident_str, $input_param.clone())),
                                Err(e) => return Err(e)
                            }
                        }
                    });

                    ast::Field {
                        ident: ast::SpannedIdent {
                            node: ident.clone(),
                            span: span.clone(),
                        },
                        expr: member_assign,
                        span: span.clone(),
                    }
                }).collect::<Vec<ast::Field>>();

            let struct_def = ecx.expr_struct(span.clone(), ecx.path_ident(span.clone(),
                substr.type_ident), content);

            quote_expr!(ecx, 
                if $input_param.is_object() {
                    Ok($struct_def)
                } else {
                    Err(::from_json::ExpectError($struct_name, $input_param.clone()))
                }
            )
        },

        _ => {
            ecx.span_err(span, "Unable to implement `from_json_struct` \
                                on a non-structure");
            ecx.expr_lit(span, ast::LitNil)
        }
    }
}
