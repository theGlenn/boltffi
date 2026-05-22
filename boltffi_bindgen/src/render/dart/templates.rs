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
#[template(path = "render_dart/record.txt", escape = "none")]
pub struct RecordTemplate<'a> {
    pub record: &'a super::DartRecord,
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
        DartBlittableField, DartBlittableLayout, DartConstructor, DartConstructorKind,
        DartFunction, DartFunctionParam, DartNativeFunction, DartNativeFunctionParam,
        DartNativeType, DartRecord, DartRecordField, DartType,
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
}
