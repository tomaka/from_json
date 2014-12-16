/*!

# from_json_macros

This crate defines two attributes:

 - `from_json_struct`: Derives `from_json::FromJson` for your structure.
 - `from_json_name`: Specifies the name of a field.

## from_json_struct

This attribute will attempt to read the structure from a JSON object.

If a field is not found, a `from_json::FieldNotFound` error is produced.
If you attempt to read from a non-object, a `from_json::ExpectError` is produced.

If one of the field has an attribute `from_json_name`, then this name will be used
 instead of the field name.

## Example

```
#![feature(phase)]

#[phase(plugin)]
extern crate from_json_macros;
extern crate from_json;
extern crate serialize;

#[from_json_struct]
struct Foo {
    a: int,
    #[from_json_name = "real_b"]
    b: bool,
    c: Bar,
}

#[from_json_struct]
struct Bar {
    e: Option<bool>,
    #[from_json_name = "type"]
    type_: String,
}

fn main() {
    use from_json::FromJson;

    let json = serialize::json::from_str(r#"{ "a": 5, "real_b": true, "c": { "e": false, "type": "hello" } }"#).unwrap();

    let _content: Foo = FromJson::from_json(&json).unwrap();
}
```

*/

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
    }.expand(ecx, meta_item, item, |i| push(i));
}

fn expand_struct_body(ecx: &mut base::ExtCtxt, span: codemap::Span,
                      substr: &generic::Substructure) -> P<ast::Expr>
{
    let ecx: &base::ExtCtxt = ecx;

    let input_param = substr.nonself_args[0].clone();

    let struct_name = format!("object {}", substr.type_ident.as_str());
    let struct_name = struct_name.as_slice();

    match substr.fields {
        &generic::StaticStruct(ref definition, generic::Named(_)) => {
            use syntax::attr::AttrMetaMethods;

            let ref fields = definition.fields;

            let content = fields.iter()
                .map(|&ast::StructField { ref node, .. }| {
                    let ident = node.ident().unwrap();
                    let ident_str = token::get_ident(ident);

                    let json_name = node.attrs.iter()
                        .find(|a| a.check_name("from_json_name"))
                        .and_then(|e| {
                            match e.node.value.node {
                                ast::MetaNameValue(_, ref value) => Some(value),
                                _ => None
                            }
                        })
                        .and_then(|value| {
                            match value.node {
                                ast::LitStr(ref s, _) => Some(s.get().to_string()),
                                _ => {
                                    ecx.span_err(span.clone(), "from_json_name requires \
                                                                a string literal");
                                    None
                                }
                            }
                        })
                        .unwrap_or(ident_str.get().to_string());
                    let json_name = json_name.as_slice();

                    let member_assign = quote_expr!(ecx, {
                        match $input_param.find($json_name) {
                            Some(elem) => match ::from_json::FromJson::from_json(elem) {
                                Ok(value) => value,
                                Err(e) => return Err(e)
                            },
                            None => match ::from_json::FromJson::from_json(&::serialize::json::Json::Null) {
                                Ok(value) => value,
                                Err(::from_json::FromJsonError::ExpectError(_, _)) => return Err(
                                    ::from_json::FromJsonError::FieldNotFound($json_name, $input_param.clone())),
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
                    Err(::from_json::FromJsonError::ExpectError($struct_name, $input_param.clone()))
                }
            )
        },

        _ => {
            ecx.span_err(span, "Unable to implement `from_json_struct` \
                                on a non-structure");
            ecx.expr_int(span, 0)
        }
    }
}
