use askama::Template;

#[derive(Template)]
#[template(path = "render_dart/prelude.txt", escape = "none")]
pub struct PreludeTemplate {}

#[derive(Template)]
#[template(path = "render_dart/custom_types.txt", escape = "none")]
pub struct CustomTypesTemplate<'a> {
    pub custom_types: &'a [super::DartCustomType],
}

#[derive(Template)]
#[template(path = "render_dart/native_functions.txt", escape = "none")]
pub struct NativeFunctionsTemplate<'a> {
    pub cfuncs: &'a [super::DartNativeFunction],
}

#[derive(Template)]
#[template(path = "render_dart/wire_functions.txt", escape = "none")]
pub struct WireFunctionsTemplate<'a> {
    pub funcs: &'a [super::DartWireFunction],
}

#[derive(Template)]
#[template(path = "render_dart/record.txt", escape = "none")]
pub struct RecordTemplate<'a> {
    pub record: &'a super::DartRecord,
}

#[derive(Template)]
#[template(path = "render_dart/enum_c_style.txt", escape = "none")]
pub struct CStyleEnumTemplate<'a> {
    pub enum_def: &'a super::DartEnum,
}

#[derive(Template)]
#[template(path = "render_dart/enum_placeholder.txt", escape = "none")]
pub struct EnumPlaceholderTemplate<'a> {
    pub enum_def: &'a super::DartEnum,
}

#[derive(Template)]
#[template(path = "render_dart/hook.build.dart.txt", escape = "none")]
pub struct BuildHookTemplate<'a> {
    pub artifact_name: &'a str,
}

#[derive(Template)]
#[template(path = "render_dart/pubspec.yaml.txt", escape = "none")]
pub struct PubspecTemplate<'a> {
    pub artifact_name: &'a str,
    pub description: Option<&'a str>,
    pub version: Option<&'a str>,
    pub repository: Option<&'a str>,
}

#[cfg(test)]
mod tests {
    use askama::Template;

    use super::super::plan::{
        DartBlittableField, DartBlittableLayout, DartConstructor, DartConstructorKind, DartEnum,
        DartEnumKind, DartEnumVariant, DartFunction, DartFunctionParam, DartNativeFunction,
        DartNativeFunctionParam, DartNativeType, DartRecord, DartRecordField, DartType,
        DartWireFunction, DartWireFunctionParam, DartWireReturn,
    };
    use super::*;
    use crate::ir::{
        OffsetExpr, PrimitiveType, ReadOp, ReadSeq, SizeExpr, ValueExpr, WireShape, WriteOp,
        WriteSeq,
    };

    fn primitive_read_seq(primitive: PrimitiveType, size: usize) -> ReadSeq {
        ReadSeq {
            size: SizeExpr::Fixed(size),
            ops: vec![ReadOp::Primitive {
                primitive,
                offset: OffsetExpr::Base,
            }],
            shape: WireShape::Value,
        }
    }

    fn primitive_write_seq(primitive: PrimitiveType, size: usize, name: &str) -> WriteSeq {
        WriteSeq {
            size: SizeExpr::Fixed(size),
            ops: vec![WriteOp::Primitive {
                primitive,
                value: ValueExpr::Named(name.to_string()),
            }],
            shape: WireShape::Value,
        }
    }

    fn f64_field(name: &str, offset: usize) -> DartRecordField {
        DartRecordField {
            name: name.to_string(),
            offset,
            dart_type: "double".to_string(),
            read_seq: primitive_read_seq(PrimitiveType::F64, 8),
            write_seq: primitive_write_seq(PrimitiveType::F64, 8, name),
        }
    }

    fn f64_blittable_field(name: &str, offset: usize) -> DartBlittableField {
        DartBlittableField {
            name: name.to_string(),
            primitive: PrimitiveType::F64,
            native_type: DartNativeType::Primitive(PrimitiveType::F64),
            offset_const_name: format!("_k$offset{}", name.to_uppercase()),
            offset,
        }
    }

    #[test]
    fn snapshot_record_blittable_point_fields_only() {
        let record = DartRecord {
            name: "Point".to_string(),
            is_error: false,
            fields: vec![f64_field("x", 0), f64_field("y", 8)],
            blittable_layout: Some(DartBlittableLayout {
                struct_name: "_$Point$Struct".to_string(),
                struct_size: 16,
                fields: vec![f64_blittable_field("x", 0), f64_blittable_field("y", 8)],
            }),
            constructors: vec![],
            methods: vec![],
        };
        let template = RecordTemplate { record: &record };
        insta::assert_snapshot!(template.render().unwrap());
    }

    #[test]
    fn snapshot_record_with_placeholder_ctors_and_methods() {
        let record = DartRecord {
            name: "Point".to_string(),
            is_error: false,
            fields: vec![f64_field("x", 0), f64_field("y", 8)],
            blittable_layout: Some(DartBlittableLayout {
                struct_name: "_$Point$Struct".to_string(),
                struct_size: 16,
                fields: vec![f64_blittable_field("x", 0), f64_blittable_field("y", 8)],
            }),
            constructors: vec![
                DartConstructor {
                    ffi_name: "boltffi_point_new".to_string(),
                    kind: DartConstructorKind::Default,
                    params: vec![
                        DartFunctionParam {
                            name: "x".to_string(),
                            ty: DartType::Double,
                        },
                        DartFunctionParam {
                            name: "y".to_string(),
                            ty: DartType::Double,
                        },
                    ],
                    is_fallible: false,
                },
                DartConstructor {
                    ffi_name: "boltffi_point_origin".to_string(),
                    kind: DartConstructorKind::Named {
                        name: "origin".to_string(),
                    },
                    params: vec![],
                    is_fallible: false,
                },
            ],
            methods: vec![
                DartFunction {
                    name: "distance".to_string(),
                    ffi_name: "boltffi_point_distance".to_string(),
                    params: vec![],
                    ret_ty: DartType::Double,
                },
                DartFunction {
                    name: "scale".to_string(),
                    ffi_name: "boltffi_point_scale".to_string(),
                    params: vec![DartFunctionParam {
                        name: "factor".to_string(),
                        ty: DartType::Double,
                    }],
                    ret_ty: DartType::Void,
                },
            ],
        };
        let template = RecordTemplate { record: &record };
        insta::assert_snapshot!(template.render().unwrap());
    }

    #[test]
    fn snapshot_native_functions_simple() {
        let funcs = vec![
            DartNativeFunction {
                symbol: "boltffi_add".to_string(),
                params: vec![
                    DartNativeFunctionParam {
                        name: "a".to_string(),
                        native_type: DartNativeType::Primitive(PrimitiveType::I32),
                    },
                    DartNativeFunctionParam {
                        name: "b".to_string(),
                        native_type: DartNativeType::Primitive(PrimitiveType::I32),
                    },
                ],
                return_type: DartNativeType::Primitive(PrimitiveType::I32),
                is_leaf: true,
            },
            DartNativeFunction {
                symbol: "boltffi_negate".to_string(),
                params: vec![DartNativeFunctionParam {
                    name: "x".to_string(),
                    native_type: DartNativeType::Primitive(PrimitiveType::F64),
                }],
                return_type: DartNativeType::Primitive(PrimitiveType::F64),
                is_leaf: true,
            },
        ];
        let template = NativeFunctionsTemplate { cfuncs: &funcs };
        insta::assert_snapshot!(template.render().unwrap());
    }

    fn cstyle_variant(name: &str, discriminant: i128) -> DartEnumVariant {
        DartEnumVariant {
            name: name.to_string(),
            discriminant,
            doc: None,
        }
    }

    #[test]
    fn snapshot_enum_c_style_i32_tag() {
        let enum_def = DartEnum {
            name: "Priority".to_string(),
            kind: DartEnumKind::CStyle,
            tag_type: PrimitiveType::I32,
            variants: vec![
                cstyle_variant("low", 0),
                cstyle_variant("medium", 1),
                cstyle_variant("high", 2),
                cstyle_variant("critical", 3),
            ],
            doc: None,
        };
        let template = CStyleEnumTemplate {
            enum_def: &enum_def,
        };
        insta::assert_snapshot!(template.render().unwrap());
    }

    #[test]
    fn snapshot_enum_c_style_non_ordinal_u16_tag() {
        // Discriminants that diverge from ordinals, with a non-i32 tag type.
        let enum_def = DartEnum {
            name: "HttpCode".to_string(),
            kind: DartEnumKind::CStyle,
            tag_type: PrimitiveType::U16,
            variants: vec![
                cstyle_variant("ok", 200),
                cstyle_variant("notFound", 404),
                cstyle_variant("serverError", 500),
            ],
            doc: None,
        };
        let template = CStyleEnumTemplate {
            enum_def: &enum_def,
        };
        insta::assert_snapshot!(template.render().unwrap());
    }

    #[test]
    fn snapshot_enum_placeholder_data() {
        let enum_def = DartEnum {
            name: "Shape".to_string(),
            kind: DartEnumKind::Sealed,
            tag_type: PrimitiveType::I32,
            variants: vec![],
            doc: None,
        };
        let template = EnumPlaceholderTemplate {
            enum_def: &enum_def,
        };
        insta::assert_snapshot!(template.render().unwrap());
    }

    #[test]
    fn snapshot_enum_placeholder_error() {
        let enum_def = DartEnum {
            name: "ComputeError".to_string(),
            kind: DartEnumKind::Error,
            tag_type: PrimitiveType::I32,
            variants: vec![],
            doc: None,
        };
        let template = EnumPlaceholderTemplate {
            enum_def: &enum_def,
        };
        insta::assert_snapshot!(template.render().unwrap());
    }

    #[test]
    fn snapshot_wire_functions_scalar_and_enum() {
        let funcs = vec![
            // primitive in / primitive out
            DartWireFunction {
                name: "add".to_string(),
                ffi_name: "boltffi_add".to_string(),
                params: vec![
                    DartWireFunctionParam {
                        name: "a".to_string(),
                        dart_type: "int".to_string(),
                        native_arg: "a".to_string(),
                    },
                    DartWireFunctionParam {
                        name: "b".to_string(),
                        dart_type: "int".to_string(),
                        native_arg: "b".to_string(),
                    },
                ],
                ret: DartWireReturn::Scalar {
                    dart_type: "int".to_string(),
                },
                doc: None,
            },
            // void
            DartWireFunction {
                name: "noop".to_string(),
                ffi_name: "boltffi_noop".to_string(),
                params: vec![],
                ret: DartWireReturn::Void,
                doc: None,
            },
            // enum in / enum out
            DartWireFunction {
                name: "echoPriority".to_string(),
                ffi_name: "boltffi_echo_priority".to_string(),
                params: vec![DartWireFunctionParam {
                    name: "p".to_string(),
                    dart_type: "Priority".to_string(),
                    native_arg: "p.value".to_string(),
                }],
                ret: DartWireReturn::EnumScalar {
                    dart_type: "Priority".to_string(),
                    enum_name: "Priority".to_string(),
                },
                doc: None,
            },
        ];
        let template = WireFunctionsTemplate { funcs: &funcs };
        insta::assert_snapshot!(template.render().unwrap());
    }
}
